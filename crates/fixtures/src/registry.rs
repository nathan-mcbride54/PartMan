//! The destructive-suite registry: WP-020 increment 2g.
//!
//! Until this module existed, "no destructive suite is registered" was
//! load-bearing prose backed by refusal tests. This gives the sentence a
//! compiled type. A destructive suite is now a value of [`Suite`]: it names
//! its fixture set by catalogue basename, its verified target class, its
//! per-fixture intended-change contract — the exact byte ranges a run may
//! change, everything outside them pinned by digest bracket — and the
//! teardown facts its acceptance record must establish.
//!
//! Two design rules carry over from the interlock, because they are the same
//! rules.
//!
//! **The registry is compiled, never read.** Like the catalogue the interlock
//! trusts, [`registered`] returns values compiled into the binary. Nothing is
//! read from the fixture root or any other file, so whoever writes a file
//! chooses nothing about what a destructive suite may address.
//!
//! **Admission is a proof, not a lookup.** [`Admission::admit`] consumes the
//! interlock's live [`Authorization`] — the handle-holding proof, not a path
//! list — and checks the suite's contract against the compiled catalogue and
//! the authorized target set. A function that requires an [`Admission`]
//! cannot run without every SAFE-007 factor *and* a well-formed compiled
//! contract, and "did anyone check?" stays answered by the type system.
//!
//! **The shipped registry holds exactly one suite, and its whole shape is
//! pinned by test.** It was empty through increment 2g; increment 2h
//! registered `gpt-basic-512-signature-erase`, which was the single edit 2g
//! reserved. That edit changed the meaning of every generic-refusal test —
//! from "the concept does not exist" to "the registry holds no suite" to "the
//! registry holds a suite this request did not select" — so each was re-read
//! at it, with the re-readings recorded in the tests that changed. A second
//! entry needs its own recorded boundary and re-opens them again.
//!
//! An [`Admission`] is consumed by exactly one executor,
//! `partman_ffi_linux_loop::run_destructive_suite`, which additionally
//! requires the suite to be one [`registered`] returns — this module
//! deliberately does not, so its own refusal semantics stay testable with a
//! suite that is not registered. A *generic* destructive request still
//! selects no suite and still refuses.

use core::fmt;

use crate::interlock::{Authorization, VerifiedTarget};

/// One byte range a suite's contract permits a run to change.
///
/// The range is stated with its reason so a reviewer reads *why* those bytes
/// may move, not merely that they may. Everything outside every declared
/// range is pinned by digest bracket: unchanged before/after digests over the
/// undeclared remainder are a teardown obligation, not a courtesy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntendedChange {
    /// First byte the run may change, from the start of the fixture.
    pub offset: u64,
    /// Number of bytes, starting at `offset`, the run may change.
    pub length: u64,
    /// The exact bytes the range becomes. Added by increment 2h so "changed
    /// exactly as contracted" is byte-checkable rather than narrative: the
    /// executor writes these bytes and nothing else, and the post-run check
    /// requires the range to equal them. Admission refuses a replacement
    /// whose length differs from `length`.
    pub replacement: &'static [u8],
    /// Why exactly these bytes: the reviewed sentence a reader gets.
    pub reason: &'static str,
}

/// The class of object a suite's targets must have been verified as.
///
/// A closed vocabulary, pinned by exhaustive match in tests: adding a variant
/// is a visible reviewed edit. There is exactly one class today because the
/// interlock can verify exactly one kind of object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetClass {
    /// A regular file in the generated-fixture root whose name, length, and
    /// bytes the interlock verified against the compiled catalogue, held as
    /// an open handle from verification to use.
    GeneratedFixtureFile,
}

/// One teardown fact a suite's acceptance record must establish before any
/// run under it may be described as passed.
///
/// A closed vocabulary, pinned by exhaustive match in tests. These are proof
/// obligations on the *record*, not steps the registry performs: the registry
/// runs nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeardownObligation {
    /// The declared ranges differ exactly as contracted — no more, no less.
    ChangedExactlyAsContracted,
    /// Every sampled range outside the contract has an unchanged digest.
    UnchangedOutsideContract,
    /// Descriptor-bound detach confirmed under 2e's discipline.
    DetachConfirmed,
    /// The backing file regenerated and re-digested against the catalogue.
    BackingRegeneratedToCatalogue,
}

/// One fixture a suite addresses, and what it is allowed to do to it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixtureContract {
    /// The catalogue basename. Admission refuses a name the catalogue does
    /// not generate, so a contract cannot quietly outlive its fixture.
    pub fixture: &'static str,
    /// The byte ranges a run may change. Admission refuses an empty list —
    /// a suite whose contract permits no change is not a destructive suite,
    /// and carrying the destructive profile for it would be the vacuous-pass
    /// shape this package exists to refuse.
    pub may_change: &'static [IntendedChange],
}

/// One compiled destructive suite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Suite {
    /// The exact selector name an operator must type. Nothing runs a suite
    /// the operator did not name; a generic tier request selects nothing.
    pub name: &'static str,
    /// What its targets must have been verified as.
    pub target_class: TargetClass,
    /// Every fixture it addresses, each with its intended-change contract.
    pub fixtures: &'static [FixtureContract],
    /// The teardown facts its acceptance record must establish.
    pub teardown: &'static [TeardownObligation],
}

/// The selector name of the one registered destructive suite.
pub const GPT_BASIC_512_SIGNATURE_ERASE: &str = "gpt-basic-512-signature-erase";

/// The replacement bytes the signature-erase contract writes: eight zeros,
/// erasing exactly the bytes that make the primary header claim to be one.
const SIGNATURE_ERASE_REPLACEMENT: [u8; 8] = [0; 8];

static SIGNATURE_ERASE_RANGES: [IntendedChange; 1] = [IntendedChange {
    offset: 512,
    length: 8,
    replacement: &SIGNATURE_ERASE_REPLACEMENT,
    reason: "the primary GPT header's signature field at LBA 1 for 512-byte sectors, \
             replaced with eight zero bytes — the smallest honest mutation, erasing \
             exactly the bytes that make the header claim to be one",
}];

static SIGNATURE_ERASE_FIXTURES: [FixtureContract; 1] = [FixtureContract {
    fixture: "gpt-basic-512.img",
    may_change: &SIGNATURE_ERASE_RANGES,
}];

/// Every destructive suite compiled into this build.
///
/// **Exactly one, registered by increment 2h behind its own recorded
/// boundary.** This is the single edit increment 2g's boundary reserved: the
/// 2g emptiness pin became `the_shipped_registry_holds_exactly_the_2h_suite`,
/// the xtask refusal-count pin moved from zero to one in the same change, and
/// every generic-refusal test was re-read at this edit, with the re-readings
/// recorded on the registering pull request. A second entry is a new reviewed
/// edit under a new recorded boundary, and it re-opens those tests again.
const REGISTERED: [Suite; 1] = [Suite {
    name: GPT_BASIC_512_SIGNATURE_ERASE,
    target_class: TargetClass::GeneratedFixtureFile,
    fixtures: &SIGNATURE_ERASE_FIXTURES,
    teardown: &[
        TeardownObligation::ChangedExactlyAsContracted,
        TeardownObligation::UnchangedOutsideContract,
        TeardownObligation::DetachConfirmed,
        TeardownObligation::BackingRegeneratedToCatalogue,
    ],
}];

/// The compiled registry, in declaration order.
#[must_use]
pub fn registered() -> &'static [Suite] {
    &REGISTERED
}

/// Proof that one compiled suite's contract admits one authorized target set.
///
/// Constructible only by [`Admission::admit`], which consumes the
/// [`Authorization`]: one admission is one gated run, exactly as one
/// authorization is. Deliberately **not** `Clone`, and the targets leave only
/// by [`Admission::into_targets`]:
///
/// ```compile_fail
/// # use partman_fixtures::registry::Admission;
/// fn replay(admission: &Admission) -> Admission {
///     admission.clone()
/// }
/// ```
///
/// Holding one does **not** mean the suite may run: the executor separately
/// requires the suite to be one [`registered`] returns, because `Suite` is a
/// public type with public fields and this constructor accepts any
/// `&'static Suite` so that the refusal semantics above stay testable with a
/// suite that was never registered.
#[derive(Debug)]
pub struct Admission {
    suite: &'static Suite,
    targets: Vec<VerifiedTarget>,
}

impl Admission {
    /// Check `suite`'s contract against the compiled catalogue and consume
    /// `authorization` if — and only if — it carries exactly one verified
    /// handle for each declared fixture and nothing else. Counted, not
    /// compared as a set: several handles for one fixture are a refusal, not
    /// an admission.
    ///
    /// # Errors
    ///
    /// Returns [`Refusal`] for a malformed contract or a target set that is
    /// not exactly the declared fixture set. Every failure is a refusal, and
    /// the consumed authorization is dropped with it: an authorization that
    /// reached a refusing admission is spent, not recyclable.
    pub fn admit(suite: &'static Suite, authorization: Authorization) -> Result<Self, Refusal> {
        // Contract checks run against the compiled catalogue, never a file —
        // the same root of trust the interlock uses.
        let manifest = crate::catalogue::expected();

        if suite.fixtures.is_empty() {
            return Err(Refusal::SuiteAddressesNothing { suite: suite.name });
        }

        let mut declared = std::collections::BTreeSet::new();
        for contract in suite.fixtures {
            if !declared.insert(contract.fixture) {
                return Err(Refusal::DuplicateFixtureContract {
                    suite: suite.name,
                    fixture: contract.fixture,
                });
            }
            let entry = manifest
                .entry(contract.fixture)
                .ok_or(Refusal::NotAGeneratedFixture {
                    suite: suite.name,
                    fixture: contract.fixture,
                })?;
            verify_contract(suite.name, contract, entry.length)?;
        }

        // The authorized targets must be exactly one verified handle per
        // declared fixture. A set comparison was not enough (issue #249): a
        // set of names cannot distinguish one handle per fixture from several
        // handles for one fixture, and the interlock's dedup is on the
        // *supplied* path, so two spellings of one file can both verify. The
        // suite contract is a statement about objects, and this admission is
        // what binds it to the handles a run writes through — so the handles
        // are counted here rather than inherited from what the interlock
        // probably produced. Verified target basenames are catalogue
        // basenames by construction — the interlock refused anything else —
        // so naming them here echoes compiled constants, never operator
        // input. (A lossy rendering that collided two distinct names would
        // read as a duplicate and refuse, which is the closed direction.)
        let mut authorized_counts: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        for target in authorization.targets() {
            let Some(name) = target.path().file_name() else {
                // Unreachable through `authorize` today — every verified
                // path is `root.join(catalogue_basename)` — but the silent
                // alternative was coercing such a target to the empty string
                // and comparing that, which is a default on the safety path.
                return Err(Refusal::TargetNameUnstatable {
                    suite: suite.name,
                    target: target.path().to_path_buf(),
                });
            };
            *authorized_counts
                .entry(name.to_string_lossy().into_owned())
                .or_default() += 1;
        }
        if let Some((name, count)) = authorized_counts.iter().find(|(_, count)| **count > 1) {
            return Err(Refusal::DuplicateAuthorizedTarget {
                suite: suite.name,
                target: name.clone(),
                count: *count,
            });
        }
        let authorized: std::collections::BTreeSet<String> =
            authorized_counts.into_keys().collect();
        let declared_names: std::collections::BTreeSet<String> =
            declared.iter().map(|name| (*name).to_owned()).collect();
        if authorized != declared_names {
            let missing = declared_names
                .difference(&authorized)
                .cloned()
                .collect::<Vec<_>>();
            let extra = authorized
                .difference(&declared_names)
                .cloned()
                .collect::<Vec<_>>();
            return Err(Refusal::TargetsNotTheDeclaredSet {
                suite: suite.name,
                missing,
                extra,
            });
        }

        Ok(Self {
            suite,
            targets: authorization.into_targets(),
        })
    }

    /// The admitted suite.
    #[must_use]
    pub fn suite(&self) -> &'static Suite {
        self.suite
    }

    /// Consume the proof, yielding the verified objects for one gated run.
    /// There is intentionally no way to keep the admission while taking them.
    #[must_use]
    pub fn into_targets(self) -> Vec<VerifiedTarget> {
        self.targets
    }
}

/// Check one fixture contract's ranges against the generated length.
fn verify_contract(
    suite: &'static str,
    contract: &FixtureContract,
    generated_length: u64,
) -> Result<(), Refusal> {
    if contract.may_change.is_empty() {
        return Err(Refusal::ContractPermitsNoChange {
            suite,
            fixture: contract.fixture,
        });
    }
    for range in contract.may_change {
        if range.length == 0 {
            return Err(Refusal::EmptyRange {
                suite,
                fixture: contract.fixture,
                offset: range.offset,
            });
        }
        if !u64::try_from(range.replacement.len()).is_ok_and(|len| len == range.length) {
            return Err(Refusal::ReplacementLengthMismatch {
                suite,
                fixture: contract.fixture,
                offset: range.offset,
            });
        }
        let end = range.offset.checked_add(range.length);
        if end.is_none_or(|end| end > generated_length) {
            return Err(Refusal::RangeOutOfBounds {
                suite,
                fixture: contract.fixture,
                offset: range.offset,
                length: range.length,
                generated_length,
            });
        }
    }
    // Overlapping declared ranges make "everything outside the contract" an
    // ambiguous set — two reasons claiming one byte is a contract nobody can
    // review. Touching ranges are fine; sharing a byte is refused.
    let mut ranges: Vec<(u64, u64)> = contract
        .may_change
        .iter()
        .map(|range| (range.offset, range.length))
        .collect();
    ranges.sort_unstable();
    for pair in ranges.windows(2) {
        if pair[0].0 + pair[0].1 > pair[1].0 {
            return Err(Refusal::RangesOverlap {
                suite,
                fixture: contract.fixture,
            });
        }
    }
    Ok(())
}

/// Why an admission was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The suite declares no fixtures. "Every target was verified" is
    /// vacuously true of an empty set, and a destructive suite over nothing
    /// has no business existing.
    SuiteAddressesNothing {
        /// The suite's selector name.
        suite: &'static str,
    },
    /// The suite declares the same fixture twice, making its contract
    /// ambiguous.
    DuplicateFixtureContract {
        /// The suite's selector name.
        suite: &'static str,
        /// The fixture declared more than once.
        fixture: &'static str,
    },
    /// The suite declares a fixture the compiled catalogue does not generate.
    NotAGeneratedFixture {
        /// The suite's selector name.
        suite: &'static str,
        /// The undeclared name.
        fixture: &'static str,
    },
    /// A contract permits no change at all.
    ContractPermitsNoChange {
        /// The suite's selector name.
        suite: &'static str,
        /// The fixture whose contract is vacuous.
        fixture: &'static str,
    },
    /// A declared range's replacement bytes do not match its length, so
    /// "changed exactly as contracted" would be unstatable.
    ReplacementLengthMismatch {
        /// The suite's selector name.
        suite: &'static str,
        /// The fixture whose contract carries the range.
        fixture: &'static str,
        /// The range's offset.
        offset: u64,
    },
    /// A declared range has zero length.
    EmptyRange {
        /// The suite's selector name.
        suite: &'static str,
        /// The fixture whose contract carries the range.
        fixture: &'static str,
        /// The range's offset.
        offset: u64,
    },
    /// A declared range extends past the fixture's generated length.
    RangeOutOfBounds {
        /// The suite's selector name.
        suite: &'static str,
        /// The fixture whose contract carries the range.
        fixture: &'static str,
        /// The range's offset.
        offset: u64,
        /// The range's length.
        length: u64,
        /// The fixture's generated length.
        generated_length: u64,
    },
    /// Two declared ranges share at least one byte.
    RangesOverlap {
        /// The suite's selector name.
        suite: &'static str,
        /// The fixture whose contract overlaps itself.
        fixture: &'static str,
    },
    /// A verified target's path has no final component, so the fixture it
    /// holds a handle to cannot be named. Unreachable through `authorize`
    /// today; refused rather than coerced to a default.
    TargetNameUnstatable {
        /// The suite's selector name.
        suite: &'static str,
        /// The verified path that has no file name.
        target: std::path::PathBuf,
    },
    /// The authorization carries more than one verified handle for one
    /// target name. The contract permits one object per fixture; a second
    /// handle to the same object is not a second fixture.
    DuplicateAuthorizedTarget {
        /// The suite's selector name.
        suite: &'static str,
        /// The duplicated target name.
        target: String,
        /// How many verified handles carry it.
        count: usize,
    },
    /// The authorized target set is not exactly the declared fixture set.
    TargetsNotTheDeclaredSet {
        /// The suite's selector name.
        suite: &'static str,
        /// Declared fixtures the authorization does not carry.
        missing: Vec<String>,
        /// Authorized targets the suite does not declare.
        extra: Vec<String>,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SuiteAddressesNothing { suite } => write!(
                formatter,
                "suite `{suite}` declares no fixtures; a destructive suite over nothing must \
                 not exist"
            ),
            Self::DuplicateFixtureContract { suite, fixture } => write!(
                formatter,
                "suite `{suite}` declares `{fixture}` more than once, so its contract is \
                 ambiguous"
            ),
            Self::NotAGeneratedFixture { suite, fixture } => write!(
                formatter,
                "suite `{suite}` declares `{fixture}`, which the compiled catalogue does not \
                 generate"
            ),
            Self::ContractPermitsNoChange { suite, fixture } => write!(
                formatter,
                "suite `{suite}`'s contract for `{fixture}` permits no change; a suite that \
                 changes nothing must not carry the destructive profile"
            ),
            Self::ReplacementLengthMismatch {
                suite,
                fixture,
                offset,
            } => write!(
                formatter,
                "suite `{suite}`'s contract for `{fixture}` declares a range at offset \
                 {offset} whose replacement byte count differs from its length; the change \
                 it contracts cannot be stated"
            ),
            Self::EmptyRange {
                suite,
                fixture,
                offset,
            } => write!(
                formatter,
                "suite `{suite}`'s contract for `{fixture}` declares a zero-length range at \
                 offset {offset}"
            ),
            Self::RangeOutOfBounds {
                suite,
                fixture,
                offset,
                length,
                generated_length,
            } => write!(
                formatter,
                "suite `{suite}`'s contract for `{fixture}` declares {length} byte(s) at \
                 offset {offset}, past the generated length {generated_length}"
            ),
            Self::RangesOverlap { suite, fixture } => write!(
                formatter,
                "suite `{suite}`'s contract for `{fixture}` declares overlapping ranges; two \
                 reasons claiming one byte is a contract nobody can review"
            ),
            Self::TargetNameUnstatable { suite, target } => write!(
                formatter,
                "suite `{suite}`'s authorization carries a verified target whose path has no \
                 file name: {}",
                target.display()
            ),
            Self::DuplicateAuthorizedTarget {
                suite,
                target,
                count,
            } => write!(
                formatter,
                "suite `{suite}`'s authorization carries {count} verified handles named \
                 `{target}`; the contract permits exactly one object per declared fixture"
            ),
            Self::TargetsNotTheDeclaredSet {
                suite,
                missing,
                extra,
            } => write!(
                formatter,
                "suite `{suite}`'s authorized targets are not its declared fixture set: \
                 missing [{}], extra [{}]",
                missing.join(", "),
                extra.join(", ")
            ),
        }
    }
}

impl std::error::Error for Refusal {}

#[cfg(test)]
mod tests;
