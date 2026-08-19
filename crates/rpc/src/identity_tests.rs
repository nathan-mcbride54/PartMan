//! Tests for the helper-authentication skeleton (WP-040 increment 4).

use super::identity::IdentityClaim;

// Requirements: RPC-001
//   The claim vocabulary is closed: exactly one identity claim per
//   RPC-001 transport — the Windows pipe's SDDL restriction, the Unix
//   socket's peer credentials, the macOS code-signing requirement —
//   pinned by an exhaustive match so widening it fails the suite as a
//   visible reviewed edit. Every claim names what a peer proves about
//   its identity and names its transport's route decision for a
//   verifier: verified by nobody here — the Windows and macOS decisions
//   unrecorded, the Linux one recorded (ADR-0055) and naming the crate
//   that carries its verifier — and no authorization vocabulary exists
//   to close over (SI-18 resolved by ADR-0021; the skeleton still says
//   nothing about what a peer may do).
// Evidence: the_claim_vocabulary_is_closed_and_verifier_free
#[test]
fn the_claim_vocabulary_is_closed_and_verifier_free() {
    // Closure by construction: a new variant fails this match — and
    // with it the suite — before any prose can drift.
    for claim in IdentityClaim::ALL {
        match claim {
            IdentityClaim::WindowsPipeSddl
            | IdentityClaim::UnixPeerCredentials
            | IdentityClaim::MacosCodeSigning => {}
        }
    }

    // One claim per RPC-001 transport, in the requirement's order,
    // pinned as literals.
    assert_eq!(
        IdentityClaim::ALL.map(IdentityClaim::transport),
        [
            "windows named pipe",
            "linux unix domain socket",
            "macos xpc or equivalently verified socket",
        ]
    );

    // Verified by nobody here: every claim's verifier arrives with its
    // transport's route decision; Windows and macOS are unrecorded (the
    // truthful endpoint-less state, stated per claim), Linux is recorded
    // and names the crate that carries its verifier.
    for claim in IdentityClaim::ALL {
        assert!(
            claim.waits_on().contains("route decision"),
            "{claim:?} must name the decision its verifier arrives with"
        );
        if claim == IdentityClaim::UnixPeerCredentials {
            assert!(
                claim.waits_on().contains("recorded: ADR-0055")
                    && claim.waits_on().contains("partman-transport-linux"),
                "the Linux claim names its recorded decision and its verifier's crate"
            );
        } else {
            assert!(
                claim.waits_on().contains("unrecorded"),
                "{claim:?} must state that no route is recorded yet"
            );
        }
        assert!(
            !claim.proves().is_empty(),
            "{claim:?} must name the identity fact the peer proves"
        );
    }

    // Distinct claims, distinct transports: the vocabulary carries no
    // alias a route decision could quietly widen through.
    let transports = IdentityClaim::ALL.map(IdentityClaim::transport);
    let mut deduped = transports.to_vec();
    deduped.sort_unstable();
    deduped.dedup();
    assert_eq!(deduped.len(), transports.len());
}
