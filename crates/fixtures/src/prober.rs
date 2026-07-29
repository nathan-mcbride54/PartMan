//! What a real prober reports for each fixture, recorded so it can regress.
//!
//! `crates/fixtures/src/evidence.rs` proves a fixture's bytes have the structure
//! its rationale claims. That is necessary and not sufficient: a structure this
//! project believes is correct can still be one `libblkid` declines to name, and
//! **a fixture a real prober does not recognize proves nothing.** Two of the
//! signature writers here were undetectable until their checksums were
//! reproduced, and neither the format documentation nor this crate's own tests
//! could have said so.
//!
//! Until now that check was manual: someone ran `blkid` by hand, read the
//! output, and wrote a table into a document. The project review recorded it as
//! an open finding — "real-prober acceptance is manual, not regression-protected"
//! — and it is the last place in this work package where an important property
//! rests on a human having looked once.
//!
//! This module records what `libblkid` 2.41 reported on 2026-07-28, and
//! `cargo xtask probe` re-runs the tools and compares. It needs Linux, so it is
//! not part of `cargo xtask ci`; CI runs it as its own job.
//!
//! # Why the comparison is a chosen subset rather than the whole output
//!
//! `blkid` emits keys that vary with its version, and the fixtures are probed on
//! at least two: `libblkid` 2.41 in the development environment and whatever
//! `ubuntu-24.04` ships in CI. Comparing every key exactly would fail on a
//! toolchain upgrade rather than on a fixture regression, and a check that
//! fails for the wrong reason gets disabled rather than fixed.
//!
//! So the comparison is exact over the facts this project actually depends on —
//! the format name, the identity, the label, the partition-table kind, and the
//! full set of signature offsets `wipefs` enumerates — and silent about the
//! rest. Each of those is a fixture property rather than a formatting choice.
//! The `wipefs` set is compared in **both** directions, because a fixture
//! quietly gaining a signature is as much a regression as losing one.

use std::collections::{BTreeMap, BTreeSet};

/// What a prober must say about one fixture.
pub struct ProberExpectation {
    /// The fixture, by catalogue name.
    pub fixture: &'static str,
    /// `ID_FS_TYPE`, or `None` if the fixture carries no file-system-layer
    /// signature.
    pub fs_type: Option<&'static str>,
    /// `ID_FS_UUID`.
    pub fs_uuid: Option<&'static str>,
    /// `ID_FS_LABEL`.
    pub fs_label: Option<&'static str>,
    /// `ID_PART_TABLE_TYPE`.
    pub part_table_type: Option<&'static str>,
    /// Every `(offset, type)` row `wipefs -n` lists, in offset order.
    pub signatures: &'static [(u64, &'static str)],
    /// Why this is what it is — written for the entries where the answer is not
    /// the one a reader would predict from the fixture's name.
    pub note: &'static str,
}

/// What a probe run actually observed.
#[derive(Debug, Default)]
pub struct Observation {
    /// `KEY=VALUE` pairs from `blkid -p -o udev`.
    pub udev: BTreeMap<String, String>,
    /// `(offset, type)` rows from `wipefs -n`.
    pub signatures: BTreeSet<(u64, String)>,
}

/// Every recorded prober expectation.
///
/// Measured 2026-07-28 with `libblkid` 2.41.0 (util-linux 2.41, 18 March 2025)
/// on Debian under WSL2, probing the generated images as **regular files**.
/// SAFE-001 permits nothing else at Tier 1, and that limitation is load-bearing
/// for two of the notes below.
#[must_use]
pub fn expectations() -> Vec<ProberExpectation> {
    let mut all = table_expectations();
    all.extend(signature_expectations());
    all
}

/// What the probers report for the partition-table fixtures.
fn table_expectations() -> Vec<ProberExpectation> {
    let mut all = gpt_expectations();
    all.extend(other_table_expectations());
    all
}

/// The GPT family, which is where every surprising answer lives.
fn gpt_expectations() -> Vec<ProberExpectation> {
    vec![
        ProberExpectation {
            fixture: "gpt-basic-512.img",
            fs_type: None,
            fs_uuid: None,
            fs_label: None,
            part_table_type: Some("gpt"),
            signatures: &[(0x1fe, "PMBR"), (0x200, "gpt"), (0x3f_fe00, "gpt")],
            note: "The baseline. Both GPT copies and the protective MBR are enumerated.",
        },
        ProberExpectation {
            fixture: "gpt-basic-4kn.img",
            fs_type: None,
            fs_uuid: None,
            fs_label: None,
            part_table_type: Some("PMBR"),
            signatures: &[(0x1fe, "PMBR")],
            note: "PMBR, not gpt — and that is correct, not a defect. Probing a regular file, \
                   libblkid assumes 512-byte logical sectors, looks for EFI PART at 0x200, finds \
                   the zero padding of the 4096-byte protective-MBR sector, and falls back to \
                   the PMBR it did find. The 4Kn table at 0x1000 is never read. IMG-011 evidence \
                   therefore cannot come from file-based probing at all; it needs a loop device \
                   with an explicit sector size, which is privileged and so Tier 2. Recorded so \
                   nobody 'fixes' the fixture into a 512-byte table.",
        },
        ProberExpectation {
            fixture: "gpt-invalid-primary-valid-backup-512.img",
            fs_type: None,
            fs_uuid: None,
            fs_label: None,
            part_table_type: Some("gpt"),
            signatures: &[(0x1fe, "PMBR"), (0x3f_fe00, "gpt")],
            note: "blkid reports an ordinary `gpt`: it recovers silently from the backup and \
                   says nothing about the damage. Only the wipefs offset list shows the primary \
                   copy is missing. A client reading udev cannot tell a healthy disk from one \
                   running on its backup header.",
        },
        ProberExpectation {
            fixture: "gpt-conflicting-tables-512.img",
            fs_type: None,
            fs_uuid: None,
            fs_label: None,
            part_table_type: Some("gpt"),
            signatures: &[(0x1fe, "PMBR"), (0x200, "gpt"), (0x3f_fe00, "gpt")],
            note: "IDENTICAL to gpt-basic-512 from both tools, and that is the finding, not an \
                   oversight. Two independently valid tables describing different partitions is \
                   ADR-C3's definition of a table that parses ambiguously, and neither interface \
                   represents it; ID_FS_AMBIVALENT does not fire. So ADR-C3's `Indeterminate` is \
                   not observable through the interface an unprivileged client reads. Filed as \
                   SI-35. If this row ever stops matching gpt-basic-512, libblkid has gained a \
                   way to tell them apart and SI-35 narrows sharply — so the equality is worth \
                   watching rather than merely recording.",
        },
        ProberExpectation {
            fixture: "gpt-missing-backup-512.img",
            fs_type: None,
            fs_uuid: None,
            fs_label: None,
            part_table_type: Some("gpt"),
            signatures: &[(0x1fe, "PMBR"), (0x200, "gpt")],
            note: "One gpt row, not two: the backup copy is genuinely gone. This row is what \
                   would have caught the fixture's earlier defect, where only the backup header \
                   sector was erased and 16 KiB of entry array survived.",
        },
        ProberExpectation {
            fixture: "hybrid-mbr-gpt-512.img",
            fs_type: None,
            fs_uuid: None,
            fs_label: None,
            part_table_type: Some("gpt"),
            signatures: &[(0x1fe, "PMBR"), (0x200, "gpt"), (0x3f_fe00, "gpt")],
            note: "Plain `gpt`, with the MBR reported as merely protective. The fixture carries \
                   an ordinary 0x0c entry aliasing the ESP's exact extent, and libblkid sees the \
                   0xEE entry first and never mentions the conflict. INV-003's hybrid-table \
                   detection therefore cannot be delegated to libblkid, and SI-27's collision \
                   family is not observable through this interface.",
        },
    ]
}

/// Blank media, MBR, and APM.
fn other_table_expectations() -> Vec<ProberExpectation> {
    vec![
        ProberExpectation {
            fixture: "blank-512.img",
            fs_type: None,
            fs_uuid: None,
            fs_label: None,
            part_table_type: None,
            signatures: &[],
            note: "Nothing, from both tools. ADR-C3's positively-observed-absent state is the \
                   one table state a client can distinguish from the others.",
        },
        ProberExpectation {
            fixture: "mbr-basic-512.img",
            fs_type: None,
            fs_uuid: None,
            fs_label: None,
            part_table_type: Some("dos"),
            signatures: &[(0x1fe, "dos")],
            note: "`dos` rather than `PMBR`, which is the distinction the fixture's claim also \
                   enforces structurally: no entry may be type 0xEE.",
        },
        ProberExpectation {
            fixture: "apm-basic-512.img",
            fs_type: None,
            fs_uuid: None,
            fs_label: None,
            part_table_type: Some("mac"),
            signatures: &[(0x0, "mac")],
            note: "Named `mac`, at offset 0 rather than 0x1fe — the driver descriptor is the \
                   first block, where the other schemes put a boot signature at the end of it. \
                   A reader that assumes little-endian passes every other fixture and fails \
                   only here, which is why this one is in the catalogue at all.",
        },
    ]
}

/// What the probers report for the on-disk-signature fixtures.
fn signature_expectations() -> Vec<ProberExpectation> {
    vec![
        ProberExpectation {
            fixture: "luks2-whole-disk-512.img",
            fs_type: Some("crypto_LUKS"),
            fs_uuid: Some("5f1c2d3e-4a5b-6c7d-8e9f-0a1b2c3d4e5f"),
            fs_label: None,
            part_table_type: None,
            signatures: &[(0x0, "crypto_LUKS")],
            note: "FS-004 LUKS and LIN-003. The UUID is the field an earlier writer put inside \
                   the 48-byte label, leaving the fixture with none; blkid reported nothing for \
                   it then and reports it now.",
        },
        ProberExpectation {
            fixture: "lvm2-pv-orphan-512.img",
            fs_type: Some("LVM2_member"),
            fs_uuid: Some("pvuuid-0000-0000-0000-0000-0000-000000"),
            fs_label: None,
            part_table_type: None,
            signatures: &[(0x218, "LVM2_member")],
            note: "Detected only because the label's own CRC is reproduced — a nibble-wise \
                   variant, not CRC-32. libblkid verifies it before reporting LVM2_member, so a \
                   label carrying LABELONE and a zero checksum is detected as nothing at all. \
                   This row is the external anchor for that algorithm.",
        },
        ProberExpectation {
            fixture: "mdraid-1.2-member-512.img",
            fs_type: Some("linux_raid_member"),
            fs_uuid: Some("62fc041a-4333-f945-a326-3e563a464412"),
            fs_label: Some("pm:0"),
            part_table_type: None,
            signatures: &[(0x1000, "linux_raid_member")],
            note: "FS-004 Linux RAID and LIN-005. Without the folded-sum checksum wipefs still \
                   lists the superblock, because it enumerates magic matches, while `blkid -p` \
                   reports nothing, because it validates. This row anchors that algorithm.",
        },
        ProberExpectation {
            fixture: "ext4-with-stale-mdraid-090-512.img",
            fs_type: Some("linux_raid_member"),
            fs_uuid: Some("fb2871eb-405c-788b-e2c6-fb8cfe3b5444"),
            fs_label: None,
            part_table_type: None,
            signatures: &[(0x438, "ext4"), (0x3f_0000, "linux_raid_member")],
            note: "The asymmetry the protection model turns on, and the reason this table is \
                   worth automating. wipefs reports BOTH signatures; `blkid -p -o udev` — the \
                   form udev's builtin uses, and so what an unprivileged client reads from the \
                   udev database — reports exactly ONE, and it is the STALE one. \
                   ID_FS_AMBIVALENT does not fire. The single expected fs_type here being \
                   `linux_raid_member` rather than `ext4` is the whole finding; if it ever \
                   becomes `ext4` or gains an ambivalence key, Part 5's conclusion and SI-34 \
                   both need re-reading.",
        },
    ]
}

/// Compare one observation against its expectation.
///
/// Returns every disagreement rather than the first, so one run says everything
/// that changed instead of one thing at a time.
#[must_use]
pub fn compare(expected: &ProberExpectation, observed: &Observation) -> Vec<String> {
    let mut disagreements = Vec::new();

    for (key, wanted) in [
        ("ID_FS_TYPE", expected.fs_type),
        ("ID_FS_UUID", expected.fs_uuid),
        ("ID_FS_LABEL", expected.fs_label),
        ("ID_PART_TABLE_TYPE", expected.part_table_type),
    ] {
        let actual = observed.udev.get(key).map(String::as_str);
        if actual != wanted {
            disagreements.push(match (wanted, actual) {
                (Some(wanted), Some(actual)) => {
                    format!("{key}: expected {wanted:?}, prober said {actual:?}")
                }
                (Some(wanted), None) => {
                    format!("{key}: expected {wanted:?}, prober reported nothing")
                }
                (None, Some(actual)) => {
                    format!("{key}: expected nothing, prober said {actual:?}")
                }
                (None, None) => unreachable!("equal values cannot disagree"),
            });
        }
    }

    // Both directions. A fixture gaining a signature is as much a regression as
    // one losing it, and the multi-signature finding this catalogue exists to
    // support is precisely a statement about which signatures are present.
    let wanted: BTreeSet<(u64, String)> = expected
        .signatures
        .iter()
        .map(|(offset, kind)| (*offset, (*kind).to_owned()))
        .collect();
    for (offset, kind) in wanted.difference(&observed.signatures) {
        disagreements.push(format!(
            "wipefs: expected {kind} at {offset:#x}, and it is not there"
        ));
    }
    for (offset, kind) in observed.signatures.difference(&wanted) {
        disagreements.push(format!(
            "wipefs: unexpected {kind} at {offset:#x}, which nothing recorded"
        ));
    }

    disagreements
}

/// Parse `blkid -p -o udev` output.
#[must_use]
pub fn parse_udev(output: &str) -> BTreeMap<String, String> {
    output
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.trim().to_owned(), value.trim().to_owned()))
        .collect()
}

/// Parse `wipefs -n --output OFFSET,TYPE` output.
///
/// Skips the header row if the tool emitted one, which it does when there is at
/// least one signature and omits when there is none.
#[must_use]
pub fn parse_wipefs(output: &str) -> BTreeSet<(u64, String)> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("OFFSET"))
        .filter_map(|line| {
            let (offset, kind) = line.split_once(char::is_whitespace)?;
            let digits = offset.strip_prefix("0x")?;
            let offset = u64::from_str_radix(digits, 16).ok()?;
            Some((offset, kind.trim().to_owned()))
        })
        .collect()
}

#[cfg(test)]
mod tests;
