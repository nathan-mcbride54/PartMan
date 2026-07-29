//! The interlock's whole job is refusing, so these tests are mostly about what
//! it must never accept.

use std::fs;
use std::path::{Path, PathBuf};

use super::{Authorization, DESTRUCTIVE_PROFILE, Refusal, Request, authorize};
use crate::catalogue;
use crate::manifest::Manifest;

/// A generated fixture tree in a unique temporary directory.
struct Sandbox {
    root: PathBuf,
    manifest: Manifest,
}

impl Sandbox {
    fn new(tag: &str) -> Self {
        // Process id and a per-process counter, so two concurrent runs of this
        // crate's tests cannot delete each other's fixture trees. A fixed name
        // made the suite that gates destructive execution flaky by
        // construction.
        let root = std::env::temp_dir().join(format!(
            "partman-interlock-{tag}-{}-{}",
            std::process::id(),
            crate::test_support::next_sandbox_id()
        ));
        // Tests may re-run after a failure, so start clean.
        let _ = fs::remove_dir_all(&root);
        let manifest = catalogue::generate(&root).expect("generating fixtures must succeed");
        Self { root, manifest }
    }

    fn target(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn request(&self, targets: Vec<PathBuf>) -> Request {
        Request {
            profile: Some(DESTRUCTIVE_PROFILE.to_owned()),
            token: Some(self.manifest.token().to_owned()),
            targets,
        }
    }

    fn authorize(&self, request: &Request) -> Result<Authorization, Refusal> {
        authorize(&self.root, request)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn all_three_factors_together_authorize() {
    let sandbox = Sandbox::new("happy");
    let target = sandbox.target("blank-512.img");
    let authorization = sandbox
        .authorize(&sandbox.request(vec![target.clone()]))
        .expect("a generated fixture with profile and token must authorize");
    assert_eq!(authorization.targets().len(), 1);
    assert!(authorization.targets()[0].ends_with("blank-512.img"));
}

#[test]
fn no_single_factor_is_sufficient() {
    let sandbox = Sandbox::new("single-factor");
    let target = sandbox.target("blank-512.img");

    // SAFE-007 states this outright: one variable is not proof. Each of these
    // requests carries exactly one of the three factors.
    let profile_only = Request {
        profile: Some(DESTRUCTIVE_PROFILE.to_owned()),
        token: None,
        targets: Vec::new(),
    };
    assert!(matches!(
        sandbox.authorize(&profile_only),
        Err(Refusal::TokenMissing)
    ));

    let token_only = Request {
        profile: None,
        token: Some(sandbox.manifest.token().to_owned()),
        targets: Vec::new(),
    };
    assert!(matches!(
        sandbox.authorize(&token_only),
        Err(Refusal::ProfileMissing)
    ));

    let target_only = Request {
        profile: None,
        token: None,
        targets: vec![target],
    };
    assert!(matches!(
        sandbox.authorize(&target_only),
        Err(Refusal::ProfileMissing)
    ));
}

#[test]
fn two_of_three_factors_are_still_refused() {
    let sandbox = Sandbox::new("two-factor");
    let target = sandbox.target("blank-512.img");

    let no_token = Request {
        profile: Some(DESTRUCTIVE_PROFILE.to_owned()),
        token: None,
        targets: vec![target.clone()],
    };
    assert!(matches!(
        sandbox.authorize(&no_token),
        Err(Refusal::TokenMissing)
    ));

    let no_profile = Request {
        profile: None,
        token: Some(sandbox.manifest.token().to_owned()),
        targets: vec![target],
    };
    assert!(matches!(
        sandbox.authorize(&no_profile),
        Err(Refusal::ProfileMissing)
    ));

    let no_targets = sandbox.request(Vec::new());
    assert!(matches!(
        sandbox.authorize(&no_targets),
        Err(Refusal::NoTargets)
    ));
}

#[test]
fn an_empty_target_list_is_refused_rather_than_vacuously_accepted() {
    // "Every target was verified" is trivially true of no targets. A suite that
    // passed the interlock while naming nothing would be claiming a proof it
    // never made.
    let sandbox = Sandbox::new("vacuous");
    assert!(matches!(
        sandbox.authorize(&sandbox.request(Vec::new())),
        Err(Refusal::NoTargets)
    ));
}

#[test]
fn a_wrong_profile_word_is_refused() {
    let sandbox = Sandbox::new("profile-word");
    let mut request = sandbox.request(vec![sandbox.target("blank-512.img")]);
    for word in ["Destructive", "destructive ", "", "safe", "DESTRUCTIVE"] {
        request.profile = Some(word.to_owned());
        assert!(
            matches!(sandbox.authorize(&request), Err(Refusal::ProfileMissing)),
            "profile {word:?} must not authorize"
        );
    }
}

#[test]
fn a_wrong_token_is_refused() {
    let sandbox = Sandbox::new("token");
    let mut request = sandbox.request(vec![sandbox.target("blank-512.img")]);

    let good = sandbox.manifest.token().to_owned();
    let mut truncated = good.clone();
    truncated.pop();
    let mut altered = good.clone();
    altered.replace_range(0..1, if good.starts_with('a') { "b" } else { "a" });

    for bad in [String::new(), truncated, altered, "0".repeat(64)] {
        request.token = Some(bad.clone());
        assert!(
            matches!(sandbox.authorize(&request), Err(Refusal::TokenMismatch)),
            "token {bad:?} must not authorize"
        );
    }
}

#[test]
fn a_file_outside_the_fixture_root_is_refused() {
    let sandbox = Sandbox::new("outside");
    let outside = std::env::temp_dir().join("partman-interlock-outside-file.img");
    // Give it the exact bytes of a real fixture, so only the location differs.
    let bytes = fs::read(sandbox.target("blank-512.img")).expect("fixture must be readable");
    fs::write(&outside, &bytes).expect("writing the decoy must succeed");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![outside.clone()]))
        .expect_err("a file outside the fixture root must be refused");
    assert!(matches!(refusal, Refusal::TargetOutsideRoot { .. }));

    let _ = fs::remove_file(&outside);
}

#[test]
fn traversal_out_of_the_fixture_root_is_refused() {
    let sandbox = Sandbox::new("traversal");
    let escaped = sandbox.root.join("..").join("partman-interlock-escape.img");
    let bytes = fs::read(sandbox.target("blank-512.img")).expect("fixture must be readable");
    fs::write(&escaped, &bytes).expect("writing the decoy must succeed");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![escaped.clone()]))
        .expect_err("`..` must not reach outside the fixture root");
    assert!(matches!(refusal, Refusal::TargetOutsideRoot { .. }));

    let _ = fs::remove_file(&escaped);
}

#[test]
fn a_missing_target_is_refused_rather_than_ignored() {
    let sandbox = Sandbox::new("missing");
    let refusal = sandbox
        .authorize(&sandbox.request(vec![sandbox.target("does-not-exist.img")]))
        .expect_err("a missing target must fail closed");
    assert!(matches!(refusal, Refusal::TargetUnresolvable { .. }));
}

#[test]
fn a_directory_is_refused() {
    let sandbox = Sandbox::new("directory");
    let refusal = sandbox
        .authorize(&sandbox.request(vec![sandbox.root.clone()]))
        .expect_err("a directory is not a disposable image");
    assert!(matches!(refusal, Refusal::TargetNotRegularFile { .. }));
}

#[test]
fn an_ungenerated_file_inside_the_root_is_refused() {
    // The decisive property: being in the right directory is not enough. Only
    // bytes this repository generated pass, and that is a computed fact.
    let sandbox = Sandbox::new("ungenerated");
    let intruder = sandbox.target("not-ours.img");
    fs::write(&intruder, b"this is not a generated fixture").expect("writing must succeed");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![intruder]))
        .expect_err("an unrecognized file must be refused");
    assert!(matches!(refusal, Refusal::TargetNotGenerated { .. }));
}

#[test]
fn a_modified_fixture_is_refused() {
    let sandbox = Sandbox::new("modified");
    let target = sandbox.target("blank-512.img");
    let mut bytes = fs::read(&target).expect("fixture must be readable");
    bytes[0] ^= 0xff;
    fs::write(&target, &bytes).expect("rewriting must succeed");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![target]))
        .expect_err("a fixture whose bytes changed is no longer verified");
    assert!(matches!(refusal, Refusal::TargetNotGenerated { .. }));
}

#[test]
fn one_bad_target_refuses_the_whole_request() {
    // Partial authorization would be worse than none: the caller would have a
    // list it believes is verified, containing something that is not.
    let sandbox = Sandbox::new("mixed");
    let good = sandbox.target("blank-512.img");
    let bad = sandbox.target("not-ours.img");
    fs::write(&bad, b"unrecognized").expect("writing must succeed");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![good, bad]))
        .expect_err("one unverifiable target must refuse the request");
    assert!(matches!(refusal, Refusal::TargetNotGenerated { .. }));
}

#[test]
fn every_generated_fixture_authorizes() {
    let sandbox = Sandbox::new("all");
    let targets: Vec<PathBuf> = sandbox
        .manifest
        .names()
        .map(|n| sandbox.target(n))
        .collect();
    // Equality, not `>= 8`. A floor lets entries be deleted silently while the
    // test still reads as coverage, which is the same shape of defect as a
    // fixture whose rationale nothing checks.
    assert_eq!(
        targets.len(),
        catalogue::catalogue().len(),
        "every catalogue fixture must be generated and verifiable"
    );
    let authorization = sandbox
        .authorize(&sandbox.request(targets.clone()))
        .expect("every generated fixture must be verifiable");
    assert_eq!(authorization.targets().len(), targets.len());
}

#[cfg(unix)]
#[test]
fn a_symlink_is_refused_even_when_it_points_at_a_fixture() {
    // A symlink is the shape of the attack this check exists for: a name inside
    // the fixture root that resolves somewhere else. Refusing the link itself is
    // simpler to reason about than resolving it and re-checking.
    let sandbox = Sandbox::new("symlink");
    let link = sandbox.target("link-to-fixture.img");
    std::os::unix::fs::symlink(sandbox.target("blank-512.img"), &link)
        .expect("creating the symlink must succeed");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![link]))
        .expect_err("a symlink must be refused");
    assert!(matches!(refusal, Refusal::TargetNotRegularFile { .. }));
}

#[test]
fn refusals_say_what_to_do_next() {
    // UI-010's standard applied to the runner: a refusal a developer cannot act
    // on gets worked around rather than fixed.
    let sandbox = Sandbox::new("messages");
    let refusal = sandbox
        .authorize(&Request {
            profile: None,
            token: None,
            targets: Vec::new(),
        })
        .expect_err("must refuse");
    let text = refusal.to_string();
    assert!(text.contains(DESTRUCTIVE_PROFILE), "{text}");

    let token_refusal = sandbox
        .authorize(&Request {
            profile: Some(DESTRUCTIVE_PROFILE.to_owned()),
            token: None,
            targets: Vec::new(),
        })
        .expect_err("must refuse");
    assert!(token_refusal.to_string().contains("MANIFEST"));
}

#[test]
fn authorization_cannot_be_forged_outside_this_module() {
    // A compile-time property stated as a comment because it cannot be a test:
    // `Authorization` has a private field and no public constructor, so the only
    // way to obtain one is `authorize`. If that ever changes, this assertion's
    // neighbours stop meaning anything.
    fn takes_proof(authorization: &Authorization) -> usize {
        authorization.targets().len()
    }

    let sandbox = Sandbox::new("forge");
    let authorization = sandbox
        .authorize(&sandbox.request(vec![sandbox.target("blank-512.img")]))
        .expect("must authorize");
    assert_eq!(takes_proof(&authorization), 1);
}

#[test]
fn the_fixture_root_must_exist() {
    let sandbox = Sandbox::new("no-root");
    let manifest = sandbox.manifest.clone();
    let missing = Path::new("this-directory-does-not-exist-partman");
    let request = Request {
        profile: Some(DESTRUCTIVE_PROFILE.to_owned()),
        token: Some(manifest.token().to_owned()),
        targets: vec![missing.join("x.img")],
    };
    let refusal = authorize(missing, &request).expect_err("must fail closed");
    assert!(matches!(refusal, Refusal::ManifestUnreadable(_)));
}

#[test]
fn a_forged_manifest_cannot_authorize_a_file_the_generator_never_produced() {
    // The defect a project review found, reproduced as a test. Previously the
    // interlock derived its expectations from `MANIFEST` -- a user-writable file
    // in the very directory being verified -- so writing a manifest that named
    // an arbitrary file's digest, with any well-formed token, authorized it.
    //
    // Expectations now come from the compiled catalogue, so nothing written to
    // the fixture directory can change what the interlock expects.
    let sandbox = Sandbox::new("forged-manifest");

    let victim = sandbox.target("victim.img");
    fs::write(&victim, b"pretend this is the user's boot disk").expect("writing must succeed");

    let digest = crate::manifest::hex(&sha2::Sha256::digest(
        b"pretend this is the user's boot disk",
    ));
    let forged = format!(
        "{}\ntoken {}\nimage {digest} 36 victim.img\n",
        crate::manifest::MANIFEST_HEADER,
        "a".repeat(64),
    );
    fs::write(sandbox.root.join(crate::manifest::MANIFEST_FILE), forged)
        .expect("writing the forged manifest must succeed");

    let request = Request {
        profile: Some(DESTRUCTIVE_PROFILE.to_owned()),
        token: Some("a".repeat(64)),
        targets: vec![victim],
    };
    let refusal = sandbox
        .authorize(&request)
        .expect_err("a forged manifest must not authorize anything");
    assert!(matches!(refusal, Refusal::TokenMismatch), "{refusal}");
}

#[test]
fn a_real_fixture_under_the_wrong_name_is_refused() {
    // Membership by digest alone was too weak: any file passed so long as some
    // manifest entry shared its digest. A fixture's bytes under another name are
    // still not that fixture.
    let sandbox = Sandbox::new("wrong-name");
    let bytes = fs::read(sandbox.target("blank-512.img")).expect("fixture must be readable");
    let renamed = sandbox.target("not-a-catalogue-name.img");
    fs::write(&renamed, &bytes).expect("writing must succeed");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![renamed]))
        .expect_err("a fixture under an unexpected name must be refused");
    assert!(
        matches!(refusal, Refusal::TargetNotGenerated { .. }),
        "{refusal}"
    );
}

#[test]
fn a_fixture_whose_bytes_belong_to_a_different_fixture_is_refused() {
    // The other half of the same defect: right directory, right catalogue name,
    // but the content of a different entry.
    let sandbox = Sandbox::new("swapped-content");
    let other = fs::read(sandbox.target("mbr-basic-512.img")).expect("readable");
    fs::write(sandbox.target("blank-512.img"), &other).expect("writing must succeed");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![sandbox.target("blank-512.img")]))
        .expect_err("content must match the entry for that name, not merely some entry");
    assert!(
        matches!(refusal, Refusal::TargetNotGenerated { .. }),
        "{refusal}"
    );
}

#[cfg(unix)]
#[test]
fn a_hard_link_into_the_fixture_root_is_refused() {
    // A hard link is a regular file and canonicalizes inside the root, so
    // neither of those checks sees it. Requiring content to equal a generated
    // fixture already means a link can only point at something that is one, but
    // a second name is still a second thing a destructive suite could reach.
    let sandbox = Sandbox::new("hard-link");
    let real = sandbox.target("blank-512.img");
    let link = sandbox.target("gpt-basic-512.img");
    fs::remove_file(&link).expect("clearing the target name must succeed");
    fs::hard_link(&real, &link).expect("creating the hard link must succeed");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![link]))
        .expect_err("a hard-linked target must be refused");
    assert!(
        matches!(
            refusal,
            Refusal::TargetHasOtherNames { .. } | Refusal::TargetNotGenerated { .. }
        ),
        "{refusal}"
    );
}

use sha2::Digest as _;

#[test]
fn a_fixture_copy_in_a_subdirectory_is_refused() {
    // The hole the first attempt at this fix left open, and which its own new
    // tests did not catch. Containment was `starts_with(root)` and the name came
    // from `file_name()`, so a byte-identical copy at `<root>/sub/blank-512.img`
    // satisfied the root check, the name lookup, the length check and the digest
    // check at once. It authorized. The path must be exactly where that fixture
    // is generated, not merely underneath the root.
    let sandbox = Sandbox::new("subdirectory");
    let sub = sandbox.root.join("sub");
    fs::create_dir_all(&sub).expect("creating the subdirectory must succeed");
    let bytes = fs::read(sandbox.target("blank-512.img")).expect("fixture must be readable");
    let copy = sub.join("blank-512.img");
    fs::write(&copy, &bytes).expect("writing the copy must succeed");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![copy]))
        .expect_err("a copy in a subdirectory must be refused");
    assert!(
        matches!(refusal, Refusal::TargetOutsideRoot { .. }),
        "{refusal}"
    );
}
