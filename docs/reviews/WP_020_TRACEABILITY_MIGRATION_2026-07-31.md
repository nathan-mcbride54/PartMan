# WP-020 traceability migration ledger

Date: 2026-07-31

This ledger records the zero-loss conversion of
`docs/traceability/WP-020.md` from hand-maintained prose to generated evidence.
It is a migration record, not a second traceability source. The generated
document is authoritative for current requirement-to-evidence relationships;
durable design history, qualifications, measurements, and residual risks remain
in `docs/work-packages/WP-020.md`.

## Frozen source

- Source revision:
  `c0e7dda14ee4c9e3bd43a4d906baed7868c120aa`
- Source blob:
  `ce7762463cca782b80abf35a7fefce98530b59da`
- Source path: `docs/traceability/WP-020.md`
- Source spec version: 4.0.0
- Source scope: increments 1 through 1f and 2a through 2d

Line references below address that frozen blob. They intentionally do not point
at the generated replacement, whose ordering is a function of validated source
annotations.

## Disposition vocabulary

- **Generated annotation**: the relationship now lives beside the named Rust
  test and is rendered by `cargo xtask traceability`.
- **Structured evidence**: a package-wide path, command, or shared test is
  declared in the typed block in `docs/work-packages/WP-020.md`.
- **Durable package record**: historical measurements, limitations, rejected
  approaches, and residual risks remain in the work-package narrative.
- **Corrected**: the old wording exceeded the evidence and has been narrowed in
  the durable record and generated claim.

## Header and scope metadata

The old header at lines 3–8 is accounted for explicitly:

| Frozen field | Disposition |
| --- | --- |
| Spec version 4.0.0 | Preserved as frozen-source metadata above. The generated replacement validates requirement names against the current normative specification instead of copying a version string that can drift. |
| WP-020 increments 1–1f and 2a–2d | Preserved as frozen-source scope above. Current delivery status, including increment 1g, lives in `docs/work-packages/WP-020.md` and WP-020’s README row. |
| Requirements affected: SAFE-001, SAFE-005, SAFE-007, INV-003, IMG-011 | Replaced by the complete requirement set mechanically derived from current annotations and typed evidence. The five-field list was historical and incomplete once the same evidence also named FS-004, LIN-003, LIN-005, PART-001, UI-010, SAFE-009, and numeric sections. |
| Safety constraints preserved: SAFE-001, SAFE-002, SAFE-005, SAFE-007, SAFE-009 | Current positive evidence is generated for every constraint the package directly exercises. SAFE-002 remains a durable boundary: Tier 1 is unprivileged, no destructive suite exists, and no claim here reaches a block device or privileged storage operation. |

## Primary evidence table

Every row in the old table at lines 12–61 has a destination below.

| Old line(s) and evidence | Disposition |
| --- | --- |
| 12: fixture layout and catalogue source paths | **Structured evidence** for Sections 11.3 and 16, with `cargo xtask fixtures`. |
| 13: `/tests/generated/` ignored by Git | **Corrected and strengthened.** `generated_fixture_output_is_ignored_by_the_repository` requires the repository `.gitignore` rule, not a coincidental global ignore. `a_force_added_generated_fixture_is_refused_by_the_ownership_gate` then stages such a path with `git add -f` in a temporary repository and requires the real ownership gate to refuse it by name. Ordinary staging ignores the path, while a force-added path is refused by CI. The old wording “no generated image can reach a commit” exceeded what `.gitignore` alone can prove. |
| 14: `generation_is_deterministic`, `regenerating_reproduces_identical_bytes` | **Generated annotations** in catalogue tests for deterministic generation and regeneration. |
| 15: `a_derived_guid_is_stable_and_well_formed` | **Generated annotation** in layout tests. |
| 16: `crc32_matches_the_published_check_value` | **Generated annotation** in layout tests; the independent evidence-layer CRC anchor remains separately annotated. |
| 17: GPT headers, CRC, and protective MBR | **Generated annotations** on both layout tests for INV-003. |
| 18: corrupted primary GPT remains signed but fails CRC | **Generated annotation**, with the durable correction that the valid backup makes this recoverable, not `Indeterminate`. |
| 19: independent classification of the three ADR-C3 table states | **Generated annotations** on both catalogue classifier tests. |
| 20: MBR entries | **Generated annotation** on `an_mbr_image_records_its_entries`. |
| 21: big-endian APM | **Generated annotation** on `an_apm_image_is_big_endian`. |
| 22: genuine 4 KiB-sector fixture | **Generated annotation** on `a_4kn_image_has_4096_byte_sectors`. |
| 23: all-zero blank media | **Generated annotation** for PART-001 and INV-003. |
| 24: out-of-bounds fixture writes panic | **Generated annotation** for deterministic generation and fake-success prevention. |
| 25: all three interlock factors authorize | **Generated annotation** on `all_three_factors_together_authorize`. It proves the positive case; rows 26 and 28–35 carry the refusal cases. The claim is acceptance in the unprivileged fixture harness, not permission to enable a destructive tier. |
| 26: no one or two factors suffice | **Generated annotations** on both factor-combination tests. |
| 27: destructive profile is a command-line argument | **Structured evidence** naming the existing shared xtask test; its WP-000 source annotation remains unchanged. |
| 28: empty target list refuses | **Generated annotation** on the exact test. |
| 29: compiled catalogue resists ungenerated, modified, forged, wrongly named, and cross-named files | **Generated annotations** on all five interlock tests. |
| 30: outside-root and traversal targets refuse | **Generated annotations** on both tests. |
| 31: directories and Unix symlinks refuse | **Generated annotations**; the Unix half stays platform-gated. |
| 32: missing target and missing root fail closed | **Generated annotations** on both tests. |
| 33: one bad target refuses the entire request | **Generated annotation**. |
| 34: token and profile require exact matches | **Generated annotations**. The token claim is narrowed to the exact build-derived value; it is not operator provenance. |
| 35: malformed and corrupt manifests refuse | **Generated annotations** in manifest tests. |
| 36: authorization witness has no public constructor | **Generated annotation**; the compile-time boundary remains described in the work-package record. |
| 37: authorization holds the verified object | **Generated annotation** plus **durable package record** under the increment-2 preconditions and 2d residuals. |
| 38: object verification survives path deletion | **Generated annotation** plus the mutation history retained in the durable package record. |
| 39: verified handle is consumed once; compile-fail doctest | **Generated annotation** on the runtime handoff test; the non-cloneable compile-fail proof remains documented under the increment-2 preconditions and remains part of the crate test suite. |
| 40: object shape cannot prove root membership | **Generated annotation explicitly recording a limit**, with the containment design retained in the durable package record. |
| 41: Unix final-component no-follow race | **Generated platform-gated annotation** on the scheduled seam test. |
| 42: Unix intermediate-component/root swap | **Generated platform-gated annotation**; inode control and `openat` design remain in the durable record. |
| 43: Windows local-volume root swap | **Generated platform-gated annotation** narrowed to a locally served volume; positive-control and mutation history remain in the durable record. |
| 44: Windows root handle alone blocks root rename | **Generated platform-gated annotation**; the rejected vacuous test is retained in the durable record. |
| 45: outside hard-link alias refuses | **Generated annotation**; live-defect reproduction, exact refusal, positive control, and the rejected disjunctive assertion remain durable. |
| 46: Windows nonlocal root refuses | **Generated platform-gated annotation**; NTFS/ReFS/SMB/WSL measurements and unmeasured drivers remain durable. |
| 47: Windows junction replacement refuses | **Generated platform-gated annotation**; it does not claim `FILE_FLAG_OPEN_REPARSE_POINT` coverage. |
| 48: Windows file-symlink replacement refuses as irregular | **Generated platform-gated annotation**; privilege-dependent execution and the removed unreachable guard remain durable. |
| 49: post-authorization hard-link residual | **Generated annotation explicitly recording a boundary, not a guarantee**. |
| 50: lifetime-free Windows handle wrapper has one call site | **Generated annotation** for SAFE-009; the durable record identifies this as a textual guard. |
| 51: pre-open object substitution refuses on every platform | **Generated annotation**. |
| 52: delivered handle starts at offset zero | **Generated annotation**. |
| 53: refusal messages identify a next step | **Generated annotation** for UI-010. |
| 54: unavailable destructive tiers refuse | **Structured evidence** naming the existing shared xtask test and Tier-1 command. This is fail-closed evidence, not a Tier-2 or Tier-3 implementation. |
| 55: workspace `unsafe_code = "deny"` | **Structured evidence** through the fixtures manifest and `cargo xtask ci`; the generated claim is limited to this crate inheriting the workspace policy. |
| 56: LVM2 marker and checksum structure | **Generated annotations** on both signature tests. External recognition is established separately by the Linux prober evidence. |
| 57: LUKS2 big-endian version | **Generated annotation** for FS-004 and LIN-003. |
| 58: mdraid 1.2 offset and checksum | **Generated annotations** for FS-004 and LIN-005. |
| 59: mdraid 0.90 trailing placement | **Generated annotation**. |
| 60: live ext4 plus stale mdraid signatures | **Generated annotations** on both structural tests. SI-34 remains open; these tests preserve its measured premise only. |
| 61: ext4 prober-offset baseline | **Generated annotation**. |

## Verification and known-gap narrative

The old verification paragraphs at lines 63–73 are split deliberately:

- the command relationships are **structured evidence** for
  `cargo xtask ci`, `cargo xtask test --tier 1`, `cargo xtask fixtures`, and
  `cargo xtask probe`;
- the real-filesystem, unprivileged test-harness design remains in
  `docs/work-packages/WP-020.md`;
- no generated claim says that these tests address a host block device, launch a
  privileged storage operation, or enable a destructive suite.

The old blanket statement that the crate “launches no process” is **corrected**,
not preserved. The library’s non-test code launches none, but Windows race-test
setup may run unprivileged `cmd /c mklink` to create a junction, and
`cargo xtask probe` launches `blkid` and `wipefs` over generated regular files
on Linux. None of those paths opens a storage device or performs a privileged
storage operation.

Every old known-gap bullet at lines 75–105 remains durable:

1. No Tier-2 suite or real destructive operation exists. Tier 2 and Tier 3
   continue to refuse.
2. Unix symlink evidence remains platform-gated; Windows symlink creation may
   visibly skip without Developer Mode or the required privilege.
3. Windows containment depends on filesystem rename/share semantics. UNC roots
   refuse, but drive-letter filesystems implemented by WinFsp, Dokan, sshfs-win,
   Samba, NFS, FAT32, and exFAT remain unmeasured unless explicitly stated in
   the package.
4. ReFS identity is evidenced by observed runs, not proved unique; the stronger
   `FILE_ID_INFO` design remains future work.
5. Default Windows tests exercise `%TEMP%`; `PARTMAN_TEST_ROOT` measurements do
   not convert CI into evidence about every target filesystem.
6. Fixtures cover recognized LUKS2, LVM2, mdraid, and selected table/signature
   cases, not BitLocker, Storage Spaces, LDM, a recognized ZFS member, or
   mountable filesystem contents.
7. No golden-image domain integration exists while the WP-010 domain work
   remains blocked by SI-34 and SI-35.

## Signature and external-prober section

The old narrative and observed-format table at lines 107–131 are retained in
the work package under “Increment 1b” and “Increment 1e.” Current relationships
are generated from signature and prober tests. The four observed results remain
measurements against regular files on Linux:

- LUKS2 → `crypto_LUKS`
- LVM2 PV → `LVM2_member`
- mdraid 1.2 → `linux_raid_member` from util-linux 2.41 onward, with recorded
  silence below that version
- ext4 plus stale mdraid 0.90 → single answer `linux_raid_member`, while
  `wipefs` enumerates both

Every row in the old prober table at lines 134–141 maps as follows:

| Old evidence | Disposition |
| --- | --- |
| `cargo xtask probe` and its Linux CI job | **Structured evidence** for FS-004/SAFE-001/SAFE-005. Workflow ownership remains WP-000; WP-020 claims the command and prober implementation, not the workflow file. |
| `every_fixture_has_a_recorded_prober_expectation` | **Generated annotation** for two-way catalogue/expectation exhaustiveness. |
| `the_recorded_table_matches_a_real_probe_run` | **Generated annotation** tying the table to the captured 2.41 run. |
| `the_comparison_is_capable_of_failing_in_every_direction` | **Generated annotation** for lost, changed, missing, and unexpected signatures. |
| `the_conflicting_and_healthy_tables_are_recorded_as_indistinguishable` | **Generated annotation** preserving the SI-35 premise. It does not resolve SI-35 or turn prober silence into a decision. |
| `the_stale_signature_is_the_one_the_single_answer_interface_reports`, `the_capture_shows_both_signatures_where_only_one_is_reported` | **Generated annotations** preserving the SI-34 premise. They do not choose the product’s aggregation behavior. |
| `the_parsers_read_the_shapes_the_tools_actually_emit` | **Generated annotation** for captured valid and genuinely empty shapes. |
| `the_prober_check_refuses_where_its_tools_do_not_exist` | **Generated annotation** in the shared xtask source, routed explicitly to WP-020. |

The old “ZFS is not in the catalogue” qualification at lines 153–163 remains in
the package. The structural `zfs_labels_are_written_at_both_ends` annotation
does not claim external recognition.

## Increment 1f remediation

Every row in the old table at lines 147–151 has a live destination:

| Old evidence | Disposition |
| --- | --- |
| `a_line_the_parser_cannot_read_is_refused_rather_than_dropped` | **Generated annotation** for fail-closed parsing. |
| `a_changed_output_shape_cannot_pass_as_a_blank_fixture` | **Generated annotation** proving foreign output cannot become a valid empty observation. |
| strict UTF-8 in `probe_output` | **Generated annotation** on `invalid_utf8_from_a_prober_is_refused_without_substitution`. The test drives the real `probe_output` call with a subprocess that emits raw `0xff`, so a regression at the call site—not only inside the decoder helper—fails. |
| `a_foreign_file_named_manifest_does_not_authorize_deletion` | **Generated annotation** in catalogue tests. |
| Unix `a_manifest_symlink_does_not_authorize_deletion` | **Generated platform-gated annotation** in catalogue tests. |

The process-id/per-process-counter sandbox fix, mutation history, and pruning
ownership rationale at lines 143–165 remain in the work package’s Increment 1f
section. They are implementation history rather than additional current
requirement relationships.

## Review-audit corrections

The defect table and follow-up evidence at lines 167–187 remain durable under
“Increment 1c.” Its four evidence rows map to generated annotations:

- `the_090_set_uuid_occupies_its_four_non_adjacent_words`
- `the_luks2_fields_do_not_land_inside_the_label`
- `the_ext4_block_count_matches_the_device_it_is_written_to`
- `a_fixture_copy_in_a_subdirectory_is_refused`

The statement that the Part 5 asymmetry survived these structural corrections
is retained, but only as the measured Linux regular-file premise. It does not
resolve SI-34.

## Increment 1d evidence layer

The mutation narrative at lines 189–211 remains under “Increment 1d.” Every row
in the old evidence table at lines 214–225 maps below.

| Old evidence | Disposition |
| --- | --- |
| `every_fixture_has_a_claim_and_every_claim_has_a_fixture` | **Generated annotation** for two-way claim/catalogue exhaustiveness. |
| `every_fixture_satisfies_its_claim` | **Generated annotation** binding catalogue output to computed claims. |
| `every_claim_rejects_a_mutation_that_breaks_it_and_says_why` | **Generated annotation** for negative capability and diagnostic specificity. |
| `every_fixture_has_at_least_one_mutation_that_must_be_caught` | **Generated annotation** preventing mutation-free claims. |
| `generation_refuses_a_fixture_that_no_longer_supports_its_rationale` | **Generated annotation** proving the generation gate is load-bearing. |
| `a_fixture_with_no_registered_claim_is_refused_rather_than_passed` | **Generated annotation** for fail-closed missing registration. |
| `two_fixtures_may_not_claim_one_identity` | **Generated annotation** for cross-fixture identity uniqueness. |
| pinned LVM2 and mdraid checksum values | **Generated annotations** on both external-anchor tests. |
| published CRC-32 check value | **Generated annotation** on the independent evidence-layer implementation. |
| three pairs of independent checksum implementations | **Generated annotations** on all three agreement tests; their claims explicitly rely on the pinned anchors for external meaning. |
| `expect_partitions_usable`, exercised by mutations | **Generated annotations** on the partition-bound mutation tests; the durable package record preserves the oversized-partition defect history. |
| exact catalogue-length assertion in `every_generated_fixture_authorizes` | **Generated annotation** on that interlock test; exact equality remains in code. |

The two corrected-claim paragraphs at lines 227–240 and the remaining
root-of-trust boundary at lines 242–248 stay in the work package. In particular:

- the corrupt-primary fixture is recoverable from backup, not `Indeterminate`;
- `authorization_cannot_be_forged_outside_this_module` proves constructor
  confinement, while the surrounding suite—not that one test—detects a
  short-circuited target verifier;
- `catalogue::expected()` deliberately stays I/O-free and does not call
  `evidence::verify`; evidence gates supported generation, while tests make a
  self-consistent but purpose-losing catalogue change fail.

## Documentation correction made during migration

The work package already stated correctly that the disposable token is a pure
function of public source and proves only presentation of the exact
build-derived value. A later paragraph contradicted that by calling it an
“operator-intent proof” and asserting that the operator ran the generator. Live
crate documentation repeated the contradiction by calling the three proofs
independent and assigning the token generator history and deliberate intent.
Those statements in `docs/work-packages/WP-020.md`,
`crates/fixtures/src/lib.rs`, `crates/fixtures/src/manifest.rs`, and
`crates/fixtures/src/interlock.rs` are corrected in this increment. The
generated SAFE-007 claims now say only what the code establishes:

- exact build-derived token match;
- explicit command-line profile;
- verified generated-fixture object;
- accident friction against ambient-state-only invocation.

They make no claim about operator identity, provenance, generator execution, or
independence of the token factor.

## Result

No old evidence row, observed result, correction, limitation, or residual risk
is deleted without a destination. Current relationships become mechanically
generated; historical reasoning and qualifications remain durable; statements
that exceeded the evidence are narrowed rather than copied forward.
