//! The helper-authentication skeleton (WP-040 increment 4): the closed
//! per-transport claim vocabulary RPC-001 implies, as **types naming
//! what a peer proves, verified by nobody here**.
//!
//! RPC-001 names three transports, and each carries an identity
//! verification the connection must perform: the SDDL a Windows named
//! pipe must restrict access to, the peer credentials a Unix domain
//! socket must verify, the code-signing requirement a macOS XPC
//! connection (or its equivalently verified socket) must check. This
//! module is the vocabulary for those claims and nothing more: **no
//! claim has a verifier here**, because every transport is
//! route-decision-gated (the WP-035 increment-10 triangle, three times
//! over) and each claim's verifier arrives with its transport's
//! recorded route decision — the platform reach a verifier needs is
//! exactly the reach the route decision exists to cost out.
//! [`IdentityClaim::waits_on`] says which decision each claim waits
//! on, and `schemas/rpc/authentication.md` records the same mapping.
//!
//! **No authorization vocabulary exists here, deliberately.** SI-18
//! holds open whether a severity-1 plan needs fresh interactive
//! authorization — SAFE-002 and HLP-003 are written in contradiction —
//! and until the register resolves it, this skeleton names what a peer
//! proves about its **identity** and says nothing about what a peer
//! may do, when a human must approve, or when an approval expires.
//! HLP-003's authorization binding is WP-070's to implement under
//! whatever SI-18 decides; the closure test pins this vocabulary to
//! exactly the three identity claims so anything more is a visible
//! reviewed edit against that gate.

/// One per-transport identity claim: what a peer must prove it *is*
/// before the helper processes anything from it (RPC-001, consumed by
/// HLP-007's caller-identity rule — WP-070's to enforce).
///
/// The vocabulary is closed: one claim per RPC-001 transport, nothing
/// else. A claim is a name for an obligation, not an implementation —
/// verified by nobody here, with each verifier waiting on its
/// transport's recorded route decision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityClaim {
    /// Windows: the named pipe's SDDL restricts access to SYSTEM and
    /// the authorizing interactive user, so the peer's identity is
    /// proved by the ACL admitting the connection at all. The
    /// authorizing user is runtime data the transport learns at
    /// endpoint creation; this claim names the restriction, not a
    /// value.
    WindowsPipeSddl,
    /// Linux: the Unix domain socket (0700, root-owned directory)
    /// verifies peer credentials — the kernel-reported identity of the
    /// connecting process. Reading them needs `SO_PEERCRED` beyond
    /// std's surface, which is precisely why the verifier waits on the
    /// route decision.
    UnixPeerCredentials,
    /// macOS: the XPC connection checks a code-signing requirement —
    /// or an equivalently verified Unix domain socket does — so the
    /// peer proves it is a binary the requirement admits.
    MacosCodeSigning,
}

impl IdentityClaim {
    /// The closed vocabulary, in RPC-001's transport order. The
    /// closure test pins this to exactly these three claims.
    pub const ALL: [Self; 3] = [
        Self::WindowsPipeSddl,
        Self::UnixPeerCredentials,
        Self::MacosCodeSigning,
    ];

    /// The RPC-001 transport whose connections owe this claim.
    #[must_use]
    pub const fn transport(self) -> &'static str {
        match self {
            Self::WindowsPipeSddl => "windows named pipe",
            Self::UnixPeerCredentials => "linux unix domain socket",
            Self::MacosCodeSigning => "macos xpc or equivalently verified socket",
        }
    }

    /// What the peer proves — an identity fact, never an authorization
    /// fact (SI-18's gate).
    #[must_use]
    pub const fn proves(self) -> &'static str {
        match self {
            Self::WindowsPipeSddl => {
                "the connection was admitted by an SDDL restricting the pipe to SYSTEM \
                 and the authorizing interactive user"
            }
            Self::UnixPeerCredentials => {
                "the kernel-reported credentials of the connecting process match the \
                 peer the socket was created to serve"
            }
            Self::MacosCodeSigning => {
                "the connecting binary satisfies the helper's code-signing requirement"
            }
        }
    }

    /// The recorded route decision this claim's verifier arrives with.
    /// None is recorded yet: the protocol layer is complete and
    /// endpoint-less, which is a truthful state, not a gap.
    #[must_use]
    pub const fn waits_on(self) -> &'static str {
        match self {
            Self::WindowsPipeSddl => {
                "the windows transport route decision (Win32 security APIs; unrecorded)"
            }
            Self::UnixPeerCredentials => {
                "the linux transport route decision (SO_PEERCRED beyond std; unrecorded)"
            }
            Self::MacosCodeSigning => {
                "the macos transport route decision (platform frameworks; unrecorded)"
            }
        }
    }
}
