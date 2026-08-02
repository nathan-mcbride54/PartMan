# ADR-0010: Replace React with Svelte in the required desktop UI stack

- Status: Proposed
- Date: 2026-08-01
- Spec version: 4.1.0
- Work packages blocked: any WP-030 desktop-shell increment, and any proposal to
  implement Section 4.1's UI layer
- Requirement IDs: UI-001, UI-002, UI-003, UI-007, UI-008, UI-011, UI-013,
  ACC-011, SEC-005, SEC-007, SEC-010, SAFE-008, Section 4.1, Section 9,
  Section 12
- Decision owners: @nathan-mcbride54

## Context

Section 4.1 requires "UI: React and TypeScript" and permits a change to the
stack "only through an architecture decision record showing safety, packaging,
and cross-platform consequences." This is that record.

**Three facts make now the right moment to decide and the wrong moment to
build.**

First, **nothing on `main` is React-specific.** The React implementation only
ever existed on the Tauri comparison branch, which PR #85 closed without merge.
`packages/canonical` is plain TypeScript and framework-independent;
`schemas/design-tokens.json` is data. So the cost of changing this decision
today is the cost of editing one line of the specification, and it rises the
moment anyone writes a component.

Second, **the incumbent was never tested.** ADR-0009 put Slint 1.17.1 through 41
gates and it failed two. Tauri with React has been through none of them. The
Tauri baseline exists as measured comparison evidence, not as an approval, and
PR #91 retired all temporary implementation authority. Treating React as
"already decided" mistakes an un-exercised default for a survived one.

Third, **the accessibility record is the project's oldest open claim.** All ten
`G-AX-*` gates in the Slint report are inconclusive. UI-008's rendered half —
keyboard-only operation (ACC-011), screen-reader semantics, 200% zoom, text
spacing, high contrast, reduced motion — has never been demonstrated by any
candidate. Whatever renders PartMan's first surface has to close those, so the
toolchain's own accessibility posture is a selection criterion rather than a
detail.

## Safety analysis

**No effect on the safety boundary.** The UI framework sits above the RPC
contract (Section 4.5). It performs no raw block write, launches no privileged
command, and does not participate in identity, validation, journaling, or
recovery. Section 4.2 forbids the desktop UI from raw block writes and direct
privileged commands regardless of what renders it. This ADR does not touch
SAFE-001 through SAFE-009 and weakens no MUST.

**Where a UI framework choice does reach safety**, and how Svelte is judged on
each:

- **SEC-007, offline operation.** Core functionality must work fully offline.
  A framework whose idioms assume a server is a hazard here — not because it
  cannot be configured otherwise, but because the default shape of its
  documentation and examples teaches the wrong pattern to whoever writes the
  next component. This is the whole reason this ADR proposes Svelte and
  explicitly *not* SvelteKit; see Option C.
- **SAFE-008, helper isolation.** The privileged helper performs no network
  I/O and loads no plugin. Nothing in the front end may create a path to it
  other than the versioned RPC. Framework-neutral, but a meta-framework's
  server routes are an invitation this project should not accept.
- **SEC-010 and SEC-005, supply chain and licence.** This is what actually
  killed Slint: two unmaintained transitive crates with no safe upgrade, and
  two BSL-1.0 packages that would have required widening the allow-list. The
  npm graph is audited by the same policy — `cargo xtask cross-language` walks
  every `package.json`, requires a committed `package-lock.json`, and refuses a
  search that matched nothing. **A smaller dependency graph is therefore a
  safety property in this repository, not a preference.**
- **UI-008 and ACC-011.** Accessibility failures in a destructive tool are
  safety failures: a confirmation dialogue a screen-reader user cannot parse is
  a consent mechanism that did not obtain consent.

**This ADR establishes none of that by measurement.** It is a decision about
which stack the project intends, made while the cost is one line. The evidence
obligations are in "Verification" and they are deliberately deferred, because
supply-chain evidence has a shelf life — ADR-0009's findings are pinned to
exact versions on a stated date, and a graph audited today says nothing about
the graph built in six months.

## Options considered

### Option A — Keep React and TypeScript

**Benefits.** No specification change. The largest component ecosystem, the
deepest accessibility tooling tradition (`eslint-plugin-jsx-a11y`, established
patterns for focus management and live regions), and the widest pool of prior
art for the kind of dense inspector UI Section 7.11 describes.

**Costs.** React plus a build toolchain is the larger npm graph, and graph size
is the axis that rejected the previous candidate. Accessibility linting is
opt-in tooling rather than a compiler property, so it can be removed or
misconfigured without the build noticing.

**Failure mode.** React is retained by inertia rather than by evidence, and the
project repeats with React the mistake it avoided with Slint: shipping a stack
nobody put through the gates.

### Option B — Svelte with Vite (recommended)

**Benefits.** A compiler rather than a runtime library: less shipped code and a
smaller dependency graph, which is the axis this repository's supply-chain
policy actually measures. **Accessibility warnings are emitted by the Svelte
compiler itself**, so a11y regressions surface at build time rather than
depending on a lint plugin staying installed and configured — directly relevant
to ten gates that are currently inconclusive. Plain Vite is a build tool this
repository already understands from the Tauri baseline.

**Costs.** A smaller ecosystem for complex widgets; UI-002's topology map may
need a graph or canvas library chosen on its own merits. Testing uses
`@testing-library/svelte` rather than the React equivalent. Svelte 5's runes are
a substantial change from Svelte 4, so the version must be pinned and evaluated
as itself rather than as "Svelte".

**Failure mode.** The compiler's accessibility warnings are treated as
sufficient. They are not: they catch static markup errors, not keyboard flow,
focus order, or live-region behaviour under a real screen reader. The `G-AX-*`
gates stay open regardless of framework.

### Option C — SvelteKit

**Rejected, and this is the substantive technical judgement in this ADR.**

SvelteKit is a meta-framework organised around server-side rendering, routing,
server endpoints, form actions, and deployment adapters. A Tauri desktop shell
wants a static bundle, so adoption begins by disabling most of it —
`adapter-static`, `ssr = false` — leaving Vite underneath plus an adapter layer
that exists to be switched off.

The cost is not the extra dependencies. It is that **server-side concepts become
available in a product that must work fully offline (SEC-007) and must keep
network I/O out of the privileged path (SAFE-008).** Load functions, server
routes and form actions are the framework's documented idioms; every example the
next contributor reads will use them. Choosing a tool whose happy path
contradicts two of the product's constraints is how a constraint gets violated
by someone who was following the documentation.

### Option D — No web stack at all

Considered and out of scope here. ADR-0009 evaluated one native alternative and
rejected it on supply chain. Re-opening the shell question belongs in its own
record with its own evidence, not folded into a UI-layer decision.

## Decision

**Section 4.1's UI line becomes "UI: Svelte and TypeScript."** SvelteKit is
explicitly excluded; the build tool is Vite.

**This decision does not approve a desktop shell.** Section 4.1 continues to
name Tauri 2, and Tauri has never been through the gates ADR-0009 applied to
Slint. Changing the UI layer does not confer on the shell layer an approval it
never earned. No WP-030 implementation authority is created or restored by this
ADR; PR #91's retirement stands, and building a shell needs its own governance.

## Consequences

- Section 4.1 changes one line. This is a specification change and takes a
  version bump.
- No code changes. `main` has no UI, so there is nothing to port.
- `packages/canonical` and `schemas/design-tokens.json` are unaffected; both are
  framework-independent, and the design-token file remains the single source of
  truth that any front end must read rather than restate.
- The Tauri comparison baseline (PR #85) and the Slint candidate (PR #89) remain
  closed, unmerged historical evidence. Neither becomes reachable by this
  change.
- The ten `G-AX-*` accessibility gates remain **inconclusive**. This ADR does
  not close any of them and must not be cited as if it had.

## Verification

Deferred deliberately, and owed in full before any Svelte code merges:

1. `cargo xtask cross-language` passes with the Svelte package present, its
   `package-lock.json` committed, and every dependency audited.
2. No BSL-1.0 or otherwise non-allow-listed licence appears in the resolved npm
   graph, and the allow-list is **not** widened to accommodate one. This is the
   exact failure that rejected Slint.
3. No unmaintained-crate-equivalent advisory without a safe upgrade path.
4. A recorded comparison of the resolved graph against the Tauri/React baseline
   measured at `b0f1124`, so "smaller graph" is a measurement rather than a
   claim.
5. The `G-AX-*` gates are attempted rather than inherited: keyboard-only
   operation, screen-reader semantics, 200% zoom, text spacing, high contrast
   and reduced motion, each demonstrated or recorded as still inconclusive.

Until those exist, the correct description of this decision is "intended
stack", not "validated stack".

## Revisit conditions

- Svelte's resolved npm graph fails the supply-chain or licence policy at the
  time a shell is actually authorized. The decision is reversible precisely
  because no code depends on it.
- The `G-AX-*` evidence shows the compiler's accessibility posture does not
  translate into a measurably better outcome for the rendered gates.
- A future shell decision retires Tauri, which may change what the UI layer
  should be — or remove the question.
- Svelte's major version changes its programming model again in a way that
  invalidates the graph-size or accessibility reasoning above.
