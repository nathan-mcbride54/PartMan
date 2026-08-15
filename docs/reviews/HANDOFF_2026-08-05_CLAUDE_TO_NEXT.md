# Handoff — 2026-08-05, end of session

**From:** Claude, working with Nate through 2026-08-05.
**To:** whoever picks this up next.
**Pick up here:** §1, the second-reader readback. Nate has asked for help with
it specifically, and it is the one task in this document that **cannot be done
by the session that produced the records** — which is why it is still open.

> **Untracked local handoff artifact.** `docs/reviews/**` belongs to WP-000.
> Do not stage this into a WP-035 or WP-010 commit. If Nate wants it tracked,
> land it separately under WP-000 ownership. Two earlier handoffs sit untracked
> beside it for the same reason.

Repository state as this was written: `main` at `562f8bb`, spec **6.1.0**, no
open PRs, no open issues.

---

## 1. THE PICKUP: the second-reader readback

### What it is

Three macOS captures have an **outstanding second-reader obligation**. The
custody rule in `docs/quality/observability.md`'s macOS matrix says a second
reader must retrieve the transcript through its locator and rehash it *"before
any cell leaves `not yet taken`"*, and that **"custody failure is not a
result"**. The cells were recorded with the obligation open and stated as open;
discharging it is what makes them citable by a register decision.

### Why you, and not the previous session

The precedent is explicit. The SI-35 sitting record describes its own readback
as *"performed by the producing session and recorded there as not
independent"*, and only counts the separate one: *"an independent reader
session on 2026-08-04 that retrieved both artifacts through the locator and
rehashed each to its recorded digest"*.

I wrote all three of these records. **My readback would not count.** If you are
a fresh session, yours does. Do not let anyone — including a future me — record
a producing-session readback as discharge.

### Exactly what to verify

Evidence store: `C:\Users\nmcbr\partman-evidence\` (custodian Nate McBride).

| Capture | Transcript | Recorded SHA-256 | Bytes |
|---|---|---|---|
| `partman-macos-sitting-2` | `out-pre/00-transcript.txt` | `da5506e97d75e889b0e74c78c747912051707566f450de1d701fee789590f94d` | 18 647 |
| `partman-macos-sitting-2` | `out-post/00-transcript.txt` | `4f6e8916c87477869c617e28aeaf15cfd7f47cb571e7c44ae721b8f1027081cc` | 5 320 |
| `2026-08-05-macos-m10-ci-run31020018982` | `macos-m10-capture/00-transcript.txt` | `259b1046e1d80b40fb92fcfd99ef018af86f11b7f5086aca3e5c239a15436256` | — |

For each: retrieve **through the locator** (not from a path someone hands you),
rehash, confirm the digest matches, and confirm the archive is readable.
Sitting 1 (`2026-08-05-macos-increment6-sitting1`) is a **void** sitting; its
transcripts are retained but no cell rests on them, so it is not part of this
obligation.

### One thing you must carry into the record

**Sitting 2's digests were recorded late.** They were missing entirely until
2026-08-05 — the custody rule requires "hash algorithm, digest, and byte length
recorded", and that paragraph had only the locator and custodian. That made the
readback *unperformable as specified*: you cannot rehash "to its recorded
digest" when none is recorded.

They were then computed **from the retained capture**, which has not been
modified since retention. That is weaker than a digest recorded at retention
time, and `observability.md` says so in terms. **Do not let the readback launder
that into the stronger property.** A matching rehash confirms the copy is
unchanged since the digest was taken; it does not confirm the digest was taken
at retention. M10 needed no such repair — its digest was recorded correctly at
the time and matches.

### Where to record the discharge

`docs/quality/observability.md`, which is **WP-035's**. Three places currently
say the obligation is outstanding:

- the file's global status header (near line 15);
- the macOS sitting 2 Artifacts paragraph;
- the M10 sitting Artifacts paragraph.

`README.md`'s M0.5 prose and the WP-035 roadmap row also mention it. Sweep for
all of them — see §4, "a sitting lands in more places than one".

### What discharging it unblocks

- **SI-34's** observability element currently rests on M10 with the obligation
  open. It should not be advanced until this is done.
- **SI-39** (filed today) rests on the macOS blank-versus-foreign measurement,
  which is sitting 2 and M10. Its entry records the dependency.

It does **not** resolve either issue. It removes a custody caveat, nothing more.

---

## 2. Where the project stands

**Measurement is complete.** Windows, Linux and macOS all measured. **No
preregistered cell on any platform is `not yet taken`.** M9 is `not
established` (Apple Silicon has no Fusion Drive), which is that cell's own
declared outcome.

**The headline result:** no unprivileged client projection on any of the three
platforms separates a healthy GPT from one whose two tables disagree. M10 then
located the separating fact — it lives in the backup table, behind a read the
client is denied. So the fact exists and is privileged.

**The register grew today, which is correct rather than alarming.** Ten items
gate increment 3, seven of them direct. SI-38 was filed and resolved; SI-33's
liveness precondition was discharged; SI-39 was filed. Completing the evidence
exposed conflicts that were already latent — and one, SI-39, this repository
created that same morning via ADR-0013. The filing says so, deliberately.

**Nothing on the register is blocked on measurement any more.** Every remaining
item needs a decision or a design.

**The CLI reads real hardware.** `partman inspect` on Linux lists whole devices
with sizes, block sizes, vendor, model, serial and WWN, each labelled by the
interface that reported it, and udev values carrying an in-band caveat that
they are what root's `udevd` cached at device-add time.

---

## 3. Other open threads, in the order I would take them

1. **The readback** — §1. Small, blocking two register items.
2. **ADR-0014 (SI-35's axis) is blocked, and not on drafting.** Two drafts and
   two adversarial rounds converged on the same fork. Its central move — taking
   partition-table state out of the hashed body — breaks a **standing
   regression guard in ADR-C4**: *"A positively absent partition table and an
   unreadable one produce different body values."* Remove the field and
   `Absent` and `Indeterminate` collapse, which is the data-loss shape ADR-C4
   refused, reached by another route. Drafts are in the session scratchpad, not
   the repo; `docs/adr/0014-si35-table-state-axis.md` is **reserved** (WP-010)
   and deliberately empty. Do not draft a third version before Nate decides
   whether ADR-C4's guard can be amended.
3. **Increment 9, macOS adapter.** Obstacle is named already: `diskutil` emits
   plists, `apps/cli` has no dependencies **by design** and a test enforces the
   empty closure. Either hand-write a bounded plist reader or take a dependency
   and restate the guard. A parser of externally supplied bytes also attracts a
   Section 11.4 fuzz obligation.
4. **Increment 10, Windows adapter.** Needs Nate's decision first: no route is
   simultaneously dependency-free, `unsafe`-free and Section-16-clean. WMI needs
   FFI, which `apps/cli` cannot host under `unsafe_code = "deny"`; PowerShell
   adds a shell to the roster and still needs a JSON reader. The prior analysis
   recommended deferring Windows to WP-W100 and shipping its reach declaration
   with the existing typed refusal.
5. **SI-39, then SI-11 and SI-27.** The last two are the genuine design rounds
   and nothing routes around them.

---

## 4. Traps this session actually fell into

Not general advice — each of these cost real time today.

**A guard that cannot fail.** I wrote a privilege test that enumerated the same
fake source twice and compared. Nothing varied between the runs, so a
privilege-conditional branch would have sailed through it. Found by mutating
the code, not by reading the test. **Mutate your guards and watch them fail
before trusting them.**

**`cfg`-gated code is linted nowhere.** `cargo xtask ci` on Windows compiles the
Linux-gated tests out; running only `cargo test` in WSL never lints them. CI was
the first thing to see that code. **Run the full `cargo xtask ci` in WSL Debian,
not just `cargo test`.**

**Doc comments and code disagreeing, with the doc right.** Three times in one
file, written minutes apart: a limit documented as a refusal and implemented as
a silent truncation; a filter whose comment said "absence" and whose code
accepted any error; a `trim_end` that turned a padded SCSI vendor into a
*positively determined absence*.

**Claiming reachability without checking it.** A commit title said the adapter
read real devices while `enumerate` was called from nothing but tests. Check
what actually calls the thing before describing what it does.

**Restoring a file with `Move-Item` preserves its old mtime**, so cargo reuses
the binary built against the mutation and the suite keeps failing after you have
undone the change. Touch the file.

**Governance ordering is enforced, not advisory.** An assignment change must be
its own PR and merge first; `verify-change-ownership` refuses a mixed range. A
*claim* on a path that does not exist yet fails the ownership inventory — use
`owned-paths-reserved` instead. Both cost a round trip today.

**A sitting lands in more places than one.** Recording a result updates the
section that ran and leaves stale counts elsewhere. Three separate staleness
fixes were needed today for work done the same week.

---

## 5. Local toolchain

Windows host, WSL Debian carries the Linux half. Never move the working copy
onto the WSL filesystem — build from `/mnt/d/PartMan`. macOS is the one gate
that cannot run locally; GitHub-hosted `macos-15` runners are Apple Silicon,
ephemeral, and free on this public repo, which is how M10 was taken at no cost
after the paid Mac routes turned out to carry 24-hour minimums.

`.github/workflows/macos-m10.yml` exists and is `workflow_dispatch`-triggered
if a macOS privileged leg is ever needed again.
