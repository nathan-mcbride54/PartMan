use std::io::Cursor;

use partman_rpc::MAX_MESSAGE_BYTES;

use crate::{
    AuthorizingUser, FRAME_HEADER_BYTES, PeerCredentials, Refusal, read_frame, verify_peer,
    write_frame,
};

// Requirements: RPC-001, SAFE-005
//   The verifier IdentityClaim::UnixPeerCredentials waited on: the
//   kernel-reported credentials must be the authorizing user's. Pure, so
//   the refusal arm is testable where a second uid cannot be made: a
//   matching uid is admitted carrying its credentials and the user it
//   matched; any other uid is refused naming both, and nothing else about
//   the credentials (gid, pid) admits.
// Evidence: the_verifier_admits_the_authorizing_user_and_refuses_every_other_uid
#[test]
fn the_verifier_admits_the_authorizing_user_and_refuses_every_other_uid() {
    let user = AuthorizingUser(1000);
    let mine = PeerCredentials {
        uid: 1000,
        gid: 1000,
        pid: 4242,
    };
    let admitted = verify_peer(mine, user).expect("the authorizing user is admitted");
    assert_eq!(admitted.credentials(), mine);
    assert_eq!(admitted.user(), user);
    for other in [0, 999, 1001, 65534] {
        let creds = PeerCredentials {
            uid: other,
            gid: 1000,
            pid: 4242,
        };
        assert_eq!(
            verify_peer(creds, user),
            Err(Refusal::PeerNotAuthorizingUser {
                expected_uid: 1000,
                found_uid: other
            }),
            "uid {other} is refused even with the user's gid"
        );
    }
}

// Requirements: RPC-004, SAFE-005
//   RPC-004's bound binds the wire before any parsing: a frame header
//   declaring more than MAX_MESSAGE_BYTES is refused before allocation
//   (the reader never asks for the body), a truncated body is a typed
//   truncation, a payload over the bound is refused before a byte is
//   written, and a frame at exactly the bound round-trips.
// Evidence: frames_are_bounded_before_allocation_and_round_trip_at_the_bound
#[test]
fn frames_are_bounded_before_allocation_and_round_trip_at_the_bound() {
    let mut wire = Vec::new();
    write_frame(&mut wire, b"hello").unwrap();
    assert_eq!(&wire[..FRAME_HEADER_BYTES], &5u32.to_be_bytes());
    assert_eq!(read_frame(&mut Cursor::new(&wire)).unwrap(), b"hello");

    let over = u32::try_from(MAX_MESSAGE_BYTES + 1).unwrap().to_be_bytes();
    assert_eq!(
        read_frame(&mut Cursor::new(&over[..])),
        Err(Refusal::FrameOverBound {
            declared: (MAX_MESSAGE_BYTES + 1) as u64,
            bound: MAX_MESSAGE_BYTES
        }),
        "refused on the header alone; no body was offered and none was needed"
    );
    let truncated = [&3u32.to_be_bytes()[..], b"ab"].concat();
    assert_eq!(
        read_frame(&mut Cursor::new(&truncated)),
        Err(Refusal::FrameTruncated)
    );
    assert_eq!(
        read_frame(&mut Cursor::new(&[1u8, 2][..])),
        Err(Refusal::FrameTruncated)
    );

    let too_big = vec![0u8; MAX_MESSAGE_BYTES + 1];
    let mut sink = Vec::new();
    assert!(matches!(
        write_frame(&mut sink, &too_big),
        Err(Refusal::FrameOverBound { .. })
    ));
    assert!(sink.is_empty(), "nothing was written before the refusal");

    let at_bound = vec![7u8; MAX_MESSAGE_BYTES];
    let mut wire = Vec::new();
    write_frame(&mut wire, &at_bound).unwrap();
    assert_eq!(read_frame(&mut Cursor::new(&wire)).unwrap(), at_bound);
}

// Requirements: RPC-002, SAFE-005
//   RPC-002 over a stream, both ends in one function: a compatible pair
//   exchanges handshakes and each side holds the other's; an incompatible
//   pair refuses with the remediation naming the older side — never a
//   silent degrade — and a peer whose handshake fails the strict decode
//   is refused as a decode, not read around.
//   Off Unix the test asserts the typed unsupported-platform refusal,
//   which is what the clause means there.
// Evidence: the_handshake_exchange_is_total_and_refuses_with_a_remediation
#[test]
fn the_handshake_exchange_is_total_and_refuses_with_a_remediation() {
    #[cfg(not(unix))]
    {
        assert_eq!(crate::platform_support(), Err(Refusal::UnsupportedPlatform));
    }
    #[cfg(unix)]
    {
        use std::os::unix::net::UnixStream;

        use partman_rpc::{Handshake, PROTOCOL_VERSION};

        use crate::exchange_handshake;
        let (mut a, mut b) = UnixStream::pair().unwrap();
        let local_a = Handshake::local("1.2.3");
        let local_b = Handshake::local("1.2.4");
        let t = std::thread::spawn(move || exchange_handshake(&mut b, &local_b));
        let got_b = exchange_handshake(&mut a, &local_a).unwrap();
        let got_a = t.join().unwrap().unwrap();
        assert_eq!(got_b.build, "1.2.4");
        assert_eq!(got_a.build, "1.2.3");

        let (mut a, mut b) = UnixStream::pair().unwrap();
        let newer = Handshake {
            protocol_version: PROTOCOL_VERSION + 1,
            build: "2.0.0".to_owned(),
        };
        let local = Handshake::local("1.2.3");
        let t = std::thread::spawn(move || exchange_handshake(&mut b, &newer));
        let refusal = exchange_handshake(&mut a, &local).unwrap_err();
        match refusal {
            Refusal::Handshake(v) => {
                assert_eq!(
                    (v.local, v.remote),
                    (PROTOCOL_VERSION, PROTOCOL_VERSION + 1)
                );
                assert!(v.remediation.contains("this side speaks an older protocol"));
            }
            other => panic!("expected a handshake refusal, got {other:?}"),
        }
        assert!(matches!(t.join().unwrap(), Err(Refusal::Handshake(_))));

        let (mut a, mut b) = UnixStream::pair().unwrap();
        // The garbage peer writes, then reads our handshake before it drops its
        // end, so our own write cannot race a closed pipe.
        let t = std::thread::spawn(move || {
            write_frame(&mut b, b"not a handshake").unwrap();
            read_frame(&mut b).unwrap()
        });
        let refusal = exchange_handshake(&mut a, &Handshake::local("1.2.3")).unwrap_err();
        assert!(matches!(refusal, Refusal::Decode(_)), "got {refusal:?}");
        let ours = t.join().unwrap();
        assert_eq!(Handshake::decode(&ours).unwrap().build, "1.2.3");
    }
}

#[cfg(target_os = "linux")]
mod linux_support {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use crate::Timeouts;

    pub fn euid() -> u32 {
        rustix::process::geteuid().as_raw()
    }

    pub fn fresh_directory(tag: &str, mode: u32) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "partman-transport-{tag}-{}-{}",
            std::process::id(),
            rustix::process::geteuid().as_raw()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(mode)).unwrap();
        dir
    }

    // Generous on purpose: these are bounds against a stalled peer, not
    // a measurement of one, and a loaded CI host must not turn a bound
    // into a flake.
    pub fn timeouts() -> Timeouts {
        Timeouts {
            request_ms: 30_000,
            handshake_ms: 30_000,
        }
    }
}

// Requirements: RPC-001, SAFE-005
//   The directory rule, fail-closed and exact: a 0711 directory owned
//   by this process's effective uid is accepted; 0755 (others may
//   write nothing, but the mode is not RPC-001's) and 0700 (the
//   measured SI-41 case, which would refuse the client) are both
//   refused naming the mode found; a symlink to a good directory is
//   refused as not-a-directory; a pre-existing node at the socket
//   path — here a plain file — is refused and left untouched, never
//   replaced or re-moded; and the node the endpoint makes is 0600,
//   owned by the authorizing user, and removed when the endpoint drops.
//   Off Linux the test asserts the typed unsupported-platform refusal.
// Evidence: the_endpoint_checks_the_directory_exactly_and_makes_a_user_owned_node
#[test]
fn the_endpoint_checks_the_directory_exactly_and_makes_a_user_owned_node() {
    #[cfg(not(target_os = "linux"))]
    {
        assert_eq!(crate::platform_support(), Err(Refusal::UnsupportedPlatform));
    }
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};

        use self::linux_support::{euid, fresh_directory, timeouts};
        use crate::linux::{Endpoint, check_directory};
        use crate::{SOCKET_DIRECTORY_MODE, SOCKET_NODE_MODE};

        let me = AuthorizingUser(euid());
        let good = fresh_directory("good", SOCKET_DIRECTORY_MODE);
        assert_eq!(check_directory(&good), Ok(()));
        for bad_mode in [0o755, 0o700, 0o777, 0o710] {
            let dir = fresh_directory(&format!("mode{bad_mode:o}"), bad_mode);
            assert_eq!(
                check_directory(&dir),
                Err(Refusal::DirectoryMode { found: bad_mode })
            );
            let _ = fs::remove_dir_all(&dir);
        }
        let link =
            std::env::temp_dir().join(format!("partman-transport-link-{}", std::process::id()));
        let _ = fs::remove_file(&link);
        std::os::unix::fs::symlink(&good, &link).unwrap();
        assert_eq!(check_directory(&link), Err(Refusal::DirectoryNotADirectory));
        let _ = fs::remove_file(&link);

        let squat = fresh_directory("squat", SOCKET_DIRECTORY_MODE);
        let node = squat.join(crate::node_name(me));
        fs::write(&node, b"stranger").unwrap();
        assert_eq!(
            Endpoint::create(&squat, me, timeouts()).map(|_| ()),
            Err(Refusal::NodeAlreadyExists)
        );
        assert_eq!(
            fs::read(&node).unwrap(),
            b"stranger",
            "the stranger's node is untouched"
        );
        let _ = fs::remove_dir_all(&squat);

        let path = {
            let endpoint = Endpoint::create(&good, me, timeouts()).unwrap();
            let meta = fs::metadata(endpoint.path()).unwrap();
            assert!(meta.file_type().is_socket());
            assert_eq!(meta.permissions().mode() & 0o7777, SOCKET_NODE_MODE);
            assert_eq!(meta.uid(), me.0);
            assert_eq!(
                Endpoint::create(&good, me, timeouts()).map(|_| ()),
                Err(Refusal::NodeAlreadyExists),
                "a second endpoint for the same user does not replace the first"
            );
            endpoint.path().to_path_buf()
        };
        assert!(
            !path.exists(),
            "the node this process made is removed on drop"
        );
        let _ = fs::remove_dir_all(&good);
    }
}

// Requirements: RPC-001, RPC-002, RPC-004, SEC-007
//   The whole path at the client baseline: the test's own uid connects
//   to its own endpoint, the kernel admits it at the 0600 node, the
//   endpoint reads SO_PEERCRED (this process's uid/gid/pid), the
//   verifier admits it, the handshake exchanges, and bounded frames
//   flow both ways. peer_credentials on a pair reports the test's own
//   euid, which is the verifier's positive case; a foreign uid is not
//   constructible here and is the Tier-2 acceptance's. Off Linux the
//   test asserts the typed unsupported-platform refusal.
// Evidence: an_authorizing_user_is_admitted_verified_and_handshaken_end_to_end
#[test]
fn an_authorizing_user_is_admitted_verified_and_handshaken_end_to_end() {
    #[cfg(not(target_os = "linux"))]
    {
        assert_eq!(crate::platform_support(), Err(Refusal::UnsupportedPlatform));
    }
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        use std::os::unix::net::UnixStream;

        use partman_rpc::Handshake;

        use self::linux_support::{euid, fresh_directory, timeouts};
        use crate::SOCKET_DIRECTORY_MODE;
        use crate::linux::{Endpoint, connect, peer_credentials};

        let me = AuthorizingUser(euid());
        let dir = fresh_directory("e2e", SOCKET_DIRECTORY_MODE);
        let endpoint = Endpoint::create(&dir, me, timeouts()).unwrap();
        let path = endpoint.path().to_path_buf();
        let helper = Handshake::local("0.1.0");
        let t = std::thread::spawn(move || {
            let mut conn = endpoint.accept(&helper).unwrap();
            let peer = conn.peer();
            let got = read_frame(conn.stream()).unwrap();
            write_frame(conn.stream(), &[got.as_slice(), b"!"].concat()).unwrap();
            (peer, conn.remote().build.clone())
        });
        let client = Handshake::local("0.1.1");
        let mut conn = connect(&path, &client, timeouts()).unwrap();
        assert_eq!(conn.remote().build, "0.1.0");
        write_frame(conn.stream(), b"ping").unwrap();
        assert_eq!(read_frame(conn.stream()).unwrap(), b"ping!");
        let (peer, client_build) = t.join().unwrap();
        assert_eq!(peer.credentials().uid, me.0);
        assert_eq!(
            peer.credentials().pid,
            i32::try_from(std::process::id()).unwrap()
        );
        assert_eq!(peer.user(), me);
        assert_eq!(client_build, "0.1.1");

        let (a, _b) = UnixStream::pair().unwrap();
        let creds = peer_credentials(&a).unwrap();
        assert_eq!(creds.uid, euid());
        let _ = fs::remove_dir_all(&dir);
    }
}

// Requirements: RPC-001, HLP-007, SAFE-005
//   The refusal arm of the verifier, on a real stream, before any
//   byte is read: a peer whose injected credentials are not the
//   authorizing user's is refused by admit(), and the bytes the peer
//   had already sent are still in the stream afterwards — the
//   handshake never ran and nothing was consumed. A second uid is not
//   constructible unprivileged, which is why the credentials are
//   injected here and the kernel-side arms are the Tier-2 acceptance's.
//   Off Linux the test asserts the typed unsupported-platform refusal.
// Evidence: a_foreign_uid_is_refused_before_any_byte_is_read
#[test]
fn a_foreign_uid_is_refused_before_any_byte_is_read() {
    #[cfg(not(target_os = "linux"))]
    {
        assert_eq!(crate::platform_support(), Err(Refusal::UnsupportedPlatform));
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::net::UnixStream;

        use partman_rpc::Handshake;

        use self::linux_support::{euid, timeouts};
        use crate::linux::admit;

        let me = AuthorizingUser(euid());
        let (mut client, mut server) = UnixStream::pair().unwrap();
        write_frame(&mut client, b"sent before the helper looked").unwrap();
        let foreign = PeerCredentials {
            uid: me.0.wrapping_add(1),
            gid: 0,
            pid: 1,
        };
        let refusal = admit(
            &mut server,
            foreign,
            me,
            &Handshake::local("0.1.0"),
            timeouts(),
        )
        .map(|_| ())
        .unwrap_err();
        assert_eq!(
            refusal,
            Refusal::PeerNotAuthorizingUser {
                expected_uid: me.0,
                found_uid: me.0.wrapping_add(1)
            }
        );
        assert_eq!(
            read_frame(&mut server).unwrap(),
            b"sent before the helper looked",
            "the refused peer's bytes were never read"
        );
        let mine = PeerCredentials {
            uid: me.0,
            gid: 0,
            pid: 1,
        };
        let t = std::thread::spawn(move || {
            let (peer, remote) = admit(
                &mut server,
                mine,
                me,
                &Handshake::local("0.1.0"),
                timeouts(),
            )
            .unwrap();
            (peer.user(), remote.build)
        });
        let remote = crate::exchange_handshake(&mut client, &Handshake::local("0.2.0")).unwrap();
        assert_eq!(remote.build, "0.1.0");
        assert_eq!(t.join().unwrap(), (me, "0.2.0".to_owned()));
    }
}
