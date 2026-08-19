//! The Tier-2 acceptance instrument for the Linux transport (ADR-0055
//! decision 6): the arms Tier-1 cannot reach — a root-owned directory, a
//! foreign uid refused by the kernel at the node, a root peer refused by the
//! verifier before any byte is read — run in a disposable guest by root
//! and two unprivileged users. Not a product surface; it prints typed
//! outcomes and exits non-zero on refusal so a transcript can be graded.
//!
//! ```text
//! endpoint-probe serve <directory> <authorizing-uid>   # as root: create, accept once, echo one frame
//! endpoint-probe connect <socket-path>                 # as any user: connect, handshake, send/receive one frame
//! ```
#![forbid(unsafe_code)]

#[cfg(target_os = "linux")]
fn main() {
    use std::path::PathBuf;

    use partman_rpc::Handshake;
    use partman_transport_linux::linux::{Endpoint, connect};
    use partman_transport_linux::{AuthorizingUser, Timeouts, read_frame, write_frame};

    let args: Vec<String> = std::env::args().collect();
    let timeouts = Timeouts {
        request_ms: 10_000,
        handshake_ms: 10_000,
    };
    match args.get(1).map(String::as_str) {
        Some("serve") => {
            let directory = PathBuf::from(&args[2]);
            let uid: u32 = args[3].parse().expect("authorizing uid");
            match Endpoint::create(&directory, AuthorizingUser(uid), timeouts) {
                Ok(endpoint) => {
                    println!("serve: created {}", endpoint.path().display());
                    match endpoint.accept(&Handshake::local("0.0.0")) {
                        Ok(mut conn) => {
                            let creds = conn.peer().credentials();
                            println!(
                                "serve: admitted uid={} gid={} pid={} build={}",
                                creds.uid,
                                creds.gid,
                                creds.pid,
                                conn.remote().build
                            );
                            match read_frame(conn.stream()) {
                                Ok(frame) => {
                                    println!("serve: frame {} bytes", frame.len());
                                    let _ = write_frame(conn.stream(), &frame);
                                }
                                Err(refusal) => println!("serve: frame refused: {refusal}"),
                            }
                        }
                        Err(refusal) => {
                            println!("serve: refused: {refusal}");
                            std::process::exit(3);
                        }
                    }
                }
                Err(refusal) => {
                    println!("serve: endpoint refused: {refusal}");
                    std::process::exit(2);
                }
            }
        }
        Some("connect") => {
            let path = PathBuf::from(&args[2]);
            match connect(&path, &Handshake::local("0.0.1"), timeouts) {
                Ok(mut conn) => {
                    println!("connect: handshaken with build {}", conn.remote().build);
                    write_frame(conn.stream(), b"probe").expect("write");
                    match read_frame(conn.stream()) {
                        Ok(frame) => println!("connect: echoed {} bytes", frame.len()),
                        Err(refusal) => println!("connect: frame refused: {refusal}"),
                    }
                }
                Err(refusal) => {
                    println!("connect: refused: {refusal}");
                    std::process::exit(4);
                }
            }
        }
        _ => {
            eprintln!("usage: endpoint-probe serve <dir> <uid> | connect <path>");
            std::process::exit(64);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("the Linux transport exists on Linux only");
    std::process::exit(2);
}
