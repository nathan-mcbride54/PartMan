# Handoff — 2026-08-15, the r21 sitting and the re-pin at `b8d6a90`

**From:** Claude (Fable 5), the session Nate opened with "take the handoff
from the last agent, determine the next slice and complete it."
**To:** whoever picks this up next.
**Follows:** `HANDOFF_2026-08-15_OPUS_CLEANUP_TO_NEXT.md`, whose §2 named
the slice this session took. Its §3 (the open-issue map) is unchanged and
is not repeated here — read it there.

> `docs/reviews` artifact, committed under WP-000 in its own pull request,
> after the WP-020 re-pin merged. Session records go in their own WP-000
> commit, never bundled with code (the previous handoff's §6.1).

**This session wrote no product code.** It ran one Proxmox sitting and
recorded it. Everything below about the code is a reading of main.

---

## 0. Repository state — verified, not assumed

| Fact | Value |
| --- | --- |
| `main` | **`2761572`** — the merge of PR #375 (the r21 re-pin), on top of `b8d6a90` |
| Spec | **13.0.0** |
| `cargo xtask ci` | **exit 0** on the re-pin commit — 604 annotations, 50 evidence rows, 85 requirements, 640 live tests, 10 generated documents |
| WP-020 pin | **`b8d6a90`** — `git diff --name-only b8d6a90 HEAD \| grep -v '\.md$'` must print nothing |
| Open issues | **12**, unchanged: #319, #333, #347, #349, #353, #354, #356, #360, #365, #366, #370, #371 |
| Open PRs | this handoff's, if you are reading it before it merged |
| Local branches | `main`, plus `work/wp020-r21-repin` (merged) and this handoff's branch |
| Proxmox | no `partman-wp020-*` guest exists; VMID **9446** is next |

**The r21 obligation is discharged.** Nothing is owed. The next Rust
merge — any non-Markdown path — re-opens the three acceptances again and
owes r22.

---

## 1. What this session did

### 1.1 The r21 sitting, VMID 9445, 2026-08-15 UTC

Taken on **`b8d6a90`** (main's head at the time; two Markdown-only merges,
#373 and #374, past the #372 merge that tripped the condition).

- Kernel `5.15.0-186-generic`, verified before launch and after. euid 0
  over direct root SSH, no `sudo`, no injected variables (`env | grep -i
  partman` empty). `partman` account locked before any run.
- `cloud-init status --wait` returned and the dpkg locks were verified
  free (`settle-r21.sh`) before `02-guest-provision-r21.sh` launched;
  snapd was present at the script's start, so its designed path ran.
  **No void invocation** — the first sitting since r18 without one.
- `pre-acceptance` snapshot taken after provisioning; the sitting
  launched by absolute path (`nohup /root/05-guest-sitting-r21.sh`).
- **All three acceptances passed**, run status 0, identical value set to
  every prior sitting: 2e `configured_legs=2`,
  `adversarial_rebind_detected=true`, `detachments_confirmed=2`; 2h
  `fixtures_executed=1`, `ranges_written=1`, `contracted_bytes_written=8`;
  2j `fixtures_executed=1`, `ranges_written=2`,
  `contracted_bytes_written=16`. Eleven negative controls refused. Both
  fixture digests equal the catalogue at every read (`6d398dd2…2ec9`,
  `065d6461…05cc`). Six `EFI PART` dumps, as designed.
- Transcript SHA-256
  `1cdc380b1ecace774e028c9043c857468b6fcf6907c4ca4a736183cec4f24bac`,
  agreeing in the guest, on the host and on the workstation. Custody run
  **31**. Bundle: `/root/partman-wp020-evidence-r21` on the host,
  `C:\Users\nmcbr\PartMan-evidence\partman-wp020-evidence-r21\` on the
  workstation.
- Teardown `2026-08-15T22:26:25Z`; verified no config, no volumes, no
  LVM remnants.
- The r21 sitting script differs from r20's in **comment lines alone**,
  measured with `diff` on the host; the same is true of every step back
  to r16. The record now says "measured, not asserted" for that reason.

Whole sitting, create-to-teardown: about 30 minutes, most of it the
guest's warm build. Faster than the runbook's hour estimate because
nothing was void.

### 1.2 The re-pin, PR #375

Three Markdown files: `docs/work-packages/WP-020.md`, `README.md`,
`docs/quality/test-tiers.md`. Beyond the ordinary re-pin (Commit row,
custody run 31, stopping condition at `b8d6a90`, twentieth trip
narrative), it caught up **more stale text than any previous re-pin**,
each catch-up labelled in place:

- The Reproducibility count read "twenty-two times across nineteen
  guests" from r18 through r20 while listing twenty-three across
  twenty; r20 was missing entirely. Now twenty-five across twenty-two.
- The "And a *N*th" narrative had **no r20 entry**. Written, labelled.
- 2e's "What the harness reported" prose stopped at r16.
- 2h's Date / Hypervisor / Guest / Teardown rows stopped at r13–r16;
  2j's preamble and Date / Sitting / Teardown rows stopped at r16.

Every extension used timestamps and VMIDs the 2e rows already carried;
nothing was invented.

---

## 2. What was learned, in the order it would save you time

1. **The record's per-sitting maintenance is uneven, and the sweep is
   the fix, not a nicety.** Five re-pins in a row (r16–r20) extended the
   Commit rows and the custody table and left the 2h/2j sub-rows, the
   narrative and the Reproducibility count standing. Each was
   individually correct about the sitting it ran. **Grep for the old
   number, not for the phrase**: `nineteen disposable`, `twenty-two
   times`, `for the fifteenth`, the previous VMID, the previous pin.
2. **`git diff --name-only <pin> HEAD` against `HEAD`, before anything
   else.** The one-line audit from the previous handoff's §2 is now in
   README's Open issues section with the new pin. Run it on any main you
   did not watch land.
3. **The r20 traps held off.** Waiting for cloud-init and free dpkg locks
   before `02` (the `settle` script), leaving snapd in place for the
   script's designed path, launching `05` by absolute path, and checking
   `uname -r` before and after cost nothing and produced the first
   void-free sitting since r18. Keep all four.
4. **Nested heredocs inside `ssh '…'` break silently in this shell.**
   Patching a host script's header by piping a local Python file over
   `ssh 'python3 -'` worked first time; a heredoc-in-heredoc did not.
5. **`ssh-keyscan` the guest into the host's `known_hosts` at bootstrap**,
   not at teardown; `04-host-teardown` does a plain `scp`.

---

## 3. What is next

Nothing is owed. The previous handoff's §3.4 recommendation stands
unchanged, and this session did nothing to alter its measurements:

- **#349 plus #356** — the extent facts are unvalidated, both are
  preconditions for any extent-keyed predicate, and between them they
  build the overlapping-geometry fixtures the #347 round-2 panel said to
  commit before measuring any candidate in that family.
- **#353** if you want a self-contained win.
- **#347 round 3** only after the fixture population can see its own
  defect; the naming-relation direction is reasoned, not measured.
- **#319's authorization half** may be unblocked since #338 closed;
  nobody has checked.

Any of these that ships Rust owes r22 — name it in the PR body **and**
check the stopping condition against `HEAD` before merging, since a
named sitting can be discharged at the wrong pin by a sibling arc (§1.2
of the previous handoff, and now the WP-020 record's fifteenth
re-take paragraph).

---

## 4. Operational notes for the next sitting

- Copy the `-r21` script set forward to `-r22` on `root@10.7.7.100`;
  bump `VMID` to **9446** in 01/04, `CANDIDATE_COMMIT` in 02, the header
  prose in all four and the evidence path in 01/04. Header ordinal
  convention: the script lineage's r-number matches the sitting number.
- Sequence that worked, in full: `01` on host → find the guest by MAC in
  `ip neigh` after a ping sweep → `ssh-keyscan` it into the host's
  `known_hosts` → `sudo install` root's authorized_keys as `partman`,
  strip the `command=` prefix, lock `partman` → scp `02`, `05`, `06`,
  `settle` to `/root` and `chmod +x` → run `settle` → `nohup 02` with a
  log, wait for `Provisioning complete` → check `uname -r`, `dpkg -l
  snapd`, `losetup -a` → `qm snapshot pre-acceptance` → `nohup
  /root/05-…sh` (absolute) → wait for the `== run status` line → `04`
  with `GUEST_IP` → scp the bundle to the workstation → recompute the
  digest there.
- The Proxmox runbook memory has the same in condensed form.
