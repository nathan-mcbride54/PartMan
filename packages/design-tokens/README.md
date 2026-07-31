# PartMan design-token accessors

This directory is generated from the single source of truth at
`schemas/design-tokens.json`.

`generate.mjs --write` emits:

- `src/generated.ts`, typed accessors for complete foreground/background
  pairings, foreground-only text, borders, focus outlines, semantic roles, and
  non-colour shapes;
- `src/generated.css`, the private theme variables plus the audited atomic
  classes returned by those accessors.

Hand-written UI code never names a generated class or a raw
`--pm-color-*` variable. It chooses one of the declared tuples in `pairs` and
passes it to `pairClass`, `foregroundClass`, `borderClass`, or `outlineClass`.
Meaning-bearing components use `roleClass` for the private semantic-colour
custom property and `shapeClass` for their non-colour geometry. Text still uses
an explicit text pairing; this matters for roles such as `entity.freeSpace`
whose semantic colour is intentionally audited only at the 3:1 non-text floor.
The generated API deliberately exposes no helper that applies a meaning role
as a text foreground, so a UI-only role cannot become normal text through a
type-correct call. Both `pairClass` and `foregroundClass` accept only the
generated `TextContrastPair` union, preventing a 3:1 UI-only pairing from being
used as normal text. A palette or shape change in the canonical token data
therefore reaches the renderer through generated output rather than a second
mapping.

The desktop gate runs the generator in check mode and fails on any byte drift.
As defence in depth, it also scans the hand-written desktop and shared-UI
sources for enumerated colour literals, raw generated variables, hard-coded
generated classes, and inline colour-bearing styles. The generated typed API,
not that lexical scan, is the primary boundary.
The generated files are committed so review shows every visual-language change
and a Tauri build never depends on a generator side effect.
