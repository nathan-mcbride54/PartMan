//! The interlock's whole job is refusing, so these tests are mostly about what
//! it must never accept.

use std::fs;
use std::path::{Path, PathBuf};

use super::{Authorization, DESTRUCTIVE_PROFILE, Refusal, Request, authorize};
use crate::catalogue;
use crate::manifest::Manifest;

/// An action to run at the pre-open seam.
type PreOpenAction = Box<dyn Fn(&Path)>;

std::thread_local! {
    /// Test hook invoked immediately before a target is opened.
    ///
    /// The seam the 2026-07-29 follow-up audit asked for. The pre-open race is
    /// a race, and a test that merely runs two operations quickly samples it
    /// rather than proving anything; this makes the interleaving exact.
    /// Thread-local because the test harness gives each test its own thread,
    /// so one test's hook can never fire inside another's.
    static BEFORE_OPEN: std::cell::RefCell<Option<PreOpenAction>> =
        const { std::cell::RefCell::new(None) };
}

/// Invoked by `verify_target` between canonicalization and `open`.
pub(super) fn run_before_open_hook(resolved: &Path) {
    BEFORE_OPEN.with(|hook| {
        if let Some(action) = hook.borrow().as_ref() {
            action(resolved);
        }
    });
}

/// Install a pre-open action for the duration of `body`.
fn with_before_open<R>(action: impl Fn(&Path) + 'static, body: impl FnOnce() -> R) -> R {
    BEFORE_OPEN.with(|hook| *hook.borrow_mut() = Some(Box::new(action)));
    let result = body();
    BEFORE_OPEN.with(|hook| *hook.borrow_mut() = None);
    result
}

/// A generated fixture tree in a unique temporary directory.
struct Sandbox {
    root: PathBuf,
    manifest: Manifest,
}

/// Where sandboxes are built.
///
/// Defaults to the system temporary directory, and `PARTMAN_TEST_ROOT`
/// overrides it. The override is not a convenience: on Windows the containment
/// guarantee is a property of *the filesystem serving the root*, and a
/// developer's temporary directory need not be the same filesystem as the
/// clone the fixtures are really generated into. On the machine this increment
/// was written on they differ — `%TEMP%` is on an NTFS volume and the working
/// copy is on a `ReFS` one — so a green suite is evidence about `%TEMP%` unless
/// someone points it elsewhere.
///
/// It was pointed elsewhere: the whole suite was run with the root on that
/// `ReFS` volume and passed, which is how the separator defect below was found.
fn sandbox_base() -> PathBuf {
    let base = std::env::var_os("PARTMAN_TEST_ROOT").map_or_else(std::env::temp_dir, PathBuf::from);
    // Re-collecting through `components` normalises the separators. Windows
    // accepts `/` in a path, but `mklink` is parsed by `cmd`, which reads a
    // leading `/` as the start of a switch — so `PARTMAN_TEST_ROOT=D:/x/y`
    // made junction creation fail and the root-swap test blame the *platform*
    // for it. Found by pointing the suite at this repository's own ReFS volume,
    // which is exactly what the override exists for.
    base.components().collect()
}

impl Sandbox {
    fn new(tag: &str) -> Self {
        // Process id and a per-process counter, so two concurrent runs of this
        // crate's tests cannot delete each other's fixture trees. A fixed name
        // made the suite that gates destructive execution flaky by
        // construction.
        let root = sandbox_base().join(format!(
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
        // Discarded deliberately, and since increment 2d this can genuinely
        // fail rather than merely being tidy: on Windows a live `Authorization`
        // holds the root directory open with a share mode that refuses
        // deletion, so this returns `ERROR_SHARING_VIOLATION` and the tree
        // leaks into the temporary directory.
        //
        // Nothing leaks today only because Rust drops locals in reverse
        // declaration order and every test here declares its sandbox first.
        // That is an ordering coincidence, not a design — a test that binds an
        // authorization before its sandbox would leak silently. Tests that hold
        // an authorization past the point of interest drop it explicitly.
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Create a directory junction, which needs no privilege on Windows — unlike a
/// symlink, which needs `SeCreateSymbolicLinkPrivilege` and is therefore not
/// something a CI runner can be relied on to allow.
#[cfg(windows)]
fn junction(link: &Path, target: &Path) -> bool {
    std::process::Command::new("cmd")
        .args(["/c", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .output()
        .is_ok_and(|output| output.status.success())
}

/// The object identity of a path, opened without following a reparse point.
///
/// Windows' analogue of `ino()`: a decoy holding a fixture's exact bytes is
/// indistinguishable by content, so every containment assertion here is about
/// *which object* was authorized.
#[cfg(windows)]
fn identity_of(path: &Path) -> std::io::Result<(u64, u64)> {
    use std::os::windows::fs::OpenOptionsExt as _;

    let file = fs::OpenOptions::new()
        .read(true)
        .share_mode(7)
        .custom_flags(0x0020_0000)
        .open(path)?;
    super::object_identity(&file)
}

#[test]
fn all_three_factors_together_authorize() {
    let sandbox = Sandbox::new("happy");
    let target = sandbox.target("blank-512.img");
    let authorization = sandbox
        .authorize(&sandbox.request(vec![target.clone()]))
        .expect("a generated fixture with profile and token must authorize");
    assert_eq!(authorization.targets().len(), 1);
    assert!(authorization.targets()[0].path().ends_with("blank-512.img"));
}

#[test]
fn authorization_holds_the_object_it_verified_not_the_name() {
    // The replace-after-authorization test the WP-020 preconditions demand,
    // and the reason `Authorization` carries open files instead of paths. The
    // attack: authorize a real fixture, then rebind its *name* to something
    // else before the destructive suite writes. With a `Vec<PathBuf>` the
    // suite would have written to whatever the name now points at; the audit
    // called this the most important precondition before any Tier-2 write.
    let sandbox = Sandbox::new("replace-after-check");
    let target = sandbox.target("blank-512.img");
    let authorization = sandbox
        .authorize(&sandbox.request(vec![target.clone()]))
        .expect("the untouched fixture must authorize");

    let moved_aside = sandbox.root.join("moved-aside.img");

    #[cfg(windows)]
    {
        // The share mode on the held handle refuses every rebinding of the
        // name while the authorization lives: no rename away, no deletion, no
        // second write handle. The swap is stopped at step one.
        assert!(
            fs::rename(&target, &moved_aside).is_err(),
            "renaming a file whose verified handle is held must fail on Windows"
        );
        assert!(
            fs::remove_file(&target).is_err(),
            "deleting a file whose verified handle is held must fail on Windows"
        );
        assert!(
            fs::OpenOptions::new().write(true).open(&target).is_err(),
            "a second write handle to a verified target must be refused on Windows"
        );
        drop(authorization);
        // And the refusals end with the authorization: the handle is the
        // enforcement, not some persistent state.
        fs::rename(&target, &moved_aside).expect("after drop the name is free again");
    }

    #[cfg(unix)]
    {
        // POSIX has no mandatory locking to refuse the rename, so the name
        // *can* be rebound — and the guarantee is the stronger fact that it
        // does not matter. The held handle is the verified inode; writes
        // through it reach the object that was checked, and the impostor now
        // sitting at the verified path never sees a byte.
        use std::io::{Seek as _, SeekFrom, Write as _};

        fs::rename(&target, &moved_aside).expect("rename the verified object aside");
        fs::write(&target, b"impostor").expect("plant an impostor at the verified name");

        let mut targets = authorization.into_targets();
        let mut file = targets.pop().expect("one verified target").into_file();
        file.seek(SeekFrom::Start(0)).expect("rewind");
        file.write_all(b"DESTRUCTIVE-WRITE")
            .expect("write through the verified handle");
        file.sync_all().expect("flush");
        drop(file);

        let impostor = fs::read(&target).expect("read the impostor");
        assert_eq!(
            impostor, b"impostor",
            "the impostor at the verified name must never see a write"
        );
        let moved = fs::read(&moved_aside).expect("read the verified object");
        assert!(
            moved.starts_with(b"DESTRUCTIVE-WRITE"),
            "the write must have reached the object that was verified"
        );
    }
}

#[cfg(unix)]
#[test]
fn a_symlink_swapped_in_before_open_is_refused() {
    // The 2026-07-29 follow-up audit's finding 2, scheduled rather than
    // sampled. Every path check has already passed when the hook fires; the
    // name is then rebound to a symlink aimed at an out-of-root file holding
    // *the fixture's exact bytes*, so every handle-based check — regular file,
    // link count, length, digest — would accept the object if the open
    // followed the link.
    //
    // Before `O_NOFOLLOW` this authorized a handle outside the fixture tree.
    let sandbox = Sandbox::new("pre-open-symlink");
    let target = sandbox.target("blank-512.img");
    let bytes = fs::read(&target).expect("read the fixture");

    let outside = std::env::temp_dir().join(format!(
        "partman-preopen-{}-{}.img",
        std::process::id(),
        crate::test_support::next_sandbox_id()
    ));
    fs::write(&outside, &bytes).expect("an identical file outside the root");

    let decoy = outside.clone();
    let refusal = with_before_open(
        move |resolved| {
            // Runs after canonicalization, before open. Replace the verified
            // name with a link to the out-of-root twin.
            let _ = fs::remove_file(resolved);
            let _ = std::os::unix::fs::symlink(&decoy, resolved);
        },
        || sandbox.authorize(&sandbox.request(vec![target.clone()])),
    );

    let _ = fs::remove_file(&outside);
    let refusal = refusal.expect_err(
        "a symlink swapped in before the open must be refused; following it would authorize \
         an object outside the fixture root",
    );
    // `O_NOFOLLOW` surfaces as ELOOP on the open, so this is an unresolvable
    // target rather than a symlink classified by name.
    assert!(
        matches!(refusal, Refusal::TargetUnresolvable { .. }),
        "expected the open itself to refuse, got {refusal:?}"
    );
}

#[test]
fn an_object_swapped_in_before_open_is_refused_on_every_platform() {
    // The symlink test above is Unix-only, because creating a symlink on
    // Windows needs a privilege CI cannot be relied on to hold. This one runs
    // everywhere and establishes the two things that are portable: the pre-open
    // seam really does fire inside `verify_target`, and an object substituted
    // at the verified name after every path check has passed is still refused
    // by the handle-based checks.
    let sandbox = Sandbox::new("pre-open-swap");
    let target = sandbox.target("blank-512.img");

    let refusal = with_before_open(
        |resolved| {
            // A directory at the verified name: the path checks are already
            // done, so only a check against the opened object can catch this.
            let _ = fs::remove_file(resolved);
            let _ = fs::create_dir(resolved);
        },
        || sandbox.authorize(&sandbox.request(vec![target.clone()])),
    );

    let refusal = refusal.expect_err("an object substituted before the open must be refused");
    assert!(
        matches!(
            refusal,
            Refusal::TargetNotRegularFile { .. } | Refusal::TargetUnresolvable { .. }
        ),
        "expected a refusal about the opened object, got {refusal:?}"
    );
}

#[test]
fn the_handle_handed_over_starts_at_offset_zero() {
    // `verify_object` hashes the contents, so the cursor sat at EOF and the
    // replace-after-authorization test had to seek explicitly — the smell the
    // follow-up audit named. A consumer handed a freshly authorized file will
    // reasonably assume offset zero, so the safe default is structural rather
    // than documented.
    use std::io::{Seek as _, SeekFrom};

    let sandbox = Sandbox::new("cursor-at-zero");
    let target = sandbox.target("blank-512.img");
    let authorization = sandbox
        .authorize(&sandbox.request(vec![target]))
        .expect("the fixture must authorize");
    let mut targets = authorization.into_targets();
    let mut file = targets.pop().expect("one verified target").into_file();
    let position = file
        .stream_position()
        .expect("read the cursor without moving it");
    assert_eq!(
        position, 0,
        "a consumer must receive the handle rewound, not positioned at EOF"
    );
    // And it is genuinely readable from the start.
    let mut first = [0_u8; 4];
    std::io::Read::read_exact(&mut file, &mut first).expect("read from offset zero");
    assert_eq!(
        file.stream_position().expect("cursor"),
        4,
        "reading advanced from zero, so the handle was not merely reporting zero"
    );
    let _ = file.seek(SeekFrom::Start(0));
}

#[cfg(unix)]
#[test]
fn swapping_the_fixture_root_directory_before_open_cannot_redirect_the_write() {
    // At the top, before any statement: `clippy::items_after_statements` is
    // denied workspace-wide, and this Unix-only body is not compiled on the
    // machine most of this work was done on, so CI caught it rather than the
    // local gate.
    use std::os::unix::fs::MetadataExt as _;

    // The 2026-07-29 second follow-up audit's F-02, scheduled through the same
    // seam as the basename swap. `O_NOFOLLOW` constrains only the *final* path
    // component — `open(2)` says intermediate components are still resolved
    // through symlinks — so increment 2b closed one form of this race and left
    // the more general one open.
    //
    // The attack: let every path check pass, then rename the fixture root aside
    // and leave a symlink at its name pointing to an outside directory holding
    // a file with the fixture's exact bytes. Under 2b the open followed that
    // intermediate link and authorized a handle outside the tree, because
    // length, digest, type and link count all matched.
    //
    // Under 2c the open resolves from a directory *handle* taken before the
    // swap, so the rename cannot reach it: the held object is still the real
    // fixture directory whatever its name now points at.
    let sandbox = Sandbox::new("root-swap");
    let target = sandbox.target("blank-512.img");
    let bytes = fs::read(&target).expect("read the fixture");

    let decoy_root = std::env::temp_dir().join(format!(
        "partman-decoy-root-{}-{}",
        std::process::id(),
        crate::test_support::next_sandbox_id()
    ));
    fs::create_dir_all(&decoy_root).expect("create the decoy directory");
    fs::write(decoy_root.join("blank-512.img"), &bytes)
        .expect("an identical file inside the decoy");

    let sandbox_root = sandbox.root.clone();
    let sandbox_root_moved = sandbox.root.with_extension("moved-aside");
    let real_root = sandbox_root.clone();
    let moved_root = sandbox_root_moved.clone();
    let decoy = decoy_root.clone();
    let authorization = with_before_open(
        move |_resolved| {
            // Runs after canonicalization and the containment checks, before
            // the child is opened — exactly the window `open(2)` leaves for an
            // intermediate component.
            if real_root.is_dir() {
                let _ = fs::rename(&real_root, &moved_root);
                let _ = std::os::unix::fs::symlink(&decoy, &real_root);
            }
        },
        || sandbox.authorize(&sandbox.request(vec![target.clone()])),
    );

    // Content cannot distinguish the two files — that is the whole point of the
    // attack, and why a "did it refuse?" assertion would prove nothing. Object
    // identity can. The authorized handle must be the inode inside the *real*
    // fixture directory, never the decoy's.
    let decoy_ino = fs::metadata(decoy_root.join("blank-512.img"))
        .expect("stat the decoy")
        .ino();
    let real_ino = fs::metadata(sandbox_root_moved.join("blank-512.img"))
        .expect("stat the real fixture at its moved-aside name")
        .ino();
    assert_ne!(
        decoy_ino, real_ino,
        "sanity: the decoy and the real fixture must be different objects"
    );

    // Prove the race was actually staged: the fixture root's *name* now leads
    // to the decoy. If this fails the test is not exercising anything.
    let via_name = fs::canonicalize(sandbox_root.join("blank-512.img"))
        .expect("the swapped name must still resolve");
    assert_eq!(
        fs::metadata(&via_name)
            .expect("stat via the swapped name")
            .ino(),
        decoy_ino,
        "the fixture root's name must now lead to the decoy, or the swap did not happen"
    );

    let authorized_ino = authorization.map(|authorization| {
        let mut targets = authorization.into_targets();
        let file = targets.pop().expect("one verified target").into_file();
        file.metadata().expect("fstat the authorized handle").ino()
    });

    // The sandbox's own cleanup cannot reach the tree any more: its recorded
    // root is now a symlink and the real directory lives under another name.
    let _ = fs::remove_file(&sandbox_root);
    let _ = fs::remove_dir_all(&sandbox_root_moved);
    let _ = fs::remove_dir_all(&decoy_root);

    match authorized_ino {
        // Refusing outright is safe.
        Err(_) => {}
        // Succeeding is safe only if the handle is the real fixture. Under the
        // increment-2b code this assertion is what fails: the open followed the
        // intermediate symlink and the handle was the decoy's inode.
        Ok(ino) => assert_eq!(
            ino, real_ino,
            "the authorized handle must be the object inside the real fixture directory, not \
             the decoy the root's name now points at"
        ),
    }
}

#[cfg(windows)]
#[test]
fn swapping_the_fixture_root_directory_before_open_cannot_redirect_the_write() {
    // The Windows half of F-02, and the counterpart to the Unix test above.
    //
    // The mechanisms differ and the test has to respect that. Unix *resolves*
    // the child from a held descriptor, so the swap succeeds and is harmless.
    // Windows opens by pathname, and what makes that sound is that the held
    // root handle's share mode makes the swap itself impossible. So here the
    // attack fails at step one — and a test that merely asserted "the rename
    // failed" would be worthless, because a rename of the root is *already*
    // refused once any target handle is open, which was true before this
    // increment. That test passes with the fix removed. Measured.
    //
    // So the assertion is the same one Unix makes: which object did we
    // authorize. Content cannot answer it — the decoy holds identical bytes —
    // and identity can.
    let sandbox = Sandbox::new("root-swap");
    let target = sandbox.target("blank-512.img");
    let bytes = fs::read(&target).expect("read the fixture");
    let real_identity = identity_of(&target).expect("identity of the real fixture");

    let decoy_root = sandbox_base().join(format!(
        "partman-decoy-root-{}-{}",
        std::process::id(),
        crate::test_support::next_sandbox_id()
    ));
    fs::create_dir_all(&decoy_root).expect("create the decoy directory");
    let decoy_target = decoy_root.join("blank-512.img");
    fs::write(&decoy_target, &bytes).expect("an identical file inside the decoy");
    let decoy_identity = identity_of(&decoy_target).expect("identity of the decoy");
    assert_ne!(
        real_identity, decoy_identity,
        "sanity: byte-identical files must still be distinguishable objects, or this test \
         cannot detect anything"
    );

    // Positive control, run first and in its own tree with nothing held: the
    // attack must genuinely work when it is not being defended against.
    // Without this the test would pass vacuously anywhere `mklink /J` fails.
    {
        let control_root = sandbox_base().join(format!(
            "partman-control-root-{}-{}",
            std::process::id(),
            crate::test_support::next_sandbox_id()
        ));
        fs::create_dir_all(&control_root).expect("create the control root");
        fs::write(control_root.join("blank-512.img"), &bytes).expect("control fixture");
        let control_identity =
            identity_of(&control_root.join("blank-512.img")).expect("control identity");
        let moved = control_root.with_extension("moved-aside");
        fs::rename(&control_root, &moved).expect("with nothing held, the root renames freely");
        assert!(
            junction(&control_root, &decoy_root),
            "could not create a junction at {}. The control has to succeed or the defended \
             case below proves nothing, so this is a failure rather than a skip. Check the \
             separators first — `mklink` is parsed by `cmd`, which reads a leading `/` as a \
             switch — then whether the volume supports reparse points. NTFS and ReFS both do",
            control_root.display()
        );
        let redirected =
            identity_of(&control_root.join("blank-512.img")).expect("open through the junction");
        assert_eq!(
            redirected, decoy_identity,
            "the control attack must reach the decoy, or the defended case is not being tested"
        );
        assert_ne!(redirected, control_identity);
        let _ = fs::remove_dir(&control_root);
        let _ = fs::remove_dir_all(&moved);
    }

    // Now the defended case, staged through the same seam the Unix test uses:
    // every path check has passed, no target is open yet, and the root handle
    // is the only thing standing between the attacker and a redirected open.
    let real_root = sandbox.root.clone();
    let moved_root = sandbox.root.with_extension("moved-aside");
    let decoy = decoy_root.clone();
    let swap_succeeded = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = std::sync::Arc::clone(&swap_succeeded);
    let authorization = with_before_open(
        move |_resolved| {
            if fs::rename(&real_root, &moved_root).is_ok() {
                observed.store(true, std::sync::atomic::Ordering::SeqCst);
                junction(&real_root, &decoy);
            }
        },
        || sandbox.authorize(&sandbox.request(vec![target.clone()])),
    );

    let authorization = authorization.expect(
        "the fixture is untouched and the swap cannot land, so this must authorize rather than \
         refuse; a refusal here would hide whether containment held",
    );
    let mut targets = authorization.into_targets();
    let file = targets.pop().expect("one verified target").into_file();
    let authorized_identity = super::object_identity(&file).expect("identity of the handle");
    drop(file);

    assert_eq!(
        authorized_identity, real_identity,
        "the authorized handle must be the object inside the real fixture directory, never the \
         decoy the root's name was aimed at"
    );
    // Recorded, not asserted as the property: *how* it was stopped. With the
    // root handle removed this flips to true and the assertion above fails,
    // which is the mutation this test exists to catch.
    assert!(
        !swap_succeeded.load(std::sync::atomic::Ordering::SeqCst),
        "the root rename should have been refused while the root handle was held"
    );

    let _ = fs::remove_dir_all(&decoy_root);
}

#[cfg(windows)]
#[test]
fn the_root_handle_alone_refuses_renaming_the_root() {
    // Isolates the new mechanism from everything that already existed. No
    // target is open, so the refusal below can only come from the root handle's
    // share mode — unlike a rename attempted while an `Authorization` is alive,
    // which the target handle already refused before this increment.
    //
    // Fails if the root handle is removed, and fails if `FILE_SHARE_DELETE` is
    // ever added to it: measured, `share = READ` and `READ|WRITE` both refuse,
    // `READ|WRITE|DELETE` permits.
    let sandbox = Sandbox::new("root-handle-alone");
    let moved = sandbox.root.with_extension("moved-aside");

    let held = super::RootDirectory::open(&sandbox.root).expect("the fixture root must open");
    assert!(
        fs::rename(&sandbox.root, &moved).is_err(),
        "a held root directory handle must make the root un-renamable"
    );
    drop(held);

    // Control: the refusal is the handle, not something ambient about the path.
    fs::rename(&sandbox.root, &moved).expect("once the handle is dropped the root renames freely");
    fs::rename(&moved, &sandbox.root).expect("put it back so the sandbox can clean up");
}

#[cfg(windows)]
#[test]
fn an_entry_replaced_by_a_junction_is_refused() {
    // The root handle protects the directory *object*, not the entries in it:
    // an entry can still be deleted and something else put at its name. That is
    // fine, and this pins why.
    //
    // It also guards a specific footgun this increment introduced. The root
    // open now needs `FILE_FLAG_BACKUP_SEMANTICS`, and copying that flag down
    // into `open_child` would make this junction *open* instead of being
    // refused — and report `is_file()`, so the regular-file check would not
    // catch it either. Measured, junction at the child name: no flags → os 5;
    // `OPEN_REPARSE_POINT` → os 5; `OPEN_REPARSE_POINT | BACKUP_SEMANTICS` →
    // opened.
    //
    // Deliberately **not** labelled as coverage of `FILE_FLAG_OPEN_REPARSE_POINT`:
    // the junction is refused with or without that flag, because opening a
    // directory without backup semantics fails either way.
    let sandbox = Sandbox::new("entry-junction");
    let target = sandbox.target("blank-512.img");
    let bytes = fs::read(&target).expect("read the fixture");

    let outside = sandbox_base().join(format!(
        "partman-junction-target-{}-{}",
        std::process::id(),
        crate::test_support::next_sandbox_id()
    ));
    fs::create_dir_all(&outside).expect("create the junction's target");
    fs::write(outside.join("blank-512.img"), &bytes).expect("identical bytes beyond the junction");

    let elsewhere = outside.clone();
    let refusal = with_before_open(
        move |resolved| {
            let _ = fs::remove_file(resolved);
            junction(resolved, &elsewhere);
        },
        || sandbox.authorize(&sandbox.request(vec![target.clone()])),
    );

    let refusal = refusal.expect_err("a junction at the fixture's name must be refused");
    assert!(
        matches!(refusal, Refusal::TargetUnresolvable { .. }),
        "{refusal:?}"
    );
    let _ = fs::remove_dir_all(&outside);
}

#[cfg(windows)]
#[test]
fn a_file_symlink_swapped_in_before_open_is_refused_as_an_irregular_object() {
    // The Windows counterpart to `a_symlink_swapped_in_before_open_is_refused`,
    // and the one piece of coverage here that the environment can take away:
    // creating a file symlink needs `SeCreateSymbolicLinkPrivilege` or
    // Developer Mode, and a CI runner cannot be relied on to have either.
    //
    // It says so out loud when it cannot run. A test that skips in silence
    // stops being coverage without anyone noticing, and this repository has
    // already had to delete traceability rows that named evidence which no
    // longer existed.
    //
    // What it pins: the refusal comes from `is_file()` **through the handle**,
    // not from the length check and not from the by-path `symlink_metadata`
    // hygiene check that the interlock itself documents as raceable. Measured
    // for both a file symlink and a junction, with and without backup
    // semantics: `is_file()` is false in every case.
    let sandbox = Sandbox::new("pre-open-symlink");
    let target = sandbox.target("blank-512.img");
    let bytes = fs::read(&target).expect("read the fixture");

    let outside = sandbox_base().join(format!(
        "partman-symlink-decoy-{}-{}.img",
        std::process::id(),
        crate::test_support::next_sandbox_id()
    ));
    fs::write(&outside, &bytes).expect("an identical file outside the root");

    // Probe the privilege before staging anything, so the skip is clean.
    let probe = sandbox.root.join("privilege-probe.img");
    let permitted = std::os::windows::fs::symlink_file(&outside, &probe).is_ok();
    let _ = fs::remove_file(&probe);
    if !permitted {
        println!(
            "SKIPPED a_file_symlink_swapped_in_before_open_is_refused_as_an_irregular_object: \
             this host cannot create a file symlink (no SeCreateSymbolicLinkPrivilege and no \
             Developer Mode). The reparse-point path is therefore UNTESTED here."
        );
        let _ = fs::remove_file(&outside);
        return;
    }

    let decoy = outside.clone();
    let refusal = with_before_open(
        move |resolved| {
            let _ = fs::remove_file(resolved);
            let _ = std::os::windows::fs::symlink_file(&decoy, resolved);
        },
        || sandbox.authorize(&sandbox.request(vec![target.clone()])),
    );

    let refusal = refusal.expect_err(
        "a symlink swapped in before the open must be refused; following it would authorize an \
         object outside the fixture root",
    );
    assert!(
        matches!(refusal, Refusal::TargetNotRegularFile { .. }),
        "expected the opened object to be refused as irregular, got {refusal:?}"
    );
    let _ = fs::remove_file(&outside);
}

#[cfg(windows)]
#[test]
fn a_root_that_is_not_locally_served_is_refused_rather_than_trusted() {
    // Containment on Windows is the *filesystem* refusing to rename a directory
    // that is held open. A redirector need not implement that, and one
    // measurably does not: with the root handle held on a `\\wsl.localhost\`
    // path, a swap staged from the Linux side succeeded and the child open
    // returned the decoy's identity.
    //
    // The wiring first, and this half is the one that matters. An earlier
    // version of this test exercised only the classifier below, and deleting
    // the call site in `RootDirectory::hold` left the entire suite green —
    // found by running the mutation rather than by reading. `hold` takes an
    // already-canonical path, so a literal UNC string reaches the precondition
    // without a network filesystem needing to exist on this machine.
    let refusal =
        super::RootDirectory::hold(PathBuf::from(r"\\?\UNC\wsl.localhost\Debian\tmp\generated"))
            .expect_err("a root that is not locally served must be refused");
    assert!(
        matches!(refusal, Refusal::RootNotLocallyServed { .. }),
        "expected the namespace precondition to refuse before the handle was taken, got \
         {refusal:?}"
    );

    // And the classifier itself, over paths in both directions.
    for local in [
        r"\\?\C:\Users\someone\PartMan\tests\generated",
        r"\\?\D:\PartMan\tests\generated",
    ] {
        assert!(
            super::root_namespace_is_local(Path::new(local)),
            "{local} is a local volume and must be permitted"
        );
    }
    for remote in [
        r"\\?\UNC\wsl.localhost\Debian\tmp\generated",
        r"\\?\UNC\server\share\generated",
        r"\\server\share\generated",
    ] {
        assert!(
            !super::root_namespace_is_local(Path::new(remote)),
            "{remote} is not locally served and must be refused"
        );
    }
    // A path `canonicalize` could not have produced has no prefix to classify,
    // and is refused rather than reasoned about.
    assert!(!super::root_namespace_is_local(Path::new(
        "tests/generated"
    )));
}

#[test]
fn a_new_hard_link_after_authorization_is_not_prevented() {
    // A boundary, recorded as a test so it cannot be quietly upgraded into a
    // guarantee by prose. This catches no mutation and is labelled as such.
    //
    // The link count is a snapshot taken at authorization. Nothing stops a new
    // name being added afterwards, on either platform. The reason that is
    // tolerable is an *argument*, not a measurement of impossibility: the
    // object bound at open cannot be renamed or deleted under its own name
    // while the handle lives, and its contents were verified, so any alias
    // created inside the window necessarily names bytes that were already
    // established as disposable fixture content. If either of those pins ever
    // weakens, this stops being harmless.
    let sandbox = Sandbox::new("post-auth-link");
    let target = sandbox.target("blank-512.img");
    let authorization = sandbox
        .authorize(&sandbox.request(vec![target.clone()]))
        .expect("the untouched fixture must authorize");

    let alias = sandbox_base().join(format!(
        "partman-late-alias-{}-{}.img",
        std::process::id(),
        crate::test_support::next_sandbox_id()
    ));
    let linked = fs::hard_link(&target, &alias);
    assert!(
        linked.is_ok(),
        "recorded honestly: an alias created after authorization is not prevented"
    );

    drop(authorization);
    let _ = fs::remove_file(&alias);
}

#[test]
fn the_windows_handle_wrapper_is_confined_to_one_call_site() {
    // `winapi_util::HandleRef` has no lifetime parameter and is constructible
    // from safe code, so a second call site could obtain an authoritative
    // answer for a closed handle — inside a crate whose manifest says
    // `unsafe_code = "deny"` and with no `unsafe` token anywhere. That was
    // demonstrated during review, which is why the wrapper is reached through
    // exactly one function.
    //
    // A textual gate, and this repository knows what those are worth: the
    // action-pin scanner was defeated three times as a text scanner before it
    // became a structural parse. It is used here because the property is about
    // *how many places name a symbol*, which is a textual property, and because
    // the alternative — trusting review — is what it replaces.
    // Assembled rather than written out, so this test's own source does not
    // contain the string it searches for. Comment lines are skipped for the
    // same reason: the prose above, and the doc comment on the function being
    // protected, both have to be free to *name* the thing they explain.
    let needle = concat!("winapi", "_util::");

    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut mentions = Vec::new();
    let mut pending = vec![source];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).expect("the crate's own source must be readable") {
            let path = entry.expect("readable directory entry").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("source must be readable");
            for (number, line) in text.lines().enumerate() {
                if !line.trim_start().starts_with("//") && line.contains(needle) {
                    mentions.push(format!("{}:{}", path.display(), number + 1));
                }
            }
        }
    }
    assert_eq!(
        mentions.len(),
        1,
        "the Windows handle wrapper must be reached from exactly one place in this crate, \
         found: {mentions:?}"
    );
}

#[test]
fn object_verification_alone_cannot_prove_root_membership() {
    // The 2026-07-29 follow-up audit's finding 2, established directly rather
    // than by argument. `verify_object` checks regular-file, link count,
    // length, and digest — all through the handle, all correct, and none of
    // them about *where* the object lives. A user's ordinary file may hold the
    // same bytes as a fixture, so content identity proves fixture shape and
    // says nothing about disposability or containment.
    //
    // This is why the pre-open path checks are load-bearing, and therefore why
    // the open itself must not be able to follow a symlink out of the root.
    // The regression test for that is
    // `a_symlink_swapped_in_before_open_is_refused`.
    let sandbox = Sandbox::new("outside-root-bytes");
    let entry = sandbox
        .manifest
        .entry("blank-512.img")
        .expect("the blank fixture is in the manifest")
        .clone();

    // An identical copy, deliberately outside the fixture root.
    let outside = std::env::temp_dir().join(format!(
        "partman-outside-{}-{}.img",
        std::process::id(),
        crate::test_support::next_sandbox_id()
    ));
    let bytes = fs::read(sandbox.target("blank-512.img")).expect("read the fixture");
    fs::write(&outside, &bytes).expect("write an identical file outside the root");

    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&outside)
        .expect("open the outside file");
    let verdict = super::verify_object(&mut file, entry.length, &entry.digest, &outside);
    drop(file);
    let _ = fs::remove_file(&outside);

    assert!(
        verdict.is_ok(),
        "recorded honestly: the object checks accept an out-of-root file with fixture bytes, \
         so containment cannot rest on them"
    );
}

#[test]
fn object_verification_survives_the_path_ceasing_to_exist() {
    // The deterministic proof that `verify_object` is handle-pure. A deletion
    // sweep on the first version showed that downgrading its `fstat` to a
    // by-path `stat` kept every test green — the difference only shows during
    // a race, which a unit test cannot stage reliably. So the seam is tested
    // the other way around: open the object, make the path *gone*, and then
    // verify. Any check that touches a path instead of the handle now fails
    // on a missing file, so this test passing proves every check reads what
    // is held.
    let sandbox = Sandbox::new("handle-purity");
    let target = sandbox.target("blank-512.img");
    let entry = sandbox
        .manifest
        .entry("blank-512.img")
        .expect("the blank fixture is in the manifest")
        .clone();

    let mut options = fs::OpenOptions::new();
    options.read(true).write(true);
    #[cfg(windows)]
    {
        // Permissive sharing (read | write | delete), unlike the interlock's
        // own restrictive mode: this handle must allow the rename below.
        use std::os::windows::fs::OpenOptionsExt as _;
        options.share_mode(7);
    }
    let mut file = options.open(&target).expect("open the fixture");

    // Make the name useless. On Unix the file is unlinked outright; Windows
    // refuses deletion of an open file even with FILE_SHARE_DELETE on some
    // filesystems, so the name is renamed away instead — equally gone from
    // where any by-path check would look.
    #[cfg(unix)]
    fs::remove_file(&target).expect("unlink the verified name");
    #[cfg(windows)]
    fs::rename(&target, sandbox.root.join("elsewhere.img")).expect("rename the name away");
    assert!(!target.exists(), "the verified path must be gone");

    super::verify_object(&mut file, entry.length, &entry.digest, &target)
        .expect("verification must succeed through the handle alone");
}

#[test]
fn the_verified_handle_is_what_the_consumer_receives() {
    // `into_targets` consumes the authorization, and each target yields the
    // open file itself. There is no path-reopening step for a race to hide in:
    // handing over the handle *is* the handover.
    let sandbox = Sandbox::new("handle-handover");
    let target = sandbox.target("blank-512.img");
    let authorization = sandbox
        .authorize(&sandbox.request(vec![target]))
        .expect("the fixture must authorize");

    let expected_length = sandbox
        .manifest
        .entry("blank-512.img")
        .expect("the blank fixture is in the manifest")
        .length;
    let targets = authorization.into_targets();
    assert_eq!(targets.len(), 1);
    let metadata = targets[0]
        .file
        .metadata()
        .expect("fstat on the verified handle");
    assert!(metadata.is_file());
    assert_eq!(
        metadata.len(),
        expected_length,
        "the handle is open on the exact object whose length the manifest records"
    );
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

#[test]
fn a_hard_link_to_a_file_outside_the_root_is_refused() {
    // F-03, and the reason this test is not `#[cfg(unix)]` any more: until
    // increment 2d the link count was read only on Unix, and on Windows this
    // exact arrangement **authorized**. Reproduced end to end during review —
    // a stand-in user file was destroyed through the authorized handle while
    // the link count read 2 the whole time.
    //
    // The arrangement matters. A victim outside the fixture root holds the
    // fixture's exact bytes — an ordinary thing for a user to have, since the
    // images are a deterministic function of public source — and a hard link
    // to it occupies the fixture's own name. Name, location, regular-file
    // status, length and digest therefore *all* pass. The link count is the
    // only check that can refuse, which is what makes this test discriminating
    // rather than merely red.
    let sandbox = Sandbox::new("hard-link-outside");
    let target = sandbox.target("blank-512.img");
    let bytes = fs::read(&target).expect("read the fixture");

    let victim = sandbox_base().join(format!(
        "partman-victim-{}-{}.dat",
        std::process::id(),
        crate::test_support::next_sandbox_id()
    ));
    fs::write(&victim, &bytes).expect("a user file that happens to hold fixture bytes");

    fs::remove_file(&target).expect("free the fixture's name");
    fs::hard_link(&victim, &target).expect("link the victim in at the fixture's name");

    let refusal = sandbox
        .authorize(&sandbox.request(vec![target.clone()]))
        .expect_err("a target that is also reachable outside the root must be refused");

    // Exact, with the count. The disjunction this test used to carry
    // (`TargetHasOtherNames | TargetNotGenerated`) accepted the digest refusal
    // that fires when the check is deleted, so it survived removal of the only
    // thing it existed to protect.
    match refusal {
        Refusal::TargetHasOtherNames { links, .. } => assert_eq!(
            links, 2,
            "the refusal must report the count it actually observed"
        ),
        other => panic!("expected TargetHasOtherNames, got {other:?}"),
    }

    // Control: the same object, same bytes, same name — with the outside name
    // gone. If this did not authorize, the test above would be passing for some
    // reason other than the link count.
    fs::remove_file(&victim).expect("drop the outside name");
    let authorization = sandbox
        .authorize(&sandbox.request(vec![target]))
        .expect("with one name left, the very same object must authorize");
    drop(authorization);
}

#[test]
fn a_hard_link_between_two_fixture_names_is_refused() {
    // The narrower in-root case the previous version of this test covered.
    // Kept because it is the shape a careless `cp -l` produces, and separated
    // from the one above because here the digest *also* disagrees, so it cannot
    // discriminate the link check on its own.
    let sandbox = Sandbox::new("hard-link-inside");
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
