//! The helper on Linux: the endpoint, the serve loop, the idle watchdog,
//! and the system backend that enumerates through the adapter as root.

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use partman_adapter_linux::contract::{SystemContractSource, sysfs_root, udev_root};
use partman_adapter_linux::devices::{DeviceKind, Enumeration, HostAssembledKind, enumerate};
use partman_rpc::Handshake;
use partman_transport_linux::linux::Endpoint;
use partman_transport_linux::{
    AuthorizingUser, Refusal as TransportRefusal, SOCKET_DIRECTORY_MODE, Timeouts, node_name,
};

use crate::{
    AuditEvent, AuditSink, Backend, EnumeratedDevice, LaunchRefusal, Operation, Response,
    serve_connection,
};

/// The helper's configuration for one run.
#[derive(Clone, Debug)]
pub struct Config {
    /// The uid to serve (already checked by [`crate::launch_rule`]).
    pub uid: u32,
    /// The runtime directory (root-owned, `0711`; created if absent).
    pub directory: PathBuf,
    /// Seconds without a connection before the helper exits (HLP-005).
    pub idle_seconds: u64,
    /// The build version for the handshake.
    pub build: String,
    /// Transport timeouts.
    pub timeouts: Timeouts,
}

/// The audit log: one line per event, appended (HLP-006). The file is the
/// consumer's choice; stderr when none.
pub struct FileAudit {
    sink: Box<dyn Write + Send>,
}

impl FileAudit {
    /// Append to a file (created `0600`), or to stderr when `path` is
    /// `None`.
    ///
    /// # Errors
    ///
    /// The file could not be opened.
    pub fn open(path: Option<&Path>) -> std::io::Result<Self> {
        let sink: Box<dyn Write + Send> = match path {
            Some(p) => {
                let f = fs::OpenOptions::new().create(true).append(true).open(p)?;
                fs::set_permissions(p, fs::Permissions::from_mode(0o600))?;
                Box::new(f)
            }
            None => Box::new(std::io::stderr()),
        };
        Ok(Self { sink })
    }
}

impl AuditSink for FileAudit {
    fn record(&mut self, event: AuditEvent) {
        let ts = now_secs();
        let _ = writeln!(self.sink, "ts={ts} {event}");
        let _ = self.sink.flush();
    }
}

/// The backend over the real contract, as root.
pub struct SystemBackend {
    uid: u32,
    build: String,
}

impl SystemBackend {
    /// For one served uid and this build.
    #[must_use]
    pub fn new(uid: u32, build: &str) -> Self {
        Self {
            uid,
            build: build.to_owned(),
        }
    }
}

/// Whether a node for this uid already exists under the directory: the
/// launch rule's "another helper serves this user" check, used by [`run`]
/// before creating an endpoint.
#[must_use]
pub fn already_served(directory: &Path, uid: u32) -> bool {
    fs::symlink_metadata(directory.join(node_name(AuthorizingUser(uid)))).is_ok()
}

impl Backend for SystemBackend {
    fn status(&self) -> Response {
        Response::Status {
            build: self.build.clone(),
            authorizing_uid: self.uid,
            served: Operation::ALL
                .into_iter()
                .filter(|op| op.served_in_increment().is_none())
                .collect(),
        }
    }

    fn enumerate(&self) -> Response {
        let source = SystemContractSource;
        let (outcome, devices) = match enumerate(&source, &sysfs_root(), &udev_root()) {
            Enumeration::Listed { devices } => (
                "listed",
                devices
                    .iter()
                    .map(|d| EnumeratedDevice {
                        selector: d.selector.clone(),
                        kind: kind_name(&d.kind),
                        transport: format!("{:?}", d.transport),
                        properties: d.properties.len() as u64,
                    })
                    .collect(),
            ),
            Enumeration::OverLimit { .. } => ("over-limit", Vec::new()),
            Enumeration::Unavailable { .. } => ("unavailable", Vec::new()),
            Enumeration::Failed { .. } => ("failed", Vec::new()),
        };
        Response::Enumeration {
            proposal: true,
            outcome: outcome.to_owned(),
            devices,
        }
    }
}

fn kind_name(kind: &DeviceKind) -> String {
    match kind {
        DeviceKind::Plain => "plain".to_owned(),
        DeviceKind::HostAssembled(HostAssembledKind::DeviceMapper) => {
            "host-assembled:device-mapper".to_owned()
        }
        DeviceKind::HostAssembled(HostAssembledKind::Mdraid) => "host-assembled:mdraid".to_owned(),
        DeviceKind::HostAssembled(_) => "host-assembled:other".to_owned(),
        DeviceKind::Indeterminate { .. } => "indeterminate".to_owned(),
    }
}

/// Ensure the runtime directory exists with ADR-0055's mode, owned by this
/// process (root in production). Creates it `0711` if absent; never
/// changes an existing directory's mode — the transport's check refuses a
/// wrong one.
///
/// # Errors
///
/// [`TransportRefusal::Io`] if the directory cannot be created.
pub fn ensure_directory(directory: &Path) -> Result<(), TransportRefusal> {
    if !directory.exists() {
        fs::create_dir(directory).map_err(|e| TransportRefusal::Io {
            operation: "create runtime directory",
            kind: format!("{:?}", e.kind()),
        })?;
        fs::set_permissions(directory, fs::Permissions::from_mode(SOCKET_DIRECTORY_MODE)).map_err(
            |e| TransportRefusal::Io {
                operation: "set runtime directory mode",
                kind: format!("{:?}", e.kind()),
            },
        )?;
    }
    Ok(())
}

/// Run the helper: ensure the directory, create the endpoint for the uid
/// (a pre-existing node means another helper serves this user —
/// [`LaunchRefusal::AlreadyServed`], exit 0 for the launcher to connect
/// to it), then accept, serve one request per connection, and exit when
/// idle. The idle watchdog removes the node this process made before
/// exiting, so a later launch can serve again.
///
/// # Errors
///
/// [`LaunchRefusal`].
pub fn run(config: &Config, audit: &mut dyn AuditSink) -> Result<(), LaunchRefusal> {
    ensure_directory(&config.directory).map_err(LaunchRefusal::Endpoint)?;
    if already_served(&config.directory, config.uid) {
        return Err(LaunchRefusal::AlreadyServed);
    }
    let endpoint = Endpoint::create(
        &config.directory,
        AuthorizingUser(config.uid),
        config.timeouts,
    )
    .map_err(LaunchRefusal::Endpoint)?;
    audit.record(AuditEvent::Started { uid: config.uid });
    let last_activity = Arc::new(AtomicU64::new(now_secs()));
    spawn_idle_watchdog(
        Arc::clone(&last_activity),
        config.idle_seconds,
        endpoint.path().to_path_buf(),
    );
    let backend = SystemBackend::new(config.uid, &config.build);
    let local = Handshake::local(&config.build);
    loop {
        match endpoint.accept(&local) {
            Ok(mut conn) => {
                last_activity.store(now_secs(), Ordering::SeqCst);
                let creds = conn.peer().credentials();
                audit.record(AuditEvent::Admitted {
                    uid: creds.uid,
                    pid: creds.pid,
                });
                if let Err(refusal) = serve_connection(conn.stream(), &backend, audit) {
                    audit.record(AuditEvent::ConnectionRefused {
                        reason: refusal.to_string(),
                    });
                }
                last_activity.store(now_secs(), Ordering::SeqCst);
            }
            Err(refusal) => {
                audit.record(AuditEvent::ConnectionRefused {
                    reason: refusal.to_string(),
                });
            }
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// HLP-005: exit when idle. `accept` blocks, so the watchdog is a thread
/// that checks the last-activity clock; on expiry it removes the node this
/// process made (the endpoint's `Drop` does not run through
/// `process::exit`) and exits 0. The audit line is written through a
/// shared sink the watchdog owns a handle to.
fn spawn_idle_watchdog(last_activity: Arc<AtomicU64>, idle_seconds: u64, node: PathBuf) {
    let shared: Arc<Mutex<Option<FileAudit>>> = Arc::new(Mutex::new(FileAudit::open(None).ok()));
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let idle = now_secs().saturating_sub(last_activity.load(Ordering::SeqCst));
            if idle >= idle_seconds {
                if let Ok(mut guard) = shared.lock()
                    && let Some(sink) = guard.as_mut()
                {
                    sink.record(AuditEvent::IdleExit { idle_seconds: idle });
                }
                let _ = fs::remove_file(&node);
                std::process::exit(0);
            }
        }
    });
}
