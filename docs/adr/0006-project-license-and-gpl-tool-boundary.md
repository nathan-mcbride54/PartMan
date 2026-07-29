# ADR-0006: Project license and the GPL tool boundary

- Status: Accepted
- Date: 2026-07-29
- Spec version: 4.0.0
- Work packages blocked: WP-000 (recorded gap), and every future package that
  links or vendors a third-party storage library
- Requirement IDs: SEC-005, SEC-010, PKG-004, SAFE-004, LIN-001
- Decision owners: @nathan-mcbride54

## Context

The project has carried no license by decision. `README.md` said so, every
crate set `publish = false` to avoid Cargo's requirement, and `deny.toml`
exempted `publish = false` crates from the license gate so `cargo deny` would
not report `error[unlicensed]` against the workspace itself.
`docs/traceability/WP-000.md` recorded the consequence as an open gap: SEC-005
requires a dependency and license inventory per release, and an inventory that
omits the product's own terms is not an inventory. It had to be resolved before
the Section 19 release gate.

Two things now force it earlier than that gate. The repository is going public,
and default copyright on a public repository is the worst of both worlds: the
source is readable by everyone and usable by no one, while every drive-by pull
request arrives with undefined rights on both sides. And the stated product
intent has changed — PartMan is to be given away, including the capabilities
that comparable tools charge for.

This decision cannot be left implicit. A license absorbed by default, or
inferred later from the fact that people were already copying the code, is not a
grant; and after outside contributions arrive, relicensing requires the
agreement of every contributor. This is a one-way door and belongs in the record
with its rejected alternatives.

A second question rides along with the first, and is the reason this ADR is not
purely administrative. PartMan's Linux path (LIN-001) is built on software that
is largely GPL or LGPL. Whether that constrains PartMan's own terms depends
entirely on *how* PartMan reaches it, and that boundary has never been written
down. While the project was unlicensed the question was moot. It is not moot now.

## Safety analysis

The license does not touch device identity, the privilege boundary, plan
validation, journaling, recovery, secrets handling, hostile-input parsing, or
disposable-test coverage. No MUST in `AGENT_BUILD_SPEC.md` is weakened by any
option considered here, and none of them changes what the software does.

Two safety-adjacent consequences are real and are recorded rather than waved
past:

**A permissive license permits a fork with the interlocks removed.** Someone may
take this code, delete SAFE-007's disposable-target proof or HLP-003's per-apply
authorization, and ship the result. This is true, and it is *equally* true under
GPL-3.0 — copyleft compels source disclosure, not safety. Copyleft is therefore
not a mitigation for this risk and must not be chosen as though it were. The
actual mitigations are the ones already in the design: the helper recomputes
capability and validation independently of any client claim (HLP-002, CAP-007),
so a modified *client* cannot talk an unmodified helper into an unsafe apply;
and the name is not licensed (Apache-2.0 §6 grants no trademark rights), so a
fork that strips the safety machinery has no right to call itself PartMan.

**Neither arm carries a warranty.** Both disclaim it in capital letters, which
for a tool that destroys partition tables is not boilerplate. It does not
diminish the project's own obligations under Section 12 and Section 16 — the
prohibition on simulating success or reporting a pass for a run of nothing is a
project rule, not a legal one, and no disclaimer relaxes it.

## Options considered

### Option A — MIT only

The most recognizable license in existence and the one originally proposed.
Short enough to read, understood by everyone, imposes only attribution.

Rejected. MIT grants copyright permissions and is silent on patents. Whether it
carries an implied patent license has never been settled by a court, so the
answer is unknown rather than favorable. PartMan drives NTFS, exFAT, APFS, and
ReFS code paths, a field with a live patent history — Microsoft's exFAT claims
were enforced against implementers for years before the 2019 Linux grant. The
exposure is small, but Option B removes it at a cost of one additional file.
MIT alone is also not what this project's own dependency graph uses, so it would
be a deliberate step away from the ecosystem norm for no gain.

### Option B — MIT OR Apache-2.0, at the recipient's choice

The Rust ecosystem's standard dual license: rustc, the standard library, Tauri,
and the overwhelming majority of crates in this project's tree use it.
Downstream picks whichever arm suits them.

### Option C — Apache-2.0 only

Provides the patent grant without maintaining two files.

Rejected. Apache-2.0 is incompatible with GPL-2.0-only. For most projects that
is an abstraction; for a partition manager it is a foreseeable dead end, because
the neighboring projects most likely to want this code — util-linux, parted, and
their kin — sit in exactly that license family. Option B keeps that door open
through its MIT arm at no cost.

### Option D — GPL-3.0-or-later

Matches GParted's lineage and prevents a competitor from taking PartMan closed
and selling it.

Rejected, and the rejection is a judgment call rather than a technical
conclusion. The stated goal is that *users* pay nothing; it is not that
*redistributors* are forbidden to charge. Copyleft addresses the second, which
was never the objective, and charges real costs for it: it complicates the
macOS notarized-helper and app-store distribution paths (SEC-004, PKG-004,
ADR-S1), and it contradicts the permissive-only allow-list in `deny.toml`, which
would have to be rewritten. As noted in the safety analysis, it also does not
buy the safety guarantee it is sometimes assumed to buy. If the objective ever
becomes "no one may ship a proprietary derivative," this ADR is superseded
rather than amended — see the revisit conditions.

### Option E — MPL-2.0

Per-file copyleft; already on the `deny.toml` allow-list.

Rejected. It splits the difference between goals that are not actually in
tension here, and adds per-file obligations that every contributor would have to
reason about, in exchange for a protection that Option D's rejection already
established is not being sought.

## Decision

**PartMan is licensed `MIT OR Apache-2.0`**, at the recipient's choice, matching
the Rust and Tauri ecosystems. `LICENSE-MIT` and `LICENSE-APACHE` carry the
texts; every workspace member, the excluded `fuzz` crate, and
`packages/canonical/package.json` declare the SPDX expression.

It best satisfies the normative requirements because it is the only option that
supplies an explicit patent grant (Apache-2.0 §3, with defensive termination)
while remaining compatible in both directions — a GPL-2.0 project can take the
MIT arm, a GPL-3.0 project the Apache arm — and because both arms were already
on `deny.toml`'s allow-list, so SEC-005's license inventory becomes satisfiable
without relaxing a single supply-chain rule.

Contributions are inbound=outbound under Apache-2.0 §5: a contribution is
offered under the same dual terms unless its author states otherwise in writing.
No CLA is required.

### The GPL tool boundary (binding)

PartMan's permissive terms are only sustainable because of *how* it reaches the
platform tooling, so the boundary is normative here rather than advisory:

- **Separate processes are the default.** SAFE-004 already requires external
  tools to be invoked with structured argument arrays from a fixed allow-list at
  trusted absolute paths. Running a GPL program as a separate process places no
  obligation on the caller's own terms. `ntfs-3g`, `e2fsprogs`, `xfsprogs`,
  `btrfs-progs`, `mdadm`, and `cryptsetup` are reached this way.
- **IPC is not linking.** UDisks2 is GPL and is reached over D-Bus (LIN-001).
  A D-Bus client is not a derivative work of the daemon it calls.
- **LGPL libraries may be linked dynamically**, preserving the user's ability to
  relink — the condition LGPL §4 attaches. `libblkid` and `libblockdev` are the
  expected cases.
- **GPL libraries MUST NOT be linked, statically or dynamically.**
  `libparted` (GPL-3.0-or-later) is the specific hazard: it is the obvious crate
  to reach for when a partition-table editor is wanted, and linking it would
  relicense PartMan by operation of law rather than by decision. Use
  `parted`/`sgdisk` as processes, or a native Rust implementation.

Each library's actual license MUST be verified at the integration commit that
introduces it, against the version being linked. No frozen table of library
licenses is kept in this repository, because a hand-maintained table of facts
that change upstream is a table that drifts silently — the same reason
`docs/traceability/` is meant to be generated and DOC-003's matrix is never
hand-edited.

## Consequences

Positive:

- SEC-005's dependency and license inventory is satisfiable. The recorded WP-000
  gap closes.
- `deny.toml` sets `private = { ignore = false }`, so the workspace's own crates
  are now checked by the same license gate as every dependency. The exemption
  that existed only to tolerate the unlicensed state is gone rather than left
  dormant.
- Outside contributions become possible. `CONTRIBUTING.md`'s bar on them, which
  existed because rights were undefined for both sides, is lifted.
- PKG-004's shipped notices have something definite to state.

Negative and accepted:

- A proprietary derivative is permitted. Deliberate, per Option D.
- PKG-004 must package **both** license texts, and Apache-2.0 §4 obligations
  (license copy, modified-file notices, retained attributions) travel with every
  redistribution of the product.
- Relicensing later requires every contributor's agreement. This is the one-way
  door named in the context.

Migration: none. No code changes; no schema, hash, encoding, or public interface
is affected, so no versioned migration under MODEL-003 arises.

## Verification

- `cargo xtask supply-chain` is the automated evidence. With
  `private = { ignore = false }`, `cargo deny check licenses` fails on any
  workspace crate that lacks a license key or declares one outside the
  allow-list. This is a real gate, not a restatement: before this ADR the same
  command passed *because* the workspace was exempt.
- No new test tier, fixture, or privileged environment is required. The check is
  Tier 1 and touches no block device.

Two declarations are **not** covered by that gate, and are recorded as gaps
rather than presented as covered:

- `fuzz/Cargo.toml` is excluded from the workspace and therefore from
  `cargo deny`'s graph, exactly as `docs/quality/dependency-policy.md` notes for
  its toolchain pins. Its `license` key can be deleted without failing CI.
- `packages/canonical/package.json` is checked by no license gate at all;
  `npm audit` covers advisories, not licenses.

Closing both needs a manifest-consistency check in `xtask` asserting that every
manifest in the repository declares this exact SPDX expression. That check does
not exist and is not claimed to exist.

## Revisit conditions

- A requirement emerges to **link** a GPL library, with no separate-process or
  native alternative. Then the choice is between the feature and these terms,
  and it returns here.
- The objective changes to preventing proprietary derivatives. Option D is then
  reconsidered, superseding this ADR — and only while it is still possible,
  which is to say before contributions from others accumulate.
- A patent claim is asserted against a file-system code path, testing whether
  Apache-2.0 §3 covers what was assumed.
- A contributor declines inbound=outbound, requiring an explicit CLA decision.
- `libblkid` or `libblockdev` relicenses away from LGPL, invalidating the
  dynamic-linking allowance above.
