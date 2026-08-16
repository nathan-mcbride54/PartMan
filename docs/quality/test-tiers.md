# Test tiers

The test-tier definitions come from Section 11.3 of
`AGENT_BUILD_SPEC.md` 13.0.0.

## Tier 1

Tier 1 is unprivileged and safe on every developer host. It currently contains:

- Task-runner tests: command parsing, tier fail-closed behavior, and the
  SEC-010 action-pin check (WP-000).
- Canonical encoding tests: golden vectors, strict-decode rejection cases, and
  the shared cross-language fixture (WP-010).
- Fixture and interlock tests: deterministic image synthesis, partition-table
  state classification, signature layout, and the SAFE-007 refusal cases
  (WP-020).
- Design-token and accessibility tests: WCAG contrast, colour-vision
  simulation, the specification-derived role vocabulary, and the mutation table
  that proves each check can fail (WP-030).
- Slint feasibility evidence tests: exact 41-ID ADR registry parsing;
  duplicate/missing/unknown gate refusal; duplicate-key, unknown-field, and
  evidence-owned-verdict refusal; shared `pce/1` hashing; mechanical
  supply-chain rejection; missing-proof inconclusiveness; integer artifact
  ratios; and byte-fresh generated Markdown (WP-030). This tool contains no
  Slint runtime or candidate application.

Filesystem access is all of repository-controlled text, and it has grown with
each gate: workflow and composite-action YAML plus any Dockerfile they build for
the action-pin check; Cargo and npm manifests, `cargo metadata` output and the
two licence texts for the licence check; the `owned-paths` blocks, every tracked
path from `git ls-files`, and the workspace membership `cargo metadata` reports
for the two ownership checks; both lockfiles; `.cargo/config.toml`;
`schemas/canonical-encoding-vectors.json` for the shared vectors;
`schemas/design-tokens.json` for the WP-030 accessibility harness;
`docs/adr/0009-bounded-slint-desktop-feasibility.md`, the normalized
`docs/quality/slint-feasibility-data/evidence.json`, and the generated
`docs/quality/slint-feasibility.md`; and temporary directories the tests create
and remove themselves. Tier 1 also launches `git` and the compile-time-selected
`cargo` as structured subprocesses. The report reads fixed repository paths and
writes only its one Markdown target under explicit `--write`; ordinary CI is a
read-only freshness check.

*This paragraph previously said access was limited to `.github/workflows/`, two
schema files and temporary directories. That stopped being true as gates were
added, and a boundary description that lags the code is worse than none — it is
the sentence a reader would rely on to decide the tier is safe.*

**No product path opens a block device with write intent, and no command in this
repository launches an external storage tool against a block device.** WP-020
increment 2e adds one test-harness exception to the first half, stated here
before its first run: `linux-loop-read-only` opens the loop-control and assigned
loop-device descriptors `O_RDWR` to exercise mapping control through a
write-capable handle. Its SAFE-007-verified regular-file backing descriptor is likewise
`O_RDWR`-capable. Those access modes authorize kernel mapping-state changes,
not logical fixture-byte changes: the mapping carries `LO_FLAGS_READ_ONLY`, the
probe is in-process through the held loop-device descriptor, no external tool
is launched against it, and a run cannot succeed unless detach is confirmed
and both authorized fixtures' before/after hashes are unchanged. The harness
issues no logical write, discard, or zero operation. Linux's `LOOP_CONFIGURE`
and `LOOP_CHANGE_FD` paths may internally `fsync` and therefore write back
already-dirty backing-file data or metadata; this is not a zero-physical-write
claim. Product inspection retains WP-035's read-only
open boundary, and the compiled destructive-suite registry is empty — a typed
fact pinned by test since WP-020 increment 2g, no longer prose.

**A read-only product storage adapter now exists, and this sentence no longer
says otherwise.** WP-035 increment 8 reads whole-device rows from
`/sys/class/block` attributes and the udev database under `/run/udev/data`, as
files. Its exact reach, narrowed here before the increment rather than after
it: it opens no `/dev` node, launches no subprocess, adds no dependency, and
has no privilege-conditional branch — running it as root produces the same
answer as running it as anyone else. It reads no `ID_FS_*` signature key, no
`ID_PART_ENTRY_*`, and no partition children, so it reports no partition-table
state and no signature classification. Those remain gated (SI-35, SI-34) and
unbuilt.

**At Tier 1 the stronger claim does not expire: no Tier-1 test opens a block
device at all, read or write.** Regular files are all SAFE-001 permits there —
SI-35's filing records that limitation directly — and device reads, when they
arrive, are operator-run or Tier-2 work, never Tier-1 tests.

**The enumeration adapter does not weaken that claim, and the reason is worth
stating rather than assuming.** Reading `/sys/class/block/sda/size` opens a
sysfs attribute file, not the device node `/dev/sda`; the two are different
objects and only the second is a block device. And no Tier-1 test reads the
host's real `/sys` or `/run/udev` at all — the adapter's filesystem access is
behind an injected seam, and the tier exercises it over a synthesized
directory tree the test builds itself. A source-text guard holds both
properties, so the claim above is enforced rather than promised.

**SAFE-007's interlock still provides zero coverage for the product read path.**
The read-only inspector never calls `authorize`, so nothing about the
interlock's strength — held handles, share modes, or byte verification —
protects a product read. Increment 2e deliberately invokes every SAFE-007
factor for one named Tier-2 acceptance because it must prove that the privileged
loop mapping reaches a disposable generated fixture. That is coverage for
`linux-loop-read-only` alone, not a general read-path guarantee. Product reads
continue to rest on the write-intent boundary above, on SAFE-004 wherever an
external tool is invoked, and on INV-006's no-repair/no-auto-mount discipline.
Issue #94 is closed: the full acceptance, including its adversarial rebind leg,
succeeded in a disposable Proxmox-hosted non-WSL Linux VM on 2026-08-03 — on the
implementation commit `2dbf601`, and again on the merged commit `c75b340` that
lands on main — and was re-taken sixteen times, each in a fresh
disposable VM after the record's stopping condition tripped: on `582e6d1`
(issue #175), on `4fbb2f9` when increments 2g/2h landed, on `68298f2` when
the #248/#249/#250 review-finding fixes landed, on `0625b07` when
increment 2i's general executor landed, on `39b59f5` when increment 2j
registered the two-range suite (all 2026-08-11), on `a2e6db2` (2026-08-12)
when WP-070 increment 1 tripped it from outside WP-020, on `15e6469`
(the same day) when WP-070 increment 2 tripped it again, on `94bfeba`
(the same day) when WP-070 increment 3 tripped it a third time, on
`d4f61ed` (the same day) when WP-070 increment 4 tripped it a fourth, on
`59ba1f6` (the same day) when WP-070 increment 5 tripped it a fifth, on
`667f6aa` (2026-08-13 UTC) when the WP-060 unlock arc — six Rust
merges, PRs #299–#304 — tripped it a sixth time from outside, re-taken
once at the arc's head per that arc's recorded plan, and on `77b0dd7`
(2026-08-13 UTC) when the PLAN-005 cancellation arc — three Rust
merges, PRs #307–#309 — tripped it a seventh, again re-taken once at
the arc's head, and on `b50dd19` (2026-08-13 UTC) when the WP-L100
arc — three Rust merges, PRs #314/#316/#317 — tripped it an eighth,
re-taken once at the arc's head (a choice made at re-take time on the
two preceding arcs' precedent; that arc's plan recorded no sitting
economics), and on `1f9f2c7` (2026-08-13 UTC) when the ADR-0036
planner-half arc — one Rust merge, PR #336 — tripped it a ninth, at
the arc's head with the sitting recorded in that arc's plan before the
merge, and on `f463d58` (2026-08-14 UTC) when the issue-341 panic fix
— one Rust merge, PR #342 — tripped it a tenth, its sitting named in
that PR's own body before the merge, and on `901c7d2` (2026-08-14 UTC)
when ADR-0038 — one Rust merge, PR #345 — tripped it an eleventh,
likewise named before the merge, and on `b9d1ba2` (2026-08-14 UTC)
when ADR-0039 — one Rust merge, PR #351, carried-content reach and a
bounded descent at spec 13.0.0 — tripped it a twelfth, named the same
way, and on `c9cd4bb` (2026-08-14 UTC) when the verdict-multiplicity
fix — one Rust merge, PR #357 on issue #355, `node_verdict` folding over
every matching edge — tripped it a thirteenth, its sitting run the same
day but **not** named in that PR's body beforehand, and on `86db930`
(2026-08-14 UTC) when the issue-354 referent-sweep arc — three Rust
merges, PRs #361, #362 and #363: a fixtures test-determinism fix, the
naming-referent resolve sweep, and the shared referent roster — tripped
it a fourteenth, taken once at the arc's head and named in all three PR
bodies before their merges, and on `6d4a8fc` (2026-08-15 UTC) when the
issue-318 record sweep — two merges, PRs #368 and #367, whose three
non-Markdown paths are **comment-only** — tripped it a fifteenth, named
in #367's body before the merge, and on `b8d6a90` (2026-08-15 UTC)
when ADR-0040 — one Rust merge, PR #372, whose sole non-Markdown path is
the test file `protection_tests.rs` — tripped it a sixteenth, named in
that PR's body before the merge but against a pin the r20 sitting had
already moved past, and found by checking the condition against `HEAD`,
and on `b002ac3` (2026-08-15 UTC) when the body-validity arc — two Rust
merges, PRs #377 and #379, ADR-0041 — tripped it a seventeenth, taken
once at the arc's head and named in both bodies before the first merge,
and on `53c90f1` (2026-08-16 UTC) when the issue-353 arc — two Rust
merges, PRs #382 and #384, ADR-0042 — tripped it an eighteenth, under
the same practice, and on `c83d9f1` (2026-08-16 UTC) when issue #347's
round-3 act — one Rust merge, PR #388, ADR-0043 — tripped it a
nineteenth, named in the PR body before the merge,
with identical harness values and fixture digests every time. (This sentence
previously read "four times" while the custody table held six rows — the
`39b59f5` re-take never updated it, the stale-count shape again, corrected
with the sixth.) The run record with its
exclusions and stated limits is in `docs/work-packages/WP-020.md`. Closing it
registered no destructive suite.
Later packages may add pure planner, validator, and regular-file fixture tests.

**Running this acceptance requires a clean environment, and that is a real
precondition rather than a style note.** It runs `cargo xtask ci` first, and
WP-035's `no_output_in_any_mode_carries_an_environment_value` compares every
environment value of six characters or more — and every identity-bearing value
(username, home path, computer name) of three or more — against CLI output. Run
it as root over a direct login with **no `sudo` in the chain** — `sudo` sets
`SUDO_USER` by itself — and inject no variables of your own. Do not name the
VM's user, host, or any whole path component something that appears in CLI
output: a guest account named `partman` fails that gate before the acceptance
is ever reached, because the value collides with the program's own name in
`help` output. That is the tripwire working, not a false positive, and the fix
belongs in the environment rather than in an exemption — with one recorded
exception: a root login's own name cannot be changed, so its verified-static
collision with the udev caveat is exempted in the sweep itself (PR #243, the
2026-08-11 retake's discovery).

Run it with:

```text
cargo xtask test --tier 1
```

The evidence report has a fixed-path check and an explicit regeneration mode.
The first form runs inside `cargo xtask ci`:

```text
cargo xtask slint-report
cargo xtask slint-report --write
```

The MODEL-005 Rust/TypeScript parity proof is Tier 1 too, but needs a Node
toolchain, so it has its own entry point and its own CI job:

```text
cargo xtask cross-language
```

## Tier 2 exceptions; destructive Tier 2 and all Tier 3 refuse

Exactly two higher-tier selectors are registered:

```text
cargo xtask test --tier 2 --profile destructive --acceptance linux-loop-read-only
cargo xtask test --tier 2 --profile destructive --acceptance si35-loop-capture
```

The first is WP-020's privileged, non-destructive, logical-content-read-only
acceptance. The second is WP-035's SI-35 instrument capture half: it runs the
preregistered schedule of crate-owned hold-open sessions in the same class of
disposable non-WSL Linux VM, under the same native-Linux/no-WSL/explicit-
elevation gate, and emits raw records for the unprivileged
`cargo xtask si35-project` half to normalize and judge. Neither registers a
destructive suite. Every generic destructive Tier-2 request and every
Tier-3 request still refuses. Reordered, partial, additional, unknown, Tier-1,
or Tier-3 uses of `--acceptance` refuse rather than selecting a nearby action.

WP-020 increment 1 supplies the SAFE-007 interlock itself. All three proofs are
implemented and enforced together:

- the **profile**, `--profile destructive`, taken from the command line and never
  from the environment, so it cannot be inherited from a parent shell;
- the **token**, `PARTMAN_DISPOSABLE_TOKEN`, which must match what
  `cargo xtask fixtures` records. **This factor is weak, and recorded as such.**
  This file used to say the token "cannot be known without having generated that
  fixture set", which was wrong: the token is a pure function of the source, so
  it is identical on every machine that builds the same commit, and it is
  printed where CI captures it. It proves only that the invocation presented the
  exact build-derived value — anyone holding the repository can compute it
  without running the generator, so it is accident friction rather than evidence
  of provenance, and not an independent factor. That is a recorded decision
  rather than an open
  question: [ADR-0007](../adr/0007-safe-007-third-factor.md) explains why making
  it random would have been worse, since the interlock would then have to learn
  the token from the very directory it is verifying;
- the **verified target**, re-read, re-hashed, and required to byte-equal an
  image the compiled fixture catalogue produces. This is where the interlock's
  strength actually rests. Since 2026-07-29 the verification runs through an
**open file handle that the authorization then holds**: `fstat`, length, and
every content byte are read from the handle, and that same handle is what a
registered consumer receives, so rebinding the path after authorization cannot
redirect its access. On Windows the handle's share mode also refuses
  concurrent writes, deletion, and renames while the authorization lives. The
  authorization is non-cloneable and consumed once.

A single environment variable is never sufficient proof, and disposability is
computed from a target's own bytes rather than asserted by whoever asked. A block
device cannot pass, because its bytes will never equal a generated fixture, and a
target that is not a regular file is refused before its contents are read at all.

Running a generic destructive Tier 2 with all three proofs present *still*
fails, reporting that the interlock authorized its targets but a generic
request selects no suite — the refusal cites the compiled destructive-suite
registry's count (WP-020 increment 2g). Tier 3 refuses before any suite runs.
That is deliberate: a green destructive tier is exactly the signal someone
would trust when deciding whether the interlock works, so it must never be
produced by a run of nothing (Section 12, Section 16).

A destructive suite is a compiled value naming its fixture set, verified
target class, per-fixture intended-change byte ranges with each range's
replacement bytes, and teardown proof obligations. Admission consumes the same
`Authorization` the acceptances do and refuses anything but exactly the
declared fixture set. One suite is registered (WP-020 increment 2h):

```text
cargo xtask test --tier 2 --profile destructive --suite gpt-basic-512-signature-erase
```

Its attachment is deliberately **read-write** — the one place in this
repository where a loop mapping is not read-only. That is the pre-write
discipline the increment 2e record demands a destructive path establish for
itself: on a read-write attachment the kernel's loop driver refuses
`LOOP_CHANGE_FD` outright, so the rebind is *inapplicable* rather than
detected after the fact. The suite attempts it mid-run, before writing, and a
kernel that accepted it would void the run. The write is exactly the
contracted eight bytes; the range is read before the write and required to
differ from them, so the run establishes a change rather than an equality a
never-written range would also satisfy; a digest bracket over every other byte
is taken before the write and re-checked after confirmed detach. The runner
then regenerates the fixture tree **and re-reads it from disk** against the
compiled catalogue — regeneration alone proves nothing, because the manifest
it returns is computed from the images it built in memory — and it does this
on refusal as well as on success, since every refusal after the write leaves
the fixture mutated. That check establishes the files' content, not
durability.

Its operator-run acceptance passed five times — first on 2026-08-11,
re-taken the same day on `68298f2` after the #248/#250 fixes changed its
own probe and write lines, on `0625b07` through increment 2i's general
executor, and on `39b59f5` in the sitting that first took the 2j
acceptance, then on `a2e6db2` (2026-08-12) after WP-070 increment 1
tripped the stopping condition — each time in the same VM sitting as a
re-take of the read-only acceptance above, and is recorded in
`docs/work-packages/WP-020.md`. The kernel refused the mid-run
`LOOP_CHANGE_FD` as the design requires (measured on two kernel revisions,
and classified from an observed status re-read since the second sitting),
the contracted range changed and nothing else did, and the sitting's full
negative-control set refused every time.

Increment 2j's two-range suite —
`--suite gpt-basic-512-both-signatures-erase`, both GPT header signatures
erased in one run — passed its acceptance on its first take in that same
2026-08-11 sitting, the first real-kernel run of the general executor's
multi-range chain: `fixtures_executed=1`, `ranges_written=2`,
`contracted_bytes_written=16`, one attachment and one confirmed detach, both
ranges restored by regeneration, eleven negative controls refused. Increment
2 is thereby delivered as scoped; every generic destructive Tier-2 request
and every Tier-3 request still refuses, because a generic request selects no
suite.

The named acceptance consumes the non-cloneable `Authorization`, keeps both
verified backing descriptors live, and requires each held object's initial hash
to match its compiled fixture-catalogue digest before any attach. For each leg
it configures a kernel-selected loop device from the exact held descriptor and
verifies the kernel's backing identity against that file.
It derives `/dev/loopN` only from the kernel-returned number, records the held
node's filesystem device, inode, and `rdev`, and rechecks that same descriptor
before and after use. The probe is an in-process positional read through the
held loop descriptor, never an external or path-addressed tool. After the probe,
the backing identity, loop configuration, and held-node identity must still
match. Cleanup issues `LOOP_CLR_FD` through the held descriptor and then requires
`LOOP_GET_STATUS64` to report `ENXIO`; a detach that cannot be confirmed is a
refusal. After `ENXIO`, the harness drops the held loop `File`, then boundedly
requires the exact retained-rdev `/sys/dev/block/M:m` root to be readable, not
itself contain a `partition` attribute, and have no immediate child containing a
`partition` attribute. A missing, unreadable, ambiguous, or retry-exhausted
state is a cleanup-uncertain refusal that requires discarding or reverting the
VM. The adversarial `LOOP_CHANGE_FD` leg discards its pending bytes when the
expected backing mismatch is detected. Only after both legs confirm that full
teardown does the harness hash both held fixture objects again, and only
unchanged hashes release a success observation. Any identity/configuration
mismatch, undetected rebind, changed fixture hash, or cleanup failure refuses.
A `LOOP_CONFIGURE` `EBUSY` also refuses immediately, without retry, because
isolated loop state was not established; bounded retries exist
only in cleanup, where ordinary kernel/udev discovery is permitted.

Those digest and status checks are discrete samples, not exclusive claims, and
cannot defeat an ABA change entirely between samples. External run evidence
must exclude every other actor able to modify either fixture and every other
actor able to administer or rebind loop devices. Ordinary kernel/udev read/open
discovery is allowed and is handled by bounded detach retries plus exact
retained-rdev sysfs inspection. VM isolation bounds consequences but does not
itself prove the exclusions. A pass establishes that no persistent fixture
change or rebind was observed under those conditions and that the deliberately
exercised rebind was positively bound to the exact conflicting descriptor and
detected; it does not establish continuous binding against a concurrent actor.
A future destructive path needs a separately proven pre-write discipline and
may not inherit this acceptance's conclusion.

The backing, loop-control, and loop-device descriptors are `O_RDWR`-capable for
mapping control, while `LO_FLAGS_READ_ONLY` forbids logical loop-device writes;
the harness issues no logical write, discard, or zero operation. Both configure
and rebind can nevertheless make the kernel `fsync` the backing file, so dirty
data or metadata may be written back even though logical contents do not change.

Outside that named harness, the product boundary remains the one WP-035 records:
`inspect --replay` reads one caller-named regular file; a pre-open look refuses
devices and directories before any open in the common case; `fstat` through the
opened handle is the authority; and a device swapped in by a rebinding race is
opened read-only at most long enough for the handle to identify itself, then
refused with no byte read. The doctor's roster probes launch tools at compiled
absolute paths and open nothing else. Tier 1's filesystem access beyond those
two stated reaches remains limited to repository-controlled files, to the
generated fixture tree under `tests/generated/` which `.gitignore` excludes,
and — added with WP-035 increment 8 — to synthesized directory trees a test
builds and owns for the duration of that test. The enumeration adapter's
production roots, `/sys` and `/run/udev/data`, are **not** in Tier 1's reach:
they are compiled constants the tier never passes to the seam.
