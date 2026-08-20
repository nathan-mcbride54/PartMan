//! ADR-0021's authorization ladder (increment 3): the tier the helper
//! computes for itself, the floor act it can mint alone, and the
//! interactive ceremony it cannot perform on any route this build ships.
//!
//! **The two tiers, and where each comes from.** HLP-003 requires a
//! **floor** act for every apply at every severity — fresh, explicit, by
//! the RPC-001-authenticated user, naming the exact plan hash, single-use,
//! inside the PLAN-007 window, journaled, never cached — and *additionally*
//! an interactive **ceremony** at severity ≥ Disruptive or on a plan
//! carrying any step flag. The tier is "a total function of the plan
//! body's severity and flags" and derives "from the helper's own
//! recomputed severity and flags under HLP-002, never from client-claimed
//! values" (`AGENT_BUILD_SPEC.md`, HLP-003; ADR-0021 resolving SI-18).
//!
//! **Three properties are structural here, not checked:**
//!
//! 1. **No client can name a tier.** [`required_tier`] reads only an
//!    [`crate::validate::AdmittedPlan`], whose only constructor is
//!    [`crate::validate::admit_presented_plan`] past SEC-002's arms — so a
//!    forged body, a replayed act, a cross-user presentation or an expired
//!    plan cannot reach the computation at all (CAP-007's
//!    unrepresentability, ADR-0012's shape).
//! 2. **A sixth step flag escalates by default.** The flags half of the
//!    rule compares against `StepFlags::default()` rather than enumerating
//!    the five named flags, so a flag added to PLAN-004 later takes the
//!    ceremony without an edit here — ADR-0021's own revisit condition,
//!    which an OR-chain over named flags would silently break.
//! 3. **The ceremony cannot be faked.** [`CeremonyCompleted`] has a
//!    private field and no constructor outside `#[cfg(test)]`; the only
//!    [`Ceremony`] linked into a shipped build is [`RefusingCeremony`].
//!    A patch that deletes the seam call does not compile, and one that
//!    makes the shipped implementation succeed cannot be written without
//!    a constructor that does not exist.
//!
//! **What this build does not do, decided and recorded.** The apply
//! ceremony's route — `pkcheck` through a SAFE-004 launcher, or polkit's
//! D-Bus authority — is **undecided**, and the decision owner took R8 on
//! `docs/reviews/LINUX_APPLY_CEREMONY_ROUND_2026-08-19.md`: the ceremony
//! ships as a seam that refuses, because the record contains no
//! measurement of any client `auth_admin` ever succeeding while it does
//! contain the fail-open's exact shape (a root subject is authorized with
//! no agent and no prompt, DR23). So every interactive-tier plan is
//! refused [`CeremonyUnavailable::NoInteractiveRoute`] — **one arm** for
//! "no route decided" and for "polkit absent from this host" alike, so
//! the refusal is not a channel for probing the host (SAFE-005 disables
//! the operation; it does not itemise why).
//!
//! Two constraints the round bound on every future route, recorded where
//! the implementer will stand: a shipped ceremony action declares
//! `auth_admin` in **all three** implicit values and never a `*_keep`
//! variant, and no runtime call passes a keep-implying flag. That is
//! "without retained grants" made structural.
//!
//! **The apply itself is increment 4's.** This module is the gate; nothing
//! in this build reaches it over the wire, because `apply-plan` is
//! answered `not-yet-served` naming increment 4. The gate is proven at
//! Tier 1 over authored inputs, which is what the evidence rule asks of a
//! structural property.

use partman_domain::model::step::{Severity, StepFlags};
use partman_journal::records::{AuthorizationAct, AuthorizationTier, PlanHashRef};

use crate::validate::AdmittedPlan;

/// PLAN-004's severity-plus-flags rule, as a total function.
///
/// `≥ Disruptive` **or any flag at all** takes the ceremony; everything
/// else takes the floor act. The flags half is `*flags !=
/// StepFlags::default()` deliberately — see this module's property 2.
#[must_use]
pub fn required_tier(severity: Severity, flags: &StepFlags) -> AuthorizationTier {
    if severity >= Severity::Disruptive || *flags != StepFlags::default() {
        AuthorizationTier::InteractiveCeremony
    } else {
        AuthorizationTier::FloorAct
    }
}

/// Proof that a human completed the platform's interactive ceremony for
/// one exact plan hash.
///
/// Deliberately unconstructible outside this crate's tests: a private
/// field, no `Clone`, no `Copy`, no `Default`, no public constructor. In
/// a shipped build **no value of this type exists**, which is what makes
/// the ceremony's absence a property of the types rather than of a
/// branch a patch could delete.
pub struct CeremonyCompleted {
    _seal: (),
}

impl CeremonyCompleted {
    /// The test-only constructor. `pub(crate)` and `cfg(test)`: it cannot
    /// be reached from a shipped build or from another crate.
    #[cfg(test)]
    pub(crate) const fn for_test() -> Self {
        Self { _seal: () }
    }
}

/// Why no interactive authorization happened.
///
/// **One variant on purpose.** "This build ships no route" and "this host
/// has no polkit" are the same answer to a client, because a refusal that
/// distinguished them would report the host's configuration to an
/// unprivileged caller. SAFE-005 disables the affected operation; it does
/// not owe an inventory of why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CeremonyUnavailable {
    /// No interactive authorization route is delivered on this build.
    NoInteractiveRoute,
}

impl core::fmt::Display for CeremonyUnavailable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoInteractiveRoute => write!(
                f,
                "this plan requires a fresh interactive authorization and no \
                 interactive authorization route is available; it cannot be \
                 applied on this build"
            ),
        }
    }
}

/// The platform's interactive ceremony (HLP-003's second tier).
pub trait Ceremony {
    /// Put **this plan hash** in front of **this client** and report
    /// whether the platform's administrator authorized it.
    ///
    /// The plan hash is a parameter because HLP-003 binds the ceremony to
    /// the exact plan, and the client uid because the ceremony's subject
    /// is the *client*, never the helper: DR23 measured that polkit
    /// authorizes a root subject for `auth_admin` with no agent and no
    /// prompt, so an implementation that passed itself would authorize
    /// everything and prove nothing.
    ///
    /// # Errors
    ///
    /// [`CeremonyUnavailable`] when no ceremony could be performed.
    fn perform(
        &self,
        plan: PlanHashRef,
        client_uid: u32,
    ) -> Result<CeremonyCompleted, CeremonyUnavailable>;
}

/// The only [`Ceremony`] linked into a shipped build: it refuses.
///
/// Not a stub standing in for work in progress — the round's decision.
/// It cannot be made to succeed without a [`CeremonyCompleted`], which
/// has no constructor outside this crate's tests.
#[derive(Clone, Copy, Debug, Default)]
pub struct RefusingCeremony;

impl Ceremony for RefusingCeremony {
    fn perform(
        &self,
        _plan: PlanHashRef,
        _client_uid: u32,
    ) -> Result<CeremonyCompleted, CeremonyUnavailable> {
        Err(CeremonyUnavailable::NoInteractiveRoute)
    }
}

/// An authorization the ladder granted: the journal act increment 4 will
/// append and consume through `admit_apply`.
///
/// No public constructor. An `AuthorizedApply` exists only because
/// [`authorize`] produced it, which means the tier was recomputed from an
/// [`AdmittedPlan`] and — at the interactive tier — a [`CeremonyCompleted`]
/// was moved into it.
#[derive(Debug)]
pub struct AuthorizedApply {
    act: AuthorizationAct,
}

impl AuthorizedApply {
    /// The act, ready for the journal (ADR-0028's one act, one apply).
    #[must_use]
    pub const fn act(&self) -> AuthorizationAct {
        self.act
    }
}

/// Why the ladder refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthorizationRefusal {
    /// The plan's recomputed tier is the interactive ceremony, and none
    /// is available.
    CeremonyUnavailable(CeremonyUnavailable),
}

impl core::fmt::Display for AuthorizationRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CeremonyUnavailable(cause) => write!(f, "{cause}"),
        }
    }
}

/// Climb the ladder for one admitted plan.
///
/// The tier is recomputed here from the admitted plan's own severity and
/// flags; nothing the client sent participates. At the floor tier the act
/// is minted with **no seam call, no agent and no terminal** — ADR-0021's
/// programmatic act, which is what keeps SAFE-003's unattended-apply
/// population representable. At the interactive tier the ceremony's
/// [`CeremonyCompleted`] is **moved** into the mint, so one completion
/// mints exactly one act.
///
/// The act names the helper's **own recomputed** plan hash, taken from the
/// admitted plan — never a client-supplied identifier.
///
/// # Errors
///
/// [`AuthorizationRefusal`].
pub fn authorize(
    admitted: &AdmittedPlan,
    client_uid: u32,
    ceremony: &dyn Ceremony,
) -> Result<AuthorizedApply, AuthorizationRefusal> {
    let plan = admitted.plan_hash_ref();
    let tier = required_tier(admitted.severity(), &admitted.flags());
    match tier {
        AuthorizationTier::FloorAct => Ok(AuthorizedApply {
            act: AuthorizationAct::new(plan, AuthorizationTier::FloorAct),
        }),
        AuthorizationTier::InteractiveCeremony => {
            let completed = ceremony
                .perform(plan, client_uid)
                .map_err(AuthorizationRefusal::CeremonyUnavailable)?;
            // Moved, not borrowed: `CeremonyCompleted` is neither `Copy`
            // nor `Clone`, so this value cannot mint a second act.
            let AuthorizeSeal { .. } = AuthorizeSeal { completed };
            Ok(AuthorizedApply {
                act: AuthorizationAct::new(plan, AuthorizationTier::InteractiveCeremony),
            })
        }
    }
}

/// Consumes the completion proof at the point of minting. A private
/// carrier so the move is visible in the code rather than implied by a
/// dropped binding.
struct AuthorizeSeal {
    #[allow(dead_code)]
    completed: CeremonyCompleted,
}
