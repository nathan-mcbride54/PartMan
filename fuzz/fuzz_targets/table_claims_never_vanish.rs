//! The property that makes the table parser's fail-closed promise a
//! searched property rather than prose.
//!
//! > For any windows and any legal geometry, `classify` either refuses the
//! > caller's contract violation with a typed value or returns a
//! > classification — it never panics — and **a claimed table never
//! > classifies as `Absent`**: if the head carries a protective-MBR 0xEE
//! > entry or a GPT magic at LBA 1, the answer is `Present` or
//! > `Indeterminate`, never blank.
//!
//! The second clause is the parser's load-bearing safety line: `Absent` is
//! ADR-C3's positively-determined state and PART-001's future categorical
//! invariant keys off it, so a hostile byte pattern that smuggled a
//! claimed-but-mangled table into `Absent` would be exactly the
//! unreadable-collapses-into-absent conflation ADR-C4 refused. The unit
//! suite proves it for the catalogue's shapes; this target searches for
//! the shapes nobody thought of.
//!
//! Input layout: byte 0 selects the sector size, bytes 1..9 the total
//! sector count, and the rest splits evenly into head and tail windows
//! (truncated to sector multiples). Everything the engine's 4096-byte cap
//! allows is reachable: single-sector windows, boundary geometries, and
//! every refusal arm.

#![no_main]

use libfuzzer_sys::fuzz_target;
use partman_table_parser::{Geometry, TableState, classify};

fuzz_target!(|data: &[u8]| {
    if data.len() < 9 {
        return;
    }
    let sector_size: u32 = match data[0] % 4 {
        0 => 512,
        1 => 4096,
        2 => 520,
        _ => 0,
    };
    let total_sectors = u64::from_le_bytes(data[1..9].try_into().expect("8 bytes"));
    let rest = &data[9..];
    let split = rest.len() / 2;
    // Align both windows down to sector multiples for the supported sizes,
    // so most executions reach classification; unsupported sizes pass raw
    // slices and exercise the refusal arms instead.
    let align = |slice: &[u8], sector: usize| -> usize {
        if sector == 0 { slice.len() } else { (slice.len() / sector) * sector }
    };
    let sector = if matches!(sector_size, 512 | 4096) {
        sector_size as usize
    } else {
        0
    };
    let head = &rest[..align(&rest[..split], sector)];
    let tail_raw = &rest[split..];
    let tail = &tail_raw[..align(tail_raw, sector)];

    let geometry = Geometry {
        sector_size,
        total_sectors,
    };
    let Ok(classification) = classify(head, tail, geometry) else {
        // A typed refusal is always a correct answer to a broken call.
        return;
    };

    // A claimed table must never classify as Absent.
    let claims_pmbr = sector >= 512
        && head.len() >= 512
        && head[510..512] == [0x55, 0xaa]
        && (0..4).any(|entry| head[446 + entry * 16 + 4] == 0xee);
    let claims_magic = head.len() >= sector + 8 && head[sector..sector + 8] == *b"EFI PART";
    if (claims_pmbr || claims_magic) && classification.state == TableState::Absent {
        panic!("a claimed table classified as Absent: the ADR-C4 conflation, found by search");
    }
});
