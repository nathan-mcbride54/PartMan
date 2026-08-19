//! The WP-040 Linux transport (increment 5, on ADR-0055 and spec 19.0.0):
//! RPC-001's Linux clause as an endpoint.
//!
//! **What RPC-001 says, since 19.0.0:** "Unix domain socket in a root-owned
//! directory searchable but not writable by others (mode 0711), the socket
//! node owned by the authorizing user and accessible to that user alone
//! (mode 0600), with peer-credential verification of the connecting process
//! against that user." Two gates, both enforced by the kernel before the
//! helper parses a byte — the directory keeps the path unsquattable, the
//! node admits exactly one uid — and the credential check then *verifies*
//! what the kernel admitted. The round that decided it
//! (`docs/reviews/LINUX_TRANSPORT_ROUTE_ROUND_2026-08-19.md`) measured why
//! the old `0700` could not work and why the rest is shaped this way.
//!
//! **What this crate does.** On Linux, [`linux::Endpoint::create`] checks
//! the directory fail-closed (owned by this process's own effective uid,
//! mode exactly [`SOCKET_DIRECTORY_MODE`], a directory and not a link),
//! refuses a pre-existing node of any kind rather than replacing or
//! re-moding it, binds the node, and sets it to [`SOCKET_NODE_MODE`] owned by
//! the authorizing user. [`linux::Endpoint::accept`] reads the peer's
//! credentials through `rustix`'s safe `socket_peercred` and **refuses the
//! connection before any byte is read** unless the credentials' `uid` is
//! the authorizing user ([`verify_peer`], the verifier
//! `partman_rpc::identity::IdentityClaim::UnixPeerCredentials` waited on).
//! Then the RPC-002 handshake — both sides send theirs, each applies the
//! compatibility rule, an incompatible pair refuses with the remediation
//! and closes — over RPC-004's bounded, length-prefixed frames
//! ([`read_frame`], [`write_frame`]), with timeouts from the consumer's
//! `Timeouts`. The client side ([`linux::connect`]) connects by path and runs
//! the same handshake. Both ends in one crate, so laxness has nowhere to
//! live.
//!
//! **What this crate does not do.** It does not decide how the helper is
//! launched (a systemd unit, `pkexec`, polkit — WP-L110's), what the helper
//! does with the identity (HLP-003's floor act, WP-070/WP-L110), or the
//! helper's discovery/mutation route (ADR-0054). It carries no authorization
//! vocabulary (ADR-0021). It opens no network socket and contains no network
//! address type (SEC-007). It launches nothing. Per-message credentials
//! (`SCM_CREDENTIALS`) are deferred by ADR-0055 decision 4. On every
//! platform but Linux the endpoint refuses with
//! [`Refusal::UnsupportedPlatform`]; the pure seams ([`verify_peer`],
//! the framing) compile and test everywhere.
//!
//! **Tier-1 posture (ADR-0055 decision 6).** Tests may create a listener
//! under a temporary directory the test owns (the directory rule is
//! checked against the running effective uid, which is 0 in production),
//! a `UnixStream::pair` whose credentials are the test's own, and the
//! refusal arm through [`verify_peer`]'s seam with an injected credential —
//! a second uid is not constructible unprivileged. The root-owned
//! directory, a foreign uid refused by the kernel at the node, and a
//! cross-user connect are the Tier-2 acceptance this increment owes in a
//! disposable guest, recorded with it.
//!
//! **`SO_PEERCRED` semantics this crate relies on**, measured on the
//! round's apparatus: the credentials are those of the process that
//! called `connect`, captured at connection time; a `0700` root directory
//! refuses a non-root peer with `EACCES` before accept; a `0711` directory
//! admits it and the listener reads the peer's `pid/uid/gid`.

#![forbid(unsafe_code)]

use core::fmt;
use std::io::{Read, Write};

pub use partman_rpc::stream::Timeouts;
use partman_rpc::{DecodeRefusal, Handshake, MAX_MESSAGE_BYTES, VersionRefusal};

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(test)]
mod tests;

/// RPC-001's directory mode: root-owned, searchable but not writable by
/// others. Checked **exactly**: a more permissive directory lets another
/// user create or replace the node; a less permissive one refuses the
/// client (the measured `0700` case, SI-41).
pub const SOCKET_DIRECTORY_MODE: u32 = 0o711;

/// RPC-001's socket node mode: the authorizing user alone.
pub const SOCKET_NODE_MODE: u32 = 0o600;

/// The frame header: a big-endian `u32` length, bounded by
/// [`MAX_MESSAGE_BYTES`] before any allocation.
pub const FRAME_HEADER_BYTES: usize = 4;

/// Whether this build carries the endpoint at all: `Ok` on Linux, the
/// typed [`Refusal::UnsupportedPlatform`] everywhere else. The tests'
/// non-Linux arms assert this, so every annotated test examines something
/// on every platform (the `ffi-linux-loop` precedent).
///
/// # Errors
///
/// [`Refusal::UnsupportedPlatform`] off Linux.
pub const fn platform_support() -> Result<(), Refusal> {
    if cfg!(target_os = "linux") {
        Ok(())
    } else {
        Err(Refusal::UnsupportedPlatform)
    }
}

/// The node's file name for one authorizing user, under the directory —
/// flat per-user nodes (ADR-0055 decision 3).
#[must_use]
pub fn node_name(user: AuthorizingUser) -> String {
    format!("helper-{}.sock", user.0)
}

/// The user this endpoint was created to serve: RPC-001's "authorizing
/// user", runtime data the helper learns at endpoint creation (the same
/// role the Windows claim gives its SDDL's interactive user).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthorizingUser(pub u32);

/// What the kernel reported about the connecting process at `accept`
/// (`SO_PEERCRED`). Carried as plain integers so the verifier is pure and
/// testable on every platform; on Linux [`linux::Endpoint::accept`] fills it
/// from `rustix`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeerCredentials {
    /// The connecting process's effective uid at connect time.
    pub uid: u32,
    /// Its effective gid.
    pub gid: u32,
    /// Its pid.
    pub pid: i32,
}

/// A peer the verifier admitted: the credentials, and the user they
/// matched. Constructible only through [`verify_peer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerifiedPeer {
    credentials: PeerCredentials,
    user: AuthorizingUser,
}

impl VerifiedPeer {
    /// The kernel-reported credentials the verifier admitted.
    #[must_use]
    pub const fn credentials(&self) -> PeerCredentials {
        self.credentials
    }

    /// The authorizing user they matched.
    #[must_use]
    pub const fn user(&self) -> AuthorizingUser {
        self.user
    }
}

/// Why the transport refused. Every arm is typed; the strings carry no
/// peer-authored bytes (SEC-006 at this boundary: uids, modes, counts
/// and this crate's own words only).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The endpoint exists on Linux only.
    UnsupportedPlatform,
    /// The directory is not a directory, or is reached through a link.
    DirectoryNotADirectory,
    /// The directory is not owned by this process's effective uid — the
    /// helper's own, root in production.
    DirectoryNotOwnedByEndpoint {
        /// The endpoint's effective uid.
        expected_uid: u32,
        /// The directory's owner.
        found_uid: u32,
    },
    /// The directory's mode is not exactly [`SOCKET_DIRECTORY_MODE`].
    DirectoryMode {
        /// The permission bits found.
        found: u32,
    },
    /// A node already exists at the socket path — of any kind. Never
    /// replaced, never re-moded: the path is root-created and the
    /// endpoint touches only what it made.
    NodeAlreadyExists,
    /// The kernel-reported credentials are not the authorizing user's.
    /// Issued before any byte is read from the connection.
    PeerNotAuthorizingUser {
        /// The user the endpoint serves.
        expected_uid: u32,
        /// The uid the kernel reported.
        found_uid: u32,
    },
    /// The credentials could not be read; the connection is refused.
    PeerCredentialsUnreadable {
        /// This crate's own description of the failure.
        reason: String,
    },
    /// A frame header declared more than [`MAX_MESSAGE_BYTES`]; refused
    /// before allocation.
    FrameOverBound {
        /// The declared length.
        declared: u64,
        /// The bound.
        bound: usize,
    },
    /// The peer closed the connection mid-frame.
    FrameTruncated,
    /// The RPC-002 compatibility rule refused, with its remediation.
    Handshake(VersionRefusal),
    /// The peer's handshake or message failed the strict decode.
    Decode(DecodeRefusal),
    /// An I/O operation failed or timed out; the operation is named,
    /// the OS error carried as its kind's words only.
    Io {
        /// Which operation.
        operation: &'static str,
        /// The error kind, spelled by this crate.
        kind: String,
    },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => write!(f, "the Linux transport exists on Linux only"),
            Self::DirectoryNotADirectory => {
                write!(
                    f,
                    "the socket directory is not a directory reached without a link"
                )
            }
            Self::DirectoryNotOwnedByEndpoint {
                expected_uid,
                found_uid,
            } => write!(
                f,
                "the socket directory is owned by uid {found_uid}, not the endpoint's uid \
                 {expected_uid}"
            ),
            Self::DirectoryMode { found } => write!(
                f,
                "the socket directory mode is {found:o}, not {SOCKET_DIRECTORY_MODE:o}"
            ),
            Self::NodeAlreadyExists => {
                write!(
                    f,
                    "a node already exists at the socket path; it is not replaced"
                )
            }
            Self::PeerNotAuthorizingUser {
                expected_uid,
                found_uid,
            } => write!(
                f,
                "the connecting process is uid {found_uid}, not the authorizing user \
                 {expected_uid}; refused before any byte was read"
            ),
            Self::PeerCredentialsUnreadable { reason } => {
                write!(f, "the peer's credentials could not be read: {reason}")
            }
            Self::FrameOverBound { declared, bound } => {
                write!(
                    f,
                    "a frame declared {declared} bytes, over the {bound}-byte bound"
                )
            }
            Self::FrameTruncated => write!(f, "the peer closed the connection mid-frame"),
            Self::Handshake(refusal) => write!(f, "handshake refused: {}", refusal.remediation),
            Self::Decode(refusal) => write!(f, "strict decode refused: {refusal:?}"),
            Self::Io { operation, kind } => write!(f, "{operation} failed: {kind}"),
        }
    }
}

impl std::error::Error for Refusal {}

/// The verifier `IdentityClaim::UnixPeerCredentials` waited on: the
/// kernel-reported credentials must belong to the user the endpoint was
/// created to serve. Pure, so the refusal arm is testable where a second
/// uid cannot be made.
///
/// # Errors
///
/// [`Refusal::PeerNotAuthorizingUser`] naming both uids.
pub fn verify_peer(
    credentials: PeerCredentials,
    user: AuthorizingUser,
) -> Result<VerifiedPeer, Refusal> {
    if credentials.uid == user.0 {
        Ok(VerifiedPeer { credentials, user })
    } else {
        Err(Refusal::PeerNotAuthorizingUser {
            expected_uid: user.0,
            found_uid: credentials.uid,
        })
    }
}

/// Write one frame: a big-endian `u32` length, then the bytes. Refuses a
/// payload over [`MAX_MESSAGE_BYTES`] without writing anything — this side
/// cannot emit what the peer's [`read_frame`] would refuse.
///
/// # Errors
///
/// [`Refusal::FrameOverBound`] or [`Refusal::Io`].
pub fn write_frame<W: Write>(writer: &mut W, payload: &[u8]) -> Result<(), Refusal> {
    if payload.len() > MAX_MESSAGE_BYTES {
        return Err(Refusal::FrameOverBound {
            declared: payload.len() as u64,
            bound: MAX_MESSAGE_BYTES,
        });
    }
    let header = u32::try_from(payload.len()).map_err(|_| Refusal::FrameOverBound {
        declared: payload.len() as u64,
        bound: MAX_MESSAGE_BYTES,
    })?;
    writer
        .write_all(&header.to_be_bytes())
        .and_then(|()| writer.write_all(payload))
        .and_then(|()| writer.flush())
        .map_err(|error| io_refusal("write frame", &error))
}

/// Read one frame: the header, the bound check **before any allocation**,
/// then exactly the declared bytes.
///
/// # Errors
///
/// [`Refusal::FrameOverBound`] before allocation; [`Refusal::FrameTruncated`]
/// if the peer closes mid-frame; [`Refusal::Io`] otherwise (a read timeout
/// arrives here as `Io` with the kind named).
pub fn read_frame<R: Read>(reader: &mut R) -> Result<Vec<u8>, Refusal> {
    let mut header = [0u8; FRAME_HEADER_BYTES];
    read_exact_or_truncated(reader, &mut header, "read frame header")?;
    let declared = u32::from_be_bytes(header) as usize;
    if declared > MAX_MESSAGE_BYTES {
        return Err(Refusal::FrameOverBound {
            declared: declared as u64,
            bound: MAX_MESSAGE_BYTES,
        });
    }
    let mut payload = vec![0u8; declared];
    read_exact_or_truncated(reader, &mut payload, "read frame body")?;
    Ok(payload)
}

fn read_exact_or_truncated<R: Read>(
    reader: &mut R,
    buffer: &mut [u8],
    operation: &'static str,
) -> Result<(), Refusal> {
    reader.read_exact(buffer).map_err(|error| {
        if error.kind() == std::io::ErrorKind::UnexpectedEof {
            Refusal::FrameTruncated
        } else {
            io_refusal(operation, &error)
        }
    })
}

fn io_refusal(operation: &'static str, error: &std::io::Error) -> Refusal {
    Refusal::Io {
        operation,
        kind: format!("{:?}", error.kind()),
    }
}

/// RPC-002 over a stream, both ends: send the local handshake, read the
/// peer's through the strict decode, apply the compatibility rule. Both
/// sides send before reading so neither waits on the other; an
/// incompatible pair refuses with the remediation and the caller closes.
/// The consumer's `Timeouts::handshake_ms` is applied by the transport
/// before calling this (the stream's own read timeout).
///
/// # Errors
///
/// [`Refusal::Handshake`], [`Refusal::Decode`], or a frame refusal.
pub fn exchange_handshake<S: Read + Write>(
    stream: &mut S,
    local: &Handshake,
) -> Result<Handshake, Refusal> {
    let bytes = local.encode().map_err(Refusal::Decode)?;
    write_frame(stream, &bytes)?;
    let remote_bytes = read_frame(stream)?;
    let remote = Handshake::decode(&remote_bytes).map_err(Refusal::Decode)?;
    local.compatible_with(&remote).map_err(Refusal::Handshake)?;
    Ok(remote)
}
