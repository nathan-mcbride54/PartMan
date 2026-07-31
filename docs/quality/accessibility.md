# Desktop accessibility evidence and limits

This document describes the evidence available for the WP-030 increment-2
desktop shell. It is deliberately narrower than a WCAG conformance statement.
The shell is a synthetic, read-only presentation surface: it performs no device
discovery, storage-management API or block-device I/O, planning, execution,
native command invocation, or elevation. Tauri and its webview still read the
packaged application and may maintain ordinary runtime caches.

## What is enforced automatically

| Boundary | Evidence | Claim |
| --- | --- | --- |
| Canonical visual language | `cargo xtask tokens` | Every declared text and UI pairing meets its independent WCAG 2.2 AA contrast floor in dark, light, and high-contrast themes. Meaning-bearing roles also carry an icon, label, and shape, and the colour-vision simulation remains above the project floor. |
| Typed renderer boundary | `packages/design-tokens/generate.mjs` and `npm run policy:colors` | The generated TypeScript API is the primary boundary: components can request only declared pairings, semantic roles, and supported shapes. A semantic role sets a private colour variable for a glyph, border, or shape but cannot be applied as text by a generated helper; labels and selectable controls carry an explicit audited text/surface pairing. A lexical source scan adds defence in depth against enumerated colour literals, raw palette variables, hard-coded generated classes, and inline colour-bearing styles; it is not a computed-CSS proof. |
| Shell structure and interaction | `apps/desktop/src/shell.test.tsx` and `apps/desktop/src/interactions.test.tsx` | Server-rendered markup contains the device rail, topology map, inspector, and a permanently mounted pending-plan region whose narrow-layout state is hidden rather than absent. Browser-like DOM tests activate theme changes, device and node selection, inspector updates, keyboard device selection, focus order, and drawer toggling while checking the corresponding ARIA state and valid `aria-controls` target. Topology sizing uses CSP-safe data attributes rather than inline styles. |
| Size and language boundary | `apps/desktop/src/format-bytes.ts`, `apps/desktop/src/strings.ts`, typed preview data, and their tests | Device/node sizes and byte-valued inspector facts stay `bigint` until a locale-aware formatter derives IEC units and grouped exact bytes from the same value. Shell labels, count grammar, theme names, semantic-role labels, formatter locale, and health-state presentation are outside components in the English catalogue. Preview content remains input data rather than component-owned copy. |
| Native webview boundary | `apps/desktop/src/security-boundary.test.ts` | The only capability names the main window and grants no native permissions. The CSP allows packaged same-origin script, style, font, and image content; network connections, objects, frames, form submission, and base-URL changes are denied. |
| Production artifact | `cargo xtask desktop` | Pinned npm dependencies are installed; generated output, lint, policy, type checking, and tests pass; Vite builds the web assets; and Tauri embeds them in a release executable with `--no-bundle` while forwarding Cargo's `--locked`. No dev server is the production content source, and manifest/lock drift is refused. |

The shell uses native `button`, `select`, heading, landmark, definition-list, and
ordered-list elements. Device selection exposes `aria-current`; topology
selection exposes `aria-pressed` plus a visible “Selected” label; the plan
toggle exposes `aria-expanded` and `aria-controls`, and its controlled region
remains mounted while hidden. All eight entity roles pair colour with text plus
generated icon and shape channels. Illustrative risk pairs colour with its
localized label and generated icon; health is localized text. None of these
claims depends on colour alone.

## Manual rendered-state check

On 2026-07-31 the final post-token/CSP local synthetic preview was inspected in
Chromium against the Vite development server at 1265 by 720 and 640 by 720 CSS
pixels:

- dark and high-contrast themes rendered from the canonical generated styles;
- device and topology selection updated the inspector and preserved visible
  selection text;
- the APFS container used the generated double-border shape rather than a
  role-specific hand-written mapping;
- the narrow layout had no page-level horizontal overflow, started with its
  plan drawer collapsed, and remained usable with the drawer expanded;
- the accessibility snapshot exposed all four UI-002 regions, labelled native
  controls, exact device and node byte values, and plan list semantics.

This was a visual and semantic development check over synthetic data, not an
automated conformance test, a native Tauri-webview result, or evidence from
assistive technology.

## What increment 2 does not establish

UI-008 is not complete. In particular, the repository does not yet automate:

- a complete keyboard traversal and focus-order assertion for every responsive
  state beyond the current theme, device-selection, and drawer-control checks;
- screen-reader checks in Windows, macOS, and Linux webviews;
- reflow and readability at 200% zoom;
- reduced-motion behavior beyond the current motion-free shell and global
  reduced-motion safeguard;
- computed-style comparison between every declared pairing and every rendered
  state;
- focus restoration and live-region behavior for future modal, planning,
  authorization, progress, error, and recovery flows.

The colour-vision calculation is a design smell test, not evidence of human
perception. The current semantic snapshot is not a substitute for NVDA,
VoiceOver, or Orca. The four-region shell also contains no real inventory,
planner, Apply path, progress state machine, or privileged helper, so it cannot
exercise ACC-011.

WP-030 increment 3 must add rendered computed-style assertions, keyboard and
focus-order tests, a 200% zoom/reflow matrix, reduced-motion tests, and
screen-reader validation on supported platform webviews. Those tests must keep
using synthetic data and remain unprivileged.
