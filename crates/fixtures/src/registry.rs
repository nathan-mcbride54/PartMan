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
//! **The shipped registry is empty, and that emptiness is pinned by test.**
//! Registering the first real suite is increment 2h's delivery, behind its
//! own recorded boundary and an operator-accepted VM sitting. It is also the
//! edit that changes the meaning of every generic-refusal test — from "the
//! concept does not exist" to "the registry holds no suite" to "the registry
//! holds a suite this request did not select" — so each such test must be
//! re-read at that edit, not merely re-run. No executor exists in this
//! increment: nothing consumes an [`Admission`], and every generic
//! destructive request continues to refuse.

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

/// Every destructive suite compiled into this build.
///
/// **Deliberately empty.** The first entry is increment 2h's delivery behind
/// its own recorded boundary, and `the_shipped_registry_is_empty` pins this
/// length so that entry is a visible reviewed edit which re-opens every
/// generic-refusal test for re-reading.
const REGISTERED: [Suite; 0] = [];

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
/// Nothing in this increment consumes an admission. The type exists so the
/// future executor's signature can require it, the same way the loop harness
/// requires an `Authorization` today.
#[derive(Debug)]
pub struct Admission {
    suite: &'static Suite,
    targets: Vec<VerifiedTarget>,
}

impl Admission {
    /// Check `suite`'s contract against the compiled catalogue and consume
    /// `authorization` if — and only if — its verified target set is exactly
    /// the suite's declared fixture set.
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

        // The authorized set must be exactly the declared set. Verified
        // target basenames are catalogue basenames by construction — the
        // interlock refused anything else — so naming them here echoes
        // compiled constants, never operator input.
        let authorized: std::collections::BTreeSet<String> = authorization
            .targets()
            .iter()
            .map(|target| {
                target
                    .path()
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default()
            })
            .collect();
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
