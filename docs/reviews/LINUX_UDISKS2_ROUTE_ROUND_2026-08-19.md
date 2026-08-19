# The Linux UDisks2 route round — LIN-001's discovery half, and where Section 9's UDisks2 conjunct belongs

**Date:** 2026-08-19. **Base:** `3356e20` (main), spec 17.4.0.
**Directive:** Nate — "finish the floor".
**Question:** Section 9's Debian/Ubuntu floor is a conjunction — "Debian 12 /
Ubuntu 22.04 LTS; kernel ≥ 5.15; UDisks2 ≥ 2.9". WP-L100 increments 5b and
obligation 4 measured and determine the first two conjuncts (DR16–DR19); the
third is reported `Undetermined` by construction, because the delivered Linux
client contract has no source for a UDisks2 version and LIN-001's route —
"Use UDisks2 for discovery/authorization and libblockdev or authoritative
native tools for mutations" — was claimed increment-gated behind a recorded
route decision at assignment (`docs/reviews/WP-L100_ASSIGNMENT_PLAN_2026-08-12.md`
§4) and deferred again by the increment-5 plan (§7.3). So today **every
Debian/Ubuntu host answers `Undetermined`, and the engine blocks every
operation on it under the floor reason, naming UDisks2.** This round decides
LIN-001's discovery half and, from that, what the UDisks2 conjunct is and
where it is determined — so the floor can be *finished* rather than
perpetually undetermined.

> Committed session record. `docs/reviews/**` is in WP-000's `owned-paths`
> block and lands in its own `Work-Package: WP-000` commit, never bundled
> with code. Nothing below is decided; §4 is for the decision owner. The
> recommendation prices as a **spec change under ADR** and says so.
>
> **Decided 2026-08-19 (Nate): option B, with the tool-floor entry at
> first invocation (§3.7, §5.2).** Recorded as ADR-0054 and spec 18.0.0.

## 0. The premise, and the texts the round works under

- **LIN-001** (`AGENT_BUILD_SPEC.md:617`): "Use UDisks2 for discovery/authorization
  and libblockdev or authoritative native tools for mutations." One
  sentence, two halves: a discovery/authorization interface and a
  mutation toolset.
- **Section 9** (`AGENT_BUILD_SPEC.md:779-791`): "Initial floors; changeable
  only via ADR. The capability engine may narrow further at runtime
  (CAP-004); it may never widen below these floors." The Debian/Ubuntu row:
  "Debian 12 / Ubuntu 22.04 LTS; kernel ≥ 5.15; UDisks2 ≥ 2.9". The Arch
  row: "Current rolling", "tool-version-gated". And the closing sentence:
  **"Per-tool version floors live with the capability fixtures (CAP-006) in
  `docs/capabilities/`, not in this spec."**
- **CAP-004**: "Confirm required native API/tool availability and version
  at runtime." Delivered as WP-050's `ToolRequirement`/`ToolFloor`/
  `tool_state` (WP-L100 increment 5a supplies the probes; the roster is
  pinned empty for every served source-class operation because no tool is
  invoked), fail-closed on every open arm.
- **SAFE-002** (`:151-157`): the discovery layer runs without elevation.
  **WP-L100's contract** (`crates/adapter-linux/src/lib.rs`): the adapter is a
  pure library over an injected read seam — four client-readable
  interfaces (sysfs, the udev database, procfs, os-release), opens no
  device node, launches no process, has an empty dependency closure, and
  every interface entered by a row in `docs/quality/observability.md`
  (nineteen DR rows to date). The structural guard
  `the_adapter_opens_no_device_node_and_launches_no_process` holds it.
- **HLP-002**: "Before the first write, the helper independently
  re-discovers topology and recomputes capability and validation results.
  Client-provided discovery … is an untrusted hint, never an input to
  authorization." **LIN-009**: "Use polkit rules scoped to validated plan
  execution, not broad command execution." **ADR-0006:153**: "IPC is not
  linking. UDisks2 is GPL and is reached over D-Bus (LIN-001)."
- **WP-050 increment 5** (`PlatformFact::Undetermined { conjunct }`): the
  engine blocks and names the conjunct; "never met (that would widen below
  the floor), never below (no measurement said so)".
- **The assignment plan's warning** (§4): adopting UDisks2 "buys LIN-001's
  named interface at the price of a D-Bus client dependency and an IPC
  surface … LIN-001 is claimed increment-gated behind its own recorded
  route decision, and the assignment says so rather than letting the sysfs
  route become LIN-001 by drift." This round is that record; it must not
  be the drift it warned against, and §3 asks whether it is.

## 1. What is measured

Every fact below is in the record or was measured for this round; nothing
about UDisks2's own behaviour is asserted from memory.

1. **UDisks2 is absent by default on two of the three tiers' own images,
   and purged on the third's acceptance guests.** DR18 (the Arch cloud
   image): `pacman -Q udisks2` fails, no unit, `udisksctl`/`udisksd` absent
   by name. DR19 (Debian 12 genericcloud): `dpkg-query` rc 1, no unit file,
   three paths absent by name. The jammy cloud image *ships* `udisks2`, and
   every WP-020 acceptance guest purges it with `snapd` as a recorded
   deviation (`docs/work-packages/WP-020.md:915-918`) — so no guest in the
   record has ever run the daemon, and the read-only product has passed
   fifty-four acceptances without it.
2. **No file under the four interfaces carries the daemon's version**
   (increment-5 plan F2; DR16–DR19 confirm the shapes). The version is
   reachable only over D-Bus (`org.freedesktop.UDisks2.Manager`'s `Version`
   property, which D-Bus-activates the daemon if it is installed and not
   running) or by launching `udisksctl`, neither of which the contract has.
3. **The package manager's record is a client-readable file**, measured
   for this round on two Debian 13 hosts (this workstation's WSL Debian and
   the Proxmox node, both `ID=debian`, `VERSION_ID="13"`): `/var/lib/dpkg/status`
   is `-rw-r--r-- root:root`, world-readable, and carries `Package:`/`Version:`
   stanzas; `udisks2` is not installed on either, candidate
   `2.10.1-12.1+deb13u2`. **Not in the record**: no DR row measures this
   file on a guest, and the pacman equivalent (`/var/lib/pacman/local/`) is
   unmeasured. It is named here as a *possible* source, not an established
   one.
4. **The floor's two other conjuncts are determined on every measured
   tier.** DR16/DR17 (Ubuntu 22.04, kernel 5.15.0-186), DR19 (Debian 12,
   kernel 6.1.0-52), DR18 (Arch on `ID` alone). With UDisks2 removed from
   the conjunction, every measured Debian/Ubuntu guest would answer
   `MeetsFloor` today; with it, every one answers `Undetermined` and is
   blocked.
5. **Where the product does and does not talk to UDisks2 today**: nowhere.
   No crate depends on a D-Bus client; the doctor probes only the
   repository's own toolchain; `docs/capabilities/tool-version-floors.json`
   is empty "because no storage tool is invoked anywhere in the product
   yet … a tool's floor arrives with the first package that invokes it,
   under review, with its basis stated" (`format.md` §2).
6. **What the Debian 12 and Ubuntu 22.04 archives ship** as `udisks2` is
   2.9.x (the floor's own number, which is presumably why 2.9 was chosen);
   this is archive knowledge, not a row, and is flagged as such — it
   matters only to options that keep the conjunct.

## 2. The options, each against the texts

**A. Take LIN-001 literally: UDisks2 becomes the client's discovery
interface.** The adapter gains a D-Bus client (a crate such as `zbus`, or
a hand-rolled wire implementation), enumerates block objects through
`org.freedesktop.UDisks2`, and reads the conjunct from `Manager.Version`.
*For:* it is what LIN-001 says; UDisks2 exists precisely to be a
client-readable storage interface; the conjunct becomes determinable the
honest way (absent daemon → `BelowFloor`, measured). *Against:* (i) it
replaces nineteen rows of measured file interfaces with an interface that
has **zero rows** and that, on every guest in the record, is not installed
— the read-only product would be `blocked` on every default Debian 12 and
Arch install and every acceptance environment, by a floor the spec wrote
for an inventory that opens nothing; (ii) the empty dependency closure and
the "launches no process" guard fall — a D-Bus method call to an
activatable name *causes* `dbus-daemon` to spawn `udisksd`, a process
launch the adapter did not make but did provoke, which SAFE-004's identity
and allow-list discipline cannot see; (iii) MODEL-004 provenance weakens:
UDisks2 is itself a udev consumer, so every property the adapter reports
would be attributed to a daemon that read the same database the adapter
reads directly today — a second hand where the record has a first; (iv)
ADR-0006's "IPC is not linking" holds, so licensing is not a cost — noted
so it is not counted twice. *Cost:* a new dependency tree to audit,
a D-Bus fixture apparatus for Tier-1 tests, a DR sitting per tier with the
daemon installed, and a rewrite of increments 2–4b's sources. Nothing in
the spec's texts *requires* (i)–(iii); LIN-001 names the interface, it does
not require the product to be unusable without it — but that is what the
floor row does once the interface is the route.

**B. Decide LIN-001's discovery half for the measured route, and move the
UDisks2 conjunct to where Section 9 already says tool floors live.**
LIN-001 is revised to say what the record has established: discovery
through the kernel's and the distribution's client-readable interfaces
(sysfs, the udev database, procfs, `os-release`), measured row by row;
**UDisks2, libblockdev or authoritative native tools for authorization and
mutations, behind the helper's own route decision (WP-L110)**, where
LIN-009's polkit scoping and HLP-002's re-discovery live. The Section 9
Debian/Ubuntu row drops "UDisks2 ≥ 2.9" from the *platform* conjunction,
and the number moves to `docs/capabilities/tool-version-floors.json` as a
**tool floor** — entered, per `format.md`'s rule, by the first package that
invokes UDisks2, with the ADR as its basis — gating the operations that
use the tool through CAP-004's delivered `ToolFloor`/`tool_state`
machinery (fail-closed: missing → `Missing` → blocked for *those*
operations). *For:* it is the Section 9 closing sentence applied to its
own row ("per-tool version floors live with the capability fixtures … not
in this spec"); it matches the delivered, measured contract and the
engine's existing seams (5a's roster is exactly the place a UDisks2
requirement would be declared for a mutating operation); the floor becomes
*finished* — two conjuncts, both measured, `MeetsFloor` on every measured
guest — and the UDisks2 number is kept, not dropped, attached to what it
gates. *Against:* (i) it is a **spec change**: LIN-001's sentence changes
meaning and the floor row loses a conjunct — by the pricing rule this
repository uses (major when a sentence becomes false), **18.0.0**, with an
ADR, `spec-change` label and changelog entry; (ii) the assignment plan's
drift warning — answered only if this round is read as the recorded
decision it asked for, with the alternative (A) priced honestly above,
and if the ADR records that the route was chosen *on the rows*, not on the
code; (iii) the authorization half of LIN-001 stays undecided — correctly,
since no helper exists and HLP-002 says the helper's discovery is its
own; the ADR must say that the helper's route is a separate decision and
that WP-L110 may yet choose UDisks2 there without contradiction.

**C. Keep files as the route, but read the conjunct from the package
manager's record** (`/var/lib/dpkg/status` on Debian/Ubuntu, `/var/lib/pacman/local/*/desc`
on Arch) as a fifth client-readable interface. *For:* no D-Bus, no launch,
the conjunct determinable from a file; dpkg's status file is measured
world-readable on two hosts today. *Against:* (i) it determines the
*installed package*, not the daemon's availability, and the two differ
(masked unit, different prefix, a container without D-Bus) — the adapter
would be authoring "UDisks2 ≥ 2.9 is met" about a thing it never consulted,
which is the widening-by-assertion §0 forbids, dressed as a file read;
(ii) two distribution-specific databases with their own shapes, each
needing rows (two more DR cells and an Arch guest with udisks2 installed);
(iii) it leaves LIN-001 exactly as undecided as today — it finishes the
number, not the route. C is a source, not a decision; if B is taken, C is
the natural *probe* for the tool floor (a `ToolProbe::Present { version }`
from the package record, or from `udisksctl --version` through the
launcher when a mutating package owns one) — so it is not wasted, it is
demoted.

**D. Keep everything, and qualify the row in place**: "UDisks2 ≥ 2.9 where
UDisks2 is used". *Against:* it is B's spec change with the number left in
the platform row, where Section 9 itself says tool floors do not live, and
where `PlatformFact` — a per-platform fact, not per-operation — cannot
express "where used" without a fourth arm. Strictly worse than B; listed
so it is seen to have been considered.

**E. Leave it undetermined.** The status quo: honest, and the read-only
product is blocked on every Debian/Ubuntu host it has ever been measured
on. Section 9's row would then be a floor no host meets by the product's
own reading, forever. Not a resolution.

## 3. What is genuinely open, and the adversarial pass

1. **Is B the drift the assignment plan warned about?** The warning was
   against the sysfs route *becoming* LIN-001 without a decision. B is a
   decision, priced as a major spec change under ADR, with A costed and
   the reasons on rows (§1.1–1.5) rather than on the code's existence. The
   test the pass applied: *would B be recommended if the code did not
   exist?* — yes: nineteen rows say the file route is client-readable on
   every tier at the client baseline, and two rows say the UDisks2 route's
   daemon is not there by default. The code follows the rows, not the
   other way round.
2. **Does moving the conjunct "widen below the floor"?** Section 9
   forbids the *engine* widening at runtime; it says the floors are
   "changeable only via ADR". B changes the floor via ADR. The engine's
   arm stays exactly as fail-closed as it is; what changes is which
   conjuncts the row has.
3. **Is this a §1.11 item?** Two candidate pairs were checked: LIN-001
   ("use UDisks2 for discovery") against SAFE-002 — no conflict, D-Bus is
   unprivileged and client-readable; LIN-001 against Section 9's row — no
   conflict, they agree. The tension is requirement-versus-delivery and
   floor-versus-route, which is what this round and an ADR are for, as the
   increment-5 plan §7.3 concluded. Not filed.
4. **What does the read-only matrix lose under B?** Nothing it has: no
   operation invokes UDisks2, so no capability changes status. What it
   gains is a determinable floor. What WP-L110 loses: nothing — the
   authorization/mutation route is explicitly left to it, UDisks2
   included.
5. **Is the spec pricing right?** By `partman-spec-pricing-keys-on-text`:
   LIN-001's sentence "Use UDisks2 for discovery" becomes false for the
   product as specified (discovery is through files) → major; the floor
   row's sentence changes meaning → major; one bump covers both, 18.0.0.
   The pass looked for a reading that makes it minor ("LIN-001's
   discovery is the helper's HLP-002 re-discovery, so the client's files
   do not contradict it") and rejected it: UDisks2 is by design a
   *client* interface, and reading LIN-001 as helper-only would be the
   drift of §3.1 in another coat.
6. **Would A be cheaper than it looks?** A D-Bus client for read-only
   enumeration is a few hundred lines over `zbus`; the cost is not the
   code, it is the rows (a UDisks2-present guest per tier, every property
   re-entered), the provoked launch, and the floor blocking default
   installs. The pass could not make A's floor consequence go away
   without D, which is B.
7. **Open, and the decision owner's:** whether the ADR should *also* set
   the tool-floor entry now (`udisks2 ≥ 2.9`, basis "ADR-0054, relocated
   from Section 9 17.4.0") or leave it to WP-L110's first invocation per
   `format.md`. The plan prefers leaving it, because `format.md`'s rule
   exists so that a floor is testable by the package that owns the tool;
   the ADR carries the number so it is not lost.

## 4. The recommendation

**Take B.** One ADR (ADR-0054, "LIN-001's discovery route, and the UDisks2
floor moves to the tool it gates"), one spec change (18.0.0): LIN-001
revised to name the measured discovery route and to reserve UDisks2,
libblockdev or native tools for the helper's authorization/mutation
route under WP-L110's own decision; the Section 9 Debian/Ubuntu row
becomes "Debian 12 / Ubuntu 22.04 LTS; kernel ≥ 5.15", with a note that
the UDisks2 ≥ 2.9 floor is a tool floor (CAP-006 store) attached to the
operations that invoke it. Then WP-L100: `floor.rs` drops the UDisks2
conjunct from the Debian/Ubuntu row (the Arch row already carries
`NotInRow`), `Conjunct::Undetermined` keeps its arms for the unmeasured
shapes, `fields.md` §7's UDisks2 row is rewritten as "not a platform
conjunct; a tool floor, WP-L110's to probe", tests over the DR16–DR19
bytes assert `MeetsFloor` on every measured guest, the mutations include
"the UDisks2 conjunct re-added as assumed met" and "Debian/Ubuntu composed
met with the kernel undetermined"; a WP-020 sitting (r51). The engine
(WP-050) needs no change — `Undetermined` stays for the unparsable and
unreadable shapes, which is what it was built for.

Why B over A in one sentence: the record has nineteen rows for the file
route and two rows saying the UDisks2 route's daemon is not installed by
default on two tiers; a floor that blocks an inventory which opens nothing
on every default install is not what "floor" was written to mean, and
Section 9's own last sentence already says where a tool's version floor
belongs.

## 5. Open questions for the decision owner

1. B, A, or C-as-a-stopgap-while-deciding? (C is recommended only as B's
   probe later, not as a decision.)
2. The tool-floor entry now or at first invocation (§3.7)?
3. Should the ADR pre-decide WP-L110's authorization route toward UDisks2,
   or leave it fully open? The plan says leave it open: HLP-002 makes the
   helper's discovery its own, and nothing measured bears on it yet.

## 6. What would change this round's mind

- A text under which a Section 9 conjunct gates the whole matrix
  regardless of use — none found; the closing sentence runs the other way.
- A row showing the UDisks2 route client-readable *and present by default*
  on every tier — then A's floor consequence disappears and A becomes a
  question of provenance and dependency only.
- The decision owner wanting UDisks2 as the client's interface for reasons
  outside these texts (desktop integration, udisks-mediated mounts for
  INV-006's "never auto-mount" discipline) — then A, with the floor
  consequence accepted and stated.

## 7. Next acts, in order

1. This round (WP-000, `docs/reviews/`). Decision.
2. Governance PR adding `docs/adr/0054-…` to the catalogue (ownership
   reads the catalogue from the base, so an act cannot widen its own
   assignment; two PRs, never both trailers on one commit).
3. ADR-0054 + `AGENT_BUILD_SPEC.md` (LIN-001; Section 9 row; document
   control 18.0.0; changelog row), `spec-change` label.
4. WP-L100: the floor conjunction without UDisks2, tests, mutations,
   `fields.md`, records; r51; re-pin. gitea#1010's "What waits" and the
   WP-L100 "Beyond these five" list updated; the `Undetermined` remediation
   text no longer names UDisks2 on a measured host.
