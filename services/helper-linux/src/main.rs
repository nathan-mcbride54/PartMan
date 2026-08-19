//! `partman-helper-linux --serve <uid> [--directory <dir>] [--idle-seconds <n>] [--audit <file>]`
//!
//! Launched through `pkexec` under `org.partman.helper.serve` (the launch
//! round's L2). Refuses unless `PKEXEC_UID` equals `<uid>`. Exit codes: 0
//! served (or another helper already serves this user); 2 launch refused;
//! 64 usage.
#![forbid(unsafe_code)]

#[cfg(target_os = "linux")]
fn main() {
    use std::path::PathBuf;

    use partman_helper_linux::linux::{Config, FileAudit, run};
    use partman_helper_linux::{
        DEFAULT_DIRECTORY, DEFAULT_IDLE_SECONDS, LaunchRefusal, PKEXEC_UID_VARIABLE, launch_rule,
    };
    use partman_transport_linux::Timeouts;

    let args: Vec<String> = std::env::args().collect();
    let mut uid: Option<u32> = None;
    let mut directory = PathBuf::from(DEFAULT_DIRECTORY);
    let mut idle_seconds = DEFAULT_IDLE_SECONDS;
    let mut audit_path: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match (args[i].as_str(), args.get(i + 1)) {
            ("--serve", Some(v)) => uid = v.parse().ok(),
            ("--directory", Some(v)) => directory = PathBuf::from(v),
            ("--idle-seconds", Some(v)) => idle_seconds = v.parse().unwrap_or(DEFAULT_IDLE_SECONDS),
            ("--audit", Some(v)) => audit_path = Some(PathBuf::from(v)),
            _ => {
                eprintln!(
                    "usage: partman-helper-linux --serve <uid> [--directory <dir>] [--idle-seconds <n>] [--audit <file>]"
                );
                std::process::exit(64);
            }
        }
        i += 2;
    }
    let Some(requested) = uid else {
        eprintln!("usage: partman-helper-linux --serve <uid> ...");
        std::process::exit(64);
    };
    let pkexec_uid = std::env::var(PKEXEC_UID_VARIABLE).ok();
    let uid = match launch_rule(requested, pkexec_uid.as_deref()) {
        Ok(uid) => uid,
        Err(refusal) => {
            eprintln!("launch refused: {refusal}");
            std::process::exit(2);
        }
    };
    let mut audit = match FileAudit::open(audit_path.as_deref()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("audit log: {:?}", e.kind());
            std::process::exit(2);
        }
    };
    let config = Config {
        uid,
        directory,
        idle_seconds,
        build: env!("CARGO_PKG_VERSION").to_owned(),
        timeouts: Timeouts {
            request_ms: 30_000,
            handshake_ms: 10_000,
        },
    };
    match run(&config, &mut audit) {
        Ok(()) => {}
        Err(LaunchRefusal::AlreadyServed) => {
            eprintln!("{}", LaunchRefusal::AlreadyServed);
        }
        Err(refusal) => {
            eprintln!("launch refused: {refusal}");
            std::process::exit(2);
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("the Linux helper exists on Linux only");
    std::process::exit(2);
}
