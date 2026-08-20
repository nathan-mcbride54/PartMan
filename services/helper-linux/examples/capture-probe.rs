//! The increment-2 Tier-2 instrument: run HLP-002's capture on this host
//! and print what it authored — one line per admitted device (selector,
//! arm, state, scheme), then the snapshot body hash. Root is the intended
//! caller (the helper's own context, SAFE-002 context 1); an unprivileged
//! run refuses at the first window read, which is itself evidence.
//!
//! Output carries selectors, arm names, state names and the hash — no
//! serial, no path, no label (SAFE-006's discipline, kept even in an
//! instrument). Exit 0 when a snapshot was produced; 1 when the capture
//! refused.

fn main() {
    #[cfg(target_os = "linux")]
    {
        use partman_adapter_linux::contract::{SystemContractSource, sysfs_root, udev_root};
        use partman_helper_linux::capture::{DeviceCapture, capture};
        use partman_helper_linux::linux::SystemDeviceReader;
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs());
        // The instrument takes a settled capture: it is not the helper,
        // holds no journal, and no apply can be in flight around it.
        match capture(
            &SystemContractSource,
            &sysfs_root(),
            &udev_root(),
            &SystemDeviceReader,
            now,
            false,
        ) {
            Ok(outcome) => {
                for device in &outcome.devices {
                    match device {
                        DeviceCapture::Authored {
                            selector,
                            state,
                            scheme,
                            hybrid,
                            ..
                        } => println!(
                            "capture {selector} arm=authored state={state} scheme={} hybrid={hybrid}",
                            scheme.unwrap_or("none"),
                        ),
                        DeviceCapture::NamedOnly {
                            selector, withheld, ..
                        } => println!("capture {selector} arm=named-only withheld={withheld:?}"),
                        DeviceCapture::Grouped { selector, .. } => {
                            println!("capture {selector} arm=grouped");
                        }
                        DeviceCapture::NotNamed { selector, why } => {
                            println!("capture {selector} arm=not-named why={why:?}");
                        }
                    }
                }
                println!("snapshot_hash={}", outcome.snapshot_hash);
                println!(
                    "devices={} authored={}",
                    outcome.devices.len(),
                    outcome
                        .devices
                        .iter()
                        .filter(|device| matches!(device, DeviceCapture::Authored { .. }))
                        .count()
                );
            }
            Err(refusal) => {
                eprintln!("capture refused: {refusal:?}");
                std::process::exit(1);
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("capture-probe: this instrument runs on Linux only");
        std::process::exit(1);
    }
}
