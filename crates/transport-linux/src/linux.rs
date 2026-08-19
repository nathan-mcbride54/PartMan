//! The Linux endpoint and client (ADR-0055): `std`'s Unix sockets and
//! filesystem metadata for everything but one call — `rustix`'s safe
//! `socket_peercred` for the kernel-reported credentials of the peer.

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, chown};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

use partman_rpc::Handshake;

use crate::{
    AuthorizingUser, PeerCredentials, Refusal, SOCKET_DIRECTORY_MODE, SOCKET_NODE_MODE, Timeouts,
    VerifiedPeer, exchange_handshake, node_name, verify_peer,
};

/// A created endpoint: the listener on a node this process made, in a
/// directory it verified.
#[derive(Debug)]
pub struct Endpoint {
    listener: UnixListener,
    path: PathBuf,
    user: AuthorizingUser,
    timeouts: Timeouts,
}

/// An accepted, verified, handshaken connection on the helper side.
#[derive(Debug)]
pub struct Connection {
    stream: UnixStream,
    peer: VerifiedPeer,
    remote: Handshake,
}

/// A connected, handshaken connection on the client side.
#[derive(Debug)]
pub struct ClientConnection {
    stream: UnixStream,
    remote: Handshake,
}

impl Endpoint {
    /// Create the endpoint for one authorizing user under `directory`,
    /// fail-closed on every RPC-001 rule:
    ///
    /// - `directory` must be a directory reached without a symlink, owned
    ///   by this process's effective uid (the helper's own; root in
    ///   production), mode exactly [`SOCKET_DIRECTORY_MODE`];
    /// - no node of any kind may already exist at the socket path — it is
    ///   never replaced and never re-moded;
    /// - the node this binds is set to [`SOCKET_NODE_MODE`] and owned by
    ///   `user` before the endpoint is returned.
    ///
    /// Between `bind` and the mode/owner change the node carries the
    /// process umask's default for a moment; a connection admitted in that
    /// window is still refused by [`Endpoint::accept`]'s credential check —
    /// the second gate is what makes the first's window harmless.
    ///
    /// # Errors
    ///
    /// The first rule violated, as a typed [`Refusal`].
    pub fn create(
        directory: &Path,
        user: AuthorizingUser,
        timeouts: Timeouts,
    ) -> Result<Self, Refusal> {
        check_directory(directory)?;
        let path = directory.join(node_name(user));
        if fs::symlink_metadata(&path).is_ok() {
            return Err(Refusal::NodeAlreadyExists);
        }
        let listener = UnixListener::bind(&path).map_err(|e| io_refusal("bind", &e))?;
        let node = fs::Permissions::from_mode(SOCKET_NODE_MODE);
        if let Err(e) = fs::set_permissions(&path, node) {
            let _ = fs::remove_file(&path);
            return Err(io_refusal("set node mode", &e));
        }
        if let Err(e) = chown(&path, Some(user.0), None) {
            let _ = fs::remove_file(&path);
            return Err(io_refusal("set node owner", &e));
        }
        Ok(Self {
            listener,
            path,
            user,
            timeouts,
        })
    }

    /// The node's path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The user this endpoint serves.
    #[must_use]
    pub const fn user(&self) -> AuthorizingUser {
        self.user
    }

    /// Accept one connection: read the peer's credentials, refuse unless
    /// they are the authorizing user's — **before any byte is read** — then
    /// run the RPC-002 handshake under the handshake timeout.
    ///
    /// # Errors
    ///
    /// [`Refusal::PeerNotAuthorizingUser`], [`Refusal::PeerCredentialsUnreadable`],
    /// a handshake, decode, frame or I/O refusal. A refused connection is
    /// dropped (closed) by this function.
    pub fn accept(&self, local: &Handshake) -> Result<Connection, Refusal> {
        let (mut stream, _) = self
            .listener
            .accept()
            .map_err(|e| io_refusal("accept", &e))?;
        let credentials = peer_credentials(&stream)?;
        let (peer, remote) = admit(&mut stream, credentials, self.user, local, self.timeouts)?;
        Ok(Connection {
            stream,
            peer,
            remote,
        })
    }
}

/// The admission of an accepted stream, with the credentials already
/// read: verify them against the user **before any byte is read** from
/// the stream, then run the handshake under the timeouts. Separated from
/// [`Endpoint::accept`] so the refusal arm is testable where a second
/// uid cannot be made: a test injects the credentials and proves the
/// stream's bytes are still unread after the refusal.
///
/// # Errors
///
/// [`Refusal::PeerNotAuthorizingUser`] first; then the handshake's.
pub fn admit(
    stream: &mut UnixStream,
    credentials: PeerCredentials,
    user: AuthorizingUser,
    local: &Handshake,
    timeouts: Timeouts,
) -> Result<(VerifiedPeer, Handshake), Refusal> {
    let peer = verify_peer(credentials, user)?;
    apply_timeout(stream, timeouts.handshake_ms)?;
    let remote = exchange_handshake(stream, local)?;
    apply_timeout(stream, timeouts.request_ms)?;
    Ok((peer, remote))
}

impl Drop for Endpoint {
    fn drop(&mut self) {
        // The node this process made, and only that node.
        let _ = fs::remove_file(&self.path);
    }
}

impl Connection {
    /// The verified peer.
    #[must_use]
    pub const fn peer(&self) -> VerifiedPeer {
        self.peer
    }

    /// The peer's handshake, as decoded.
    #[must_use]
    pub const fn remote(&self) -> &Handshake {
        &self.remote
    }

    /// The stream, for the consumer's framed request/response traffic
    /// ([`crate::read_frame`], [`crate::write_frame`]).
    pub fn stream(&mut self) -> &mut UnixStream {
        &mut self.stream
    }
}

impl ClientConnection {
    /// The helper's handshake, as decoded.
    #[must_use]
    pub const fn remote(&self) -> &Handshake {
        &self.remote
    }

    /// The stream, for the consumer's framed traffic.
    pub fn stream(&mut self) -> &mut UnixStream {
        &mut self.stream
    }
}

/// Connect to an endpoint's node and run the RPC-002 handshake. The
/// kernel admits or refuses the connection at the node (the first gate);
/// an `EACCES` here is the kernel saying this process is not the node's
/// user.
///
/// # Errors
///
/// [`Refusal::Io`] on connect (the kernel's admission), a handshake, decode
/// or frame refusal.
pub fn connect(
    path: &Path,
    local: &Handshake,
    timeouts: Timeouts,
) -> Result<ClientConnection, Refusal> {
    let mut stream = UnixStream::connect(path).map_err(|e| io_refusal("connect", &e))?;
    apply_timeout(&stream, timeouts.handshake_ms)?;
    let remote = exchange_handshake(&mut stream, local)?;
    apply_timeout(&stream, timeouts.request_ms)?;
    Ok(ClientConnection { stream, remote })
}

/// The kernel-reported credentials of a connected stream's peer
/// (`SO_PEERCRED`, captured at connect time), through `rustix`'s safe
/// wrapper. Public so a consumer holding a `UnixStream` from elsewhere (a
/// pre-connected pair, ADR-0055's revisit condition) can verify it with
/// [`verify_peer`].
///
/// # Errors
///
/// [`Refusal::PeerCredentialsUnreadable`].
pub fn peer_credentials(stream: &UnixStream) -> Result<PeerCredentials, Refusal> {
    let ucred = rustix::net::sockopt::socket_peercred(stream).map_err(|e| {
        Refusal::PeerCredentialsUnreadable {
            reason: format!("SO_PEERCRED: {e}"),
        }
    })?;
    Ok(PeerCredentials {
        uid: ucred.uid.as_raw(),
        gid: ucred.gid.as_raw(),
        pid: ucred.pid.as_raw_nonzero().get(),
    })
}

/// The directory rule, exactly: a directory reached without a link, owned
/// by this process's effective uid, mode [`SOCKET_DIRECTORY_MODE`].
///
/// # Errors
///
/// The first rule violated.
pub fn check_directory(directory: &Path) -> Result<(), Refusal> {
    let meta = fs::symlink_metadata(directory).map_err(|e| io_refusal("stat directory", &e))?;
    if !meta.file_type().is_dir() {
        return Err(Refusal::DirectoryNotADirectory);
    }
    let expected_uid = rustix::process::geteuid().as_raw();
    if meta.uid() != expected_uid {
        return Err(Refusal::DirectoryNotOwnedByEndpoint {
            expected_uid,
            found_uid: meta.uid(),
        });
    }
    let found = meta.permissions().mode() & 0o7777;
    if found != SOCKET_DIRECTORY_MODE {
        return Err(Refusal::DirectoryMode { found });
    }
    Ok(())
}

fn apply_timeout(stream: &UnixStream, millis: u64) -> Result<(), Refusal> {
    let duration = Some(Duration::from_millis(millis.max(1)));
    stream
        .set_read_timeout(duration)
        .and_then(|()| stream.set_write_timeout(duration))
        .map_err(|e| io_refusal("set timeout", &e))
}

fn io_refusal(operation: &'static str, error: &std::io::Error) -> Refusal {
    Refusal::Io {
        operation,
        kind: format!("{:?}", error.kind()),
    }
}
