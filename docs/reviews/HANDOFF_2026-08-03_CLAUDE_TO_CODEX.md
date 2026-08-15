# Handoff — WP-020 increment 2e acceptance, 2026-08-03

**From:** Claude (Opus 5), driving the Proxmox sitting from the Windows workstation.
**To:** Codex, for review and merge of PR **#119**.
**Ask:** verify the items in §4, then merge #119 if they hold. Do not merge on
green CI alone — §4 lists properties CI cannot check.

> This file is an **untracked local handoff artifact**, following the precedent
> the 2026-08-02 handoff set. Its path `docs/reviews/**` belongs to WP-000, not
> WP-020, so **do not stage it in the WP-020 change** —
> `verify-change-ownership` will reject a WP-020-trailered commit that touches
> it. If Nate wants it tracked, it needs its own WP-000 commit.

---

## 1. What changed, in one line

Repository issue **#94** is closed by a real passing acceptance in a disposable
Proxmox VM; increment 2e is Delivered; increment 2 is unblocked and still
unbuilt.

## 2. Commits on `codex/wp020-linux-loop-binding`

| Commit | What |
| --- | --- |
| `2dbf601` | Pre-existing. The implementation. **Unchanged by this session.** |
| `d8d9b40` | The acceptance record, delivery rows, precondition 4 closed, test-tiers/README/CHANGELOG |
| `c75b340` | Merge of `origin/main` (was CONFLICTING; only CHANGELOG conflicted) |
| `186d159` | Re-took the acceptance on the merged tree and cited it |
| `be48faf` | The checkable stopping condition (see §3) |
| `c9d56d0` | Teardown proof |
| `b38ed4a` | Transcript digests bound into the record |

Everything after `2dbf601` is **documentation only**. No `.rs`, no manifest, no
lockfile changed in this session.

## 3. The acceptance result

`cargo xtask test --tier 2 --profile destructive --acceptance linux-loop-read-only`
passed **four times**, with identical harness output and identical fixture
digests every time:

- 3× on `2dbf601`
- 1× on `c75b340` — **the cited run**, taken after `origin/main` brought
  `apps/cli/src/{doctor,inspect,tests}.rs` changes in

Reported each time: `configured_legs=2`, `required_configuration_verified=true`,
`adversarial_rebind_detected=true`, `adversarial_observation_discarded=true`,
`detachments_confirmed=2`, `partition_teardown_confirmed=true`,
`initial_fixture_hashes_matched_catalogue=true`, `fixture_hashes_unchanged=true`.

Four negative controls refused in the same session: generic destructive Tier 2
(authorized 13 targets, still refused — no suite registered), Tier 3, the
acceptance without `--profile destructive`, and the acceptance without
`PARTMAN_DISPOSABLE_TOKEN`.

**Environment:** Proxmox VE 9.2.4; stock Ubuntu 22.04.5 cloud image, base image
SHA-256 `bf4be84e…f945` verified against Canonical's published `SHA256SUMS`
before first boot; kernel 5.15.0-186-generic; euid 0; no Microsoft markers; no
USB or PCI passthrough; `pre-acceptance` snapshot as the revert boundary.

**Why the fourth run exists, and why it is the cited one.** None of main's
changed files is on the acceptance's code path (`crates/ffi-linux-loop`,
`crates/fixtures`, `tools/xtask`), so the earlier result could have been argued
forward. It was re-measured instead. That rule then recurses — the commit
recording a proof supersedes the tree the proof was taken against — so the
stopping condition is stated as something you can check:

```bash
git diff --name-only c75b340 HEAD
```

**Must list Markdown files only.** If it ever reports a non-Markdown path, the
proof is stale and the acceptance must be re-taken before the record is relied
on. Please actually run it; it is the load-bearing check of this whole handoff.

## 4. What to verify before merging

CI cannot check any of these. Each is a claim I made about my own work.

1. **Run the stopping-condition command above.** Everything else rests on it.
2. **Read `docs/work-packages/WP-020.md` → "Increment 2e acceptance record",
   specifically "What this run does not establish".** Judge whether it
   overstates. I wrote both the runbook and the record, so I am the wrong
   auditor of it. In particular: the guest **was not network-isolated** — it
   held a DHCP address and default route throughout. I originally planned to
   air-gap it and abandoned that; the reasoning is in §5.
3. **Check the increment-2 row still says "unblocked and still not delivered".**
   Closing #94 registers no destructive suite. If any surface now reads as
   though a destructive harness exists, that is a defect.
4. **Check the `snapd` purge is presented as a deviation, not hygiene.** The
   stock cloud image held four squashfs loop devices at first boot; the
   no-other-loop-administrator exclusion cannot hold with snapd running. Purging
   it makes the guest non-stock, which the handoff-preferred "stock Ubuntu
   22.04" wording implies it would be.
5. **Spot-check a transcript against its recorded digest** (§6 has the paths and
   hashes). The digests are in `WP-020.md` under "Transcript custody".
6. **Confirm no WP-035-owned surface was edited.** See §7 — two README
   statements are now stale and I deliberately left them alone.

## 5. Judgment calls I made, that you may disagree with

- **Abandoned network isolation.** I had told Nate I would drop the guest's link
  during the run. I did not. The acceptance requires guest isolation, no other
  fixture modifier, and no other loop administrator — not an air gap; that was
  my own addition. Dropping the hypervisor link would have made the run
  unobservable and forced me to guess its duration, bringing the link back
  mid-run and polluting the window. Connectivity is recorded as a fact in the
  transcript. **If you think the weaker claim is unacceptable, the run must be
  redone** — the fix is not a wording change.
- **Ran as root over direct SSH, with the cloud-init account locked.** Forced by
  §8, not preference.
- **Retained the two superseded transcripts** rather than deleting them, so the
  record shows the artifact was revised.
- **Marked increment 2e "Delivered" while the PR is unmerged.** The row
  describes post-merge state, matching how 2a–2d were written. If review fails,
  the row is wrong and must move with it.

## 6. Evidence bundle

Archived outside the repository at:

```text
C:\Users\nmcbr\PartMan-evidence\WP-020-increment-2e-2026-08-03\
```

| Path | Contents |
| --- | --- |
| `00-host-environment.txt` | Proxmox version/node, VMID, base-image digest, full `qm config`, passthrough assertion |
| `guest/`, `guest-run2/`, `guest-run3/` | Runs 1–3 on `2dbf601` |
| `guest-merged/` | **Run 4 on `c75b340` — the cited run** |
| `90-final-vm-config.txt`, `91-snapshots.txt` | VM state captured immediately before destroy |
| `92-teardown-proof.txt` | Post-destroy verification |

Each run directory holds `20-transcript.txt` and `21-transcript.sha256`; runs
1–3 also carry the provisioning log and environment record. Digests are in §3's
table and in `WP-020.md`.

## 7. What I did **not** do, and why

- **Two README statements are now stale and I left them.** Line ~169 (M0.5
  prose) and the **WP-035 row** still say issue #94 is open and that M0.5's
  #94-gated loop criterion is unsatisfied. Both are now wrong. WP-020's grant
  covers its own README row only and excludes another package's row and the
  surrounding prose, so fixing them needs a WP-035 authorization row. **Do not
  fold that into this PR.**
- **Did not mark #119 ready-for-review or merge it.** Deliberate: the merge
  condition here is the property, not the checkmark, and I authored both the
  evidence and the record.
- **The PR body is stale.** It still says "This is intentionally a draft… issue
  #94 and increment 2e remain open until this exact commit passes the registered
  acceptance." That needs rewriting when the PR goes ready.
- **Did not run the WP-035 loop measurement.** Its preregistered protocol
  requires a **fresh** disposable VM *and* that #94's implementation "has landed
  and been reviewed" — i.e. it is gated on merging this PR.

## 8. Operational precondition — the one that will bite you

The acceptance runs `cargo xtask ci` first, and WP-035's
`no_output_in_any_mode_carries_an_environment_value` compares **every**
environment value of ≥6 characters against CLI output.

Two runs failed before the acceptance was reached: first on an injected
`CIUSER=partman`, then on the `SUDO_USER=partman` that `sudo` sets by itself.
Both are coincidental collisions — `partman` is the program's own name and
appears in `help` output — not leaks. **The tripwire behaved correctly.**

**Run this acceptance as root over a direct login, with no `sudo` anywhere in
the chain and no injected variables, and do not name the VM's user, host, or any
whole path component something that appears in CLI output.** Adding an exemption
to the test would have edited the very commit under proof. This is now recorded
in `docs/quality/test-tiers.md`.

## 9. Gate matrix as run

**Local**, on the final tree — Windows: `cargo xtask ci`, `cross-language`
(38/38), `supply-chain`, `verify-change-ownership --base origin/main` (17 paths,
WP-020, `Cargo.lock` regenerated-not-authored). WSL Debian: `cargo xtask ci`,
`cargo xtask probe` (13/13 fixtures).

**GitHub CI on `b38ed4a`'s predecessor `c9d56d0`: 12/12 SUCCESS** — Tier 1 on
ubuntu-24.04 / windows-2025 / macos-15, cross-language parity on all three,
real-prober acceptance, fuzz smoke, supply-chain on all three, GitGuardian.
**Re-check CI on `b38ed4a` before merging**; it was pushed after that run.

**Not run anywhere:** the acceptance on macOS (it is Linux-only by construction
and refuses elsewhere), and any destructive suite (none exists).

## 10. Teardown

VM 9420 destroyed 2026-08-03T20:01:33Z with
`qm stop; qm destroy --purge --destroy-unreferenced-disks 1`. Verified
afterwards rather than inferred from a zero exit: the VM config no longer
exists, no LVM volume matching the VMID remains, and the `pre-acceptance`
snapshot volume is gone.

Two SanDisk USB devices were attached to the **host** throughout. They were
never passed through, never referenced by any VM config, and their partition
layout was unchanged afterwards. They are for a later WP-035 L-matrix sitting
and are **not** currently byte-identical — `sda` has one 32 GB partition, `sdb`
has a hybrid-ISO layout that looks like a live installer. L9 needs two identical
media, so making them so means overwriting both; `sdb` may hold something worth
keeping. That is Nate's call, not a reviewer's.
