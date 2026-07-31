# ADR-0009: Bound Slint adoption to a production-feasibility evaluation

- Status: Accepted
- Date: 2026-07-31
- Spec version: 4.1.0
- Work packages blocked: WP-030 Slint feasibility work and any proposal to
  replace the required Tauri 2, React, and TypeScript desktop stack
- Requirement IDs: SAFE-001, SAFE-002, SAFE-004, SAFE-009, MODEL-001, UI-001, UI-002,
  UI-003, UI-007, UI-008, UI-011, UI-013, PLAN-004, SEC-005, SEC-007,
  SEC-010, PKG-001, PKG-002, PKG-003, PKG-004, PKG-005, Section 4.1,
  Section 9, Section 11.7, Section 12
- Decision owners: @nathan-mcbride54

## Context

Section 4.1 currently requires Tauri 2 with React and TypeScript. That decision
avoids bundling Chromium in the application, but it still makes the desktop
surface a web application hosted by the operating system's web view. The
unmerged WP-030 increment-2 implementation demonstrates that this is workable:
its complete desktop gate passes, including 31 desktop tests and a native
release build. It also demonstrates the cost. On Linux, Tauri 2.11.5 reaches
the archived GTK3 Rust bindings and needs fifteen exact advisory exceptions,
including a guarded unsound `glib` advisory. On Windows, the small synthetic
shell starts a seven-process WebView2 tree.

The decision owner would prefer not to ship a browser-shaped desktop stack when
a production-suitable native toolkit can meet the same requirements with less
runtime and supply-chain weight. Slint is the strongest candidate found so far.
It is not Electron and does not require a browser engine or Node.js at runtime.
Its `.slint` declarative UI is compiled ahead of time and linked with a Rust
runtime, a window-system backend, and a selected renderer. That makes it
architecturally attractive for PartMan's Rust-heavy, unprivileged desktop
shell.

Attractiveness is not compatibility. The evaluated release is Slint 1.17.1,
published 2026-07-07 and confirmed as the latest stable GitHub release on
2026-07-31. Its first-party material names material limitations that intersect
PartMan directly:

- Slint's published desktop test matrix covers Windows 10 x86-64 and Windows
  11 x86-64/aarch64, but on macOS it lists only versions 14, 15, and 26 on
  aarch64. PartMan's required floor is macOS 13 on both Apple Silicon and Intel.
  Compilation on a current hosted runner cannot close that gap.
- Upstream's Linux statement targets the most recent LTS or newer and assumes
  glibc, D-Bus, and either X11 or Wayland. Debian 12 and Ubuntu 22.04 are older
  PartMan floors, so the general Linux statement does not qualify them.
- The lightweight software renderer has no transform rotation or scaling,
  shadows, text outline, rounded clipping, or layer opacity, and its text
  rendering is currently limited to Western scripts. PartMan must display
  user-controlled labels and paths faithfully even though its v1 product
  strings are English.
- FemtoVG requires OpenGL and documents sometimes sub-optimal
  text and path quality. Skia has broader rendering support but is explicitly
  described as having a heavy disk footprint.
- Disabling Slint's public default features does not make its internal graph
  codec-free. The target graph still enables `image` JPEG/PNG support through
  `i-slint-core` and resolves the `resvg` SVG stack and its own transitive
  decoders. Slint's convenience
  `slint-build` crate would additionally pull the full default `image` codec
  set into the build-host graph even when an application uses no image asset.
  A bounded PartMan-owned AOT adapter can call the already-resolved pinned
  compiler without that feature uplift, but it creates an explicit internal-API
  maintenance obligation. Both dependency surfaces still need inventory,
  audit, and footprint evidence.
- Upstream warns that Rust applications can exhaust the smaller default MSVC
  main-thread stack on Windows, especially in debug builds, and recommends a
  linker stack increase. A partitioning UI must prove deep synthetic topology
  does not terminate the process; a workspace-wide `rustflags` workaround is
  prohibited by repository policy.
- The standard accessibility surface exposes OS accessibility APIs, landmark
  roles, stable accessibility IDs, and selection state, but an API existing is
  not evidence that PartMan's custom topology is usable with NVDA, Narrator,
  VoiceOver, and Orca.
- Owned/modal window semantics remain an open upstream feature. `PopupWindow` was
  created for combo boxes and still has open general-purpose limitations. A
  TreeView role exists, but the standard TreeView widget is not provided.
- `Text` and `TextInput` expose letter spacing but not word spacing or line
  height. WCAG 1.4.12 text-spacing evidence therefore needs an explicit layout
  and string strategy; toggling a style property cannot establish compliance.
- Slint builds an executable, not a signed installer, notarized application,
  updater, rollback system, or Linux package. Those remain PartMan work.
- Slint is triple-licensed. The GPLv3 arm is incompatible with ADR-0006's
  binding prohibition on linking GPL libraries. The royalty-free desktop
  license is usable without changing PartMan's source license, but distribution
  requires Slint attribution. A commercial license is the other non-GPL path.

The relevant primary sources, pinned to the evaluated release where possible,
are:

- <https://github.com/slint-ui/slint/releases/tag/v1.17.1>
- <https://raw.githubusercontent.com/slint-ui/slint/v1.17.1/api/rs/slint/Cargo.toml>
- <https://raw.githubusercontent.com/slint-ui/slint/v1.17.1/LICENSE.md>
- <https://github.com/slint-ui/slint/blob/v1.17.1/LICENSES/LicenseRef-Slint-Royalty-free-2.0.md>
- <https://github.com/slint-ui/slint/blob/v1.17.1/SECURITY.md>
- <https://github.com/slint-ui/slint/blob/v1.17.1/docs/astro/src/content/docs/guide/platforms/desktop.mdx>
- <https://github.com/slint-ui/slint/blob/v1.17.1/docs/astro/src/content/docs/guide/backends-and-renderers/backends_and_renderers.mdx>
- <https://github.com/slint-ui/slint/blob/v1.17.1/docs/astro/src/content/docs/guide/backends-and-renderers/backend_winit.md>
- <https://github.com/slint-ui/slint/blob/v1.17.1/docs/astro/src/content/docs/reference/std-widgets/style.mdx>
- <https://github.com/slint-ui/slint/blob/v1.17.1/docs/astro/src/content/docs/reference/common.mdx>
- <https://github.com/slint-ui/slint/blob/v1.17.1/api/rs/build/lib.rs>
- <https://github.com/slint-ui/slint/blob/v1.17.1/api/rs/build/Cargo.toml>
- <https://github.com/slint-ui/slint/blob/v1.17.1/api/rs/macros/Cargo.toml>
- <https://github.com/slint-ui/slint/blob/v1.17.1/internal/compiler/lib.rs>
- <https://github.com/slint-ui/slint/blob/v1.17.1/internal/compiler/Cargo.toml>
- <https://github.com/slint-ui/slint/blob/v1.17.1/internal/core/Cargo.toml>
- <https://github.com/slint-ui/slint/blob/v1.17.1/internal/backends/selector/api.rs>
- <https://github.com/slint-ui/slint/blob/v1.17.1/internal/backends/winit/lib.rs>
- <https://github.com/slint-ui/slint/blob/v1.17.1/Cargo.lock>
- <https://github.com/image-rs/image/blob/v0.25.10/Cargo.toml>
- <https://github.com/rust-skia/rust-skia/tree/0.99.0>
- <https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-env>
- <https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-env-changed>
- <https://github.com/slint-ui/slint/issues/6607>
- <https://github.com/slint-ui/slint/issues/1143>
- <https://github.com/slint-ui/slint/issues/505>

This cannot be decided from dependency count, a screenshot, or an upstream
support table. It needs one bounded implementation measured against the Tauri
shell on the exact platforms and interaction boundaries PartMan promises.

## Safety analysis

The candidate application is presentation-only. It MUST use synthetic topology
data and MUST NOT discover devices, invoke a storage utility, expose a native
command bridge, start or contact a helper, request elevation, write a plan, or
simulate a successful storage operation. The external qualification harness may
perform read-only inventory of named, disposable VM state and generated
synthetic disks solely to detect packaging effects under SAFE-001. It never
passes discovered identifiers to the candidate, opens a real host disk, or
performs a storage mutation. SAFE-002 and SAFE-004 therefore remain unchanged.
No destructive test tier becomes available through this ADR.

The UI remains outside the eventual authorization boundary. A future helper
will independently re-probe identity and validate an authorized plan under
HLP-002 and HLP-003 regardless of which toolkit draws the client. Slint state,
callbacks, accessibility properties, and formatted values are untrusted
presentation inputs; none may become proof of device identity or authorization.

Exact storage values remain `u64` in Rust. The Slint boundary receives
preformatted IEC display text, preformatted exact-byte text, and bounded visual
weights rather than converting authoritative byte counts through a renderer
numeric type. This preserves UI-013 and avoids turning layout arithmetic into a
lossy storage model.

The evaluation MUST keep every PartMan-authored user-facing string outside
`.slint` markup, including accessibility names, empty states, and errors.
Platform-owned native-dialog text is recorded but is not a second PartMan
catalogue. Slint `SharedString` is UTF-8, while a Linux path can contain
arbitrary bytes and a Windows path can contain ill-formed UTF-16. Raw path and
device identifiers therefore remain in Rust and are never replaced by the
string sent to Slint. The view model carries a separate stable opaque ID, a full
collision-safe escaped representation that round-trips every original code
unit, and a separately bounded visual string that is not claimed to be
reversible. Bounded text is cut only between original grapheme clusters or
whole escape tokens and receives an unambiguous truncation marker; it never
splits an ASCII escape sequence or changes the underlying identifier. Tests
MUST cover invalid UTF-8, ill-formed UTF-16, NUL and newline, bidi controls,
literal backslashes and escape lookalikes, combining sequences, whole-token and
grapheme truncation, and bounded rendering of very long labels.

No Slint runtime inspection or control server may reach a release-shaped build.
In particular, `mcp`, `system-testing`, and `live-preview` are development
features that enable remote or runtime control and are excluded. The evaluation
uses compile-time `.slint` generation, never the interpreter.

The royalty-free license is elected for the evaluation and for any distributed
candidate binary. PartMan remains `MIT OR Apache-2.0`; this election applies to
the linked Slint framework, not to PartMan's own source. Before any candidate
binary is distributed, it MUST satisfy Slint Royalty-free License 2.0's
attribution condition and PKG-004's shipped-license and notice requirements.
The elected route is the license's public-webpage alternative: display Slint's
official attribution badge on a public page where PartMan binaries are
downloaded, in a position easily found by any visitor. This avoids depending on
the pinned `AboutSlint` widget, whose fixed pointer-only external-link action is
not sufficient evidence for UI-008 keyboard parity. Automated readback of the
published download page and badge target MUST pass before any linked binary is
uploaded or otherwise distributed. The application itself performs no
automatic or in-process network access and contains no attribution URL action;
core functionality remains fully offline. Applicable licenses and third-party
notices also ship in the package. If the public badge route is unavailable or
unacceptable, the candidate requires a commercial Slint license or fails. The
GPL arm is not a fallback.

## Options considered

### Option A — Keep Tauri 2, React, and TypeScript without an evaluation

This obeys the current specification and preserves an already substantial
implementation. Tauri is mature, brings DOM-based accessibility and testing,
and uses the operating system web view rather than bundling Electron.

Rejected as the immediate decision. The measured process footprint and the
Linux GTK3/advisory boundary are large enough to justify testing a native
alternative before the first desktop shell lands. Tauri remains the production
default while that test runs.

### Option B — Adopt Slint immediately

This would remove the web front end and its Node application toolchain quickly.

Rejected. It would change a normative architecture before establishing the
macOS floor, accessibility, renderer correctness, packaging, licensing, or an
actual footprint improvement. Calling a toolkit native does not prove any of
those properties.

### Option C — Maintain a PartMan fork of Slint or rewrite its needed parts

Owning the code locally sounds like protection from dependency churn.

Rejected. A cross-platform window backend, text shaper, GPU and software
renderer, input-method layer, and Windows UI Automation/macOS AX/Linux AT-SPI
bridge are not bounded application components. Rewriting them would make
PartMan the maintainer of a second product whose least visible failures are
exactly the accessibility, international text, and platform-safety failures
this evaluation must detect. Forking would also make upstream security fixes a
manual merge obligation. PartMan will own its view model, components, token
generator, and tests; it will not vendor or fork Slint.

This rejection does not prohibit a small build adapter. Such an adapter owns
only Cargo integration around the pinned compiler: configuration, diagnostics,
dependency tracking, generated-file output, and environment rejection. It
does not copy or replace parser, compiler, renderer, platform, or accessibility
code. Its bounded surface and compile-time failure on an incompatible exact
re-pin are materially different from maintaining a toolkit fork.

### Option D — Carry Tauri and Slint as permanent desktop implementations

This would preserve a fallback and let platforms choose independently.

Rejected. Two production stacks double every accessibility, packaging,
localization, and security review surface, and make design-token drift likely.
The Tauri snapshot is a measurement and behavior baseline, not a second product
to maintain after a decision.

### Option E — Run a bounded Slint feasibility evaluation

Build the same safe, synthetic four-region shell with native Rust view models,
generated design-token bindings, explicit Slint features, and evidence gates.
Keep the implementation in a review branch until the result is known.

## Decision

**Conduct Option E. Tauri 2, React, and TypeScript remain the normative
production stack until the evaluation passes every hard gate and a later major
specification change adopts Slint.** This ADR authorizes evidence gathering; it
does not itself amend Section 4.1 or claim that Slint is supported.

That authorization becomes usable only in this order:

1. Merge this ADR as a WP-000 change.
2. Merge a governance-only pull request that edits only
   `docs/work-packages/WP-030.md`, with the exact trailer
   `Governance: authorize WP-030 Slint feasibility under ADR-0009`.
3. Start the Slint branch from the resulting `origin/main` revision.

The governance change MUST reserve the exact feasibility surface before any
implementation commit: the `apps/desktop/**` Rust workspace member and
`apps/desktop/packaging-feasibility/**` non-shipping probes;
`tools/slint-feasibility/**`; normalized
`docs/quality/slint-feasibility-data/**`; continued authority for
`schemas/design-tokens.json` and `crates/tokens/**`; `Cargo.toml` and
`deny.toml`; the exact
`tools/xtask`, CI, Dependabot, dependency-policy, test-tier, accessibility,
traceability, status, and changelog files it may change; and the generated
result `docs/quality/slint-feasibility.md`. `Cargo.lock` remains owned by WP-000
and may accompany a WP-030 manifest change only through the existing verified
`derived-paths` rule; WP-030 receives no standalone lockfile authority. The
implementation uses `Work-Package: WP-030`. The verifier reads assignment
authority from the base revision, so an implementation cannot grant itself
ownership or mix a governance trailer with application changes.

The candidate is pinned to Slint 1.17.1 and starts with this intentionally small
feature surface:

```toml
[features]
default = ["renderer-femtovg"]
renderer-femtovg = ["slint/renderer-femtovg"]
renderer-software = ["slint/renderer-software"]
comparison-combined = ["renderer-femtovg", "renderer-software"]

[dependencies]
slint = {
  version = "=1.17.1",
  default-features = false,
  features = ["std", "backend-winit", "accessibility", "compat-1-2"],
}
unicode-segmentation = "=1.13.3"

[build-dependencies]
i-slint-compiler = {
  version = "=1.17.1",
  default-features = false,
  features = ["rust", "display-diagnostics"],
}
partman-tokens = { path = "../../crates/tokens" }
spin_on = "=0.1.1"
```

Slint 1.17.1's `slint-build` crate unconditionally enables
`i-slint-compiler`'s build-host `software-renderer` and `bundle-translations`
features. In the evaluated graph, `software-renderer` enables `image` 0.25.10
with its `default` feature, Rayon, and the default
AVIF/BMP/DDS/EXR/Farbfeld/GIF/HDR/ICO/JPEG/PNG/PNM/QOI/TGA/TIFF/WebP codec set.
That capability is unnecessary because this candidate embeds no PartMan image
or font, performs no compile-time texture rendering, and uses no Slint
translation bundle.

The evaluation therefore does not resolve `slint-build`. Its PartMan-owned AOT
adapter directly uses exact `i-slint-compiler` 1.17.1 with only `rust` and
`display-diagnostics`, plus exact `spin_on` 0.1.1. The public `slint` crate
already unconditionally resolves `slint-macros`, whose host graph resolves the
same compiler and `spin_on` with `rust`, `display-diagnostics`, and
`proc_macro_span`; the direct build dependencies introduce neither a second
compiler nor a broader compiler capability. The compiler's empty `default`
feature may remain enabled through `slint-macros`, but
`software-renderer`, `bundle-translations`, and their codec uplift must remain
absent. A paired Windows probe generated, type-checked, and linked a multi-file
component through default FemtoVG, software-only, and combined configurations;
each adapter feature tree had 58 fewer reachable packages than its equivalent
`slint-build` control. Platform lock graphs remain authoritative; the probe
delta is rationale, not a cross-platform prediction. The probe observed no
executable-size change, so the improvement is credited only to build-time
supply-chain and compilation exposure, never to a runtime-footprint gate.

`i-slint-compiler` is explicitly internal and not semver-stable. Exact pinning,
the owned adapter's fixture suite, and a mandatory source/API review on every
Slint re-pin contain that risk. An incompatible release must fail compilation
until the adapter and its evidence are consciously updated; floating the
compiler or falling back to `slint-build` is prohibited. This is a small,
reviewable maintenance burden in exchange for avoiding an unused build-time
renderer, translation bundler, and codec graph.

FemtoVG is the explicit default evaluation candidate so ordinary locked
workspace and all-target test commands always have a renderer. The software
candidate is built with `--no-default-features --features renderer-software`.
The `comparison-combined` marker exists only so the repository's mandatory
`--all-features` lint graph can compile the deliberate two-renderer control;
that graph is never adoption-eligible.

`backend-default`, Qt, system tray, the target-runtime
`image-default-formats` feature, live preview, MCP, system testing, the
interpreter, Skia, and every unstable feature are excluded. PartMan markup
imports no raster/SVG asset or custom font. The runtime's unavoidable `image`
JPEG/PNG feature path and complete `resvg` closure are inventoried honestly,
while the build-host compiler's software-renderer, translation bundler, Rayon,
and default image-codec uplift remain absent.
Slint 1.17.1 resolves `skia-safe` 0.99.0 for its Skia renderer; rust-skia's
normal build can download prebuilt archives and its source-build fallback has a
large separate toolchain and source-fetch boundary. That conflicts with this
evaluation's offline, pinned-input policy in addition to Skia's documented disk
cost. Skia is therefore not even a comparison feature of the workspace member.
If FemtoVG and software cannot qualify, this evaluation fails; a later,
separately scoped decision may study Skia with externally hash-verified inputs.
The release-graph gate must inspect resolved features rather than trusting this
prose.

The graph evaluator consumes locked Cargo metadata and manifest feature/target
edges directly, classifying runtime, build, and proc-macro reachability for each
evaluated host/target pair. It does not parse `cargo tree` presentation text or
reject a crate merely because Cargo lists it for another target or an inactive
edge. Human-readable trees remain review evidence, not the policy parser.

The UI is compiled ahead of time. `build.rs` calls the pinned compiler's
`parser::parse_file`, `compile_syntax_node`, and
`generator::generate(OutputFormat::Rust, ...)` APIs through PartMan-owned code.
It explicitly selects Rust output, embedded
resources, accessibility, and the Fluent style; disables experimental/debug
behavior, translations, native menus, and compile-time scaling; fails on every
compiler diagnostic; records the root, every loaded import, and every external
resource as Cargo dependencies; writes only beneath `OUT_DIR`; and uses no
`cargo:rustc-env` handoff. PartMan includes the fixed generated path with
`include!(concat!(env!("OUT_DIR"), "/partman_ui.rs"))`; it does not invoke
`slint::include_modules!` or the `slint!` procedural macro. This prevents Cargo
from propagating an internally created `SLINT_INCLUDE_GENERATED` value into
`cargo run` or `cargo test`, where the runtime prefix guard would correctly
reject it. A fixture suite proves nested import tracking, deterministic
generation, syntax and semantic failure, warnings-as-errors, missing-input
failure, and the prohibition on PartMan image, font, translation, and
native-style input. It does not attempt to reproduce unused `slint-build` APIs.

The adapter does not accept Slint's host-dependent `native` widget style. Clean
release builds with and without Qt installed produce identical generated-input
hashes and remain Fluent and Qt-free. Hostile `SLINT_STYLE` builds are expected
to stop at the ambient-input guard before invoking any compiler constructor;
they are not compared as successful artifacts.

Exact Cargo features are not a complete configuration boundary. The 1.17.1
source-derived inventory for this candidate is:

- build/AOT: `SLINT_EMBED_TEXTURES`, `SLINT_EMBED_RESOURCES`,
  `SLINT_INLINING`, `SLINT_SCALE_FACTOR`,
  `SLINT_ENABLE_EXPERIMENTAL_FEATURES`, `SLINT_EMIT_DEBUG_INFO`, `SLINT_STYLE`,
  and `SLINT_LIVE_PREVIEW`;
- runtime under both renderers: `SLINT_BACKEND`, `SLINT_DEBUG_PERFORMANCE`,
  `SLINT_DEFAULT_FONT`, `SLINT_DESTROY_WINDOW_ON_HIDE`, `SLINT_FONT_PATH`,
  `SLINT_FULLSCREEN`, `SLINT_SCALE_FACTOR`, and `SLINT_SLOW_ANIMATIONS`;
- software-renderer runtime only: `SLINT_LINE_BY_LINE` and
  `SLINT_SOFTWARE_RENDERER_PARLEY_DISABLED`; and
- resolved-source but excluded call/feature paths: `SLINT_MACRO_CACHE` because
  PartMan never invokes `slint!`; `SLINT_INCLUDE_GENERATED` because PartMan
  never invokes `include_modules!`; `SLINT_ASSET_SECTION` and
  `SLINT_FONT_SIZES` because compiler `software-renderer` is absent;
  `SLINT_BUNDLE_TRANSLATIONS` because translation bundling is absent;
  `SLINT_CPP_NAMESPACE` because C++ output is absent;
  `SLINT_COMPILER_DENY_WARNINGS` because `slint-build` is absent; and
  `SLINT_WGPU_CPU` because every WGPU feature is absent. The compiler's own
  build script creates `SLINT_WIDGETS_LIBRARY` for upstream compilation; it is
  recorded separately as an upstream-controlled value, not accepted as a
  downstream ambient input.

The committed inventory retains active and excluded names with source
locations and feature/call-path proof. A shared guard runs in the outer
`cargo xtask desktop` process before Cargo, again in `build.rs` before any
compiler constructor, and at the release entry point before UI creation. This
layering matters because Cargo may reuse a cached build-script result when an
unknown future variable changes. Every known name and a PartMan-only test nonce
are declared with `rerun-if-env-changed`; hostile direct Cargo tests vary the
nonce to force the defense-in-depth build guard to execute. The authoritative
per-invocation outer guard and content-addressed harness start Cargo with a
minimal allow-listed environment. All guards enumerate with `vars_os`, reject
the entire prefix ASCII-case-insensitively on Windows and byte-exactly on Unix,
and cover lowercase/mixed-case names, non-Unicode Unix names, every inventoried
name, and an unknown future-style `SLINT_*` name.

Backend choice is fixed to Winit with `BackendSelector` before any component is
created. A closed Rust enum maps only `femtovg` and `software` to exact renderer
strings; unknown values fail. Every
adoption-eligible process compiles exactly one renderer and selects it
programmatically, so its feature graph makes fallback to another renderer
impossible. The application fails compilation with no shipping renderer or with
multiple shipping renderers outside an explicitly marked comparison build.
Ambient `SLINT_BACKEND` values—including unsupported and hostile values—MUST NOT
change the fully specified programmatic request.

Slint 1.17.1's public Winit builder keeps a private, always-true fallback flag:
an unknown renderer name or a renderer-factory error can enter its fallback
search, while a later FemtoVG resume/OpenGL failure is not retried with
software. It also exposes no stable public runtime-renderer getter. An exact known name
and single-renderer graph prevent cross-renderer fallback, although the private
retry machinery may still execute and return the same failure.
The combined build launches a separate process for each explicit request and
records success only after a presented frame and accessibility root; Slint's
process-global platform cannot be selected twice in one process. Because the
stable public API exposes no runtime renderer getter, the ADR intentionally does
not treat the combined process's request as runtime attestation; its evidence
records that cross-renderer fallback may have occurred. It is a non-shipping
control. No automatic in-process FemtoVG-to-software recovery is claimed.
FemtoVG-only, software-only, and combined-control builds are measured
separately. The software renderer cannot qualify if it corrupts or omits
non-Western device labels.

`schemas/design-tokens.json` remains the only visual-language authority.
PartMan's Rust token crate generates a committed `.slint` interface; hand-written
components do not carry a second palette or call raw generated colors. Text and
UI-only contrast pairings remain distinct types so a 3:1 component pairing
cannot be used for normal text. The generated boundary also carries the eight
UI-003 roles and typed IDs for every semantic icon, label, and shape in the
canonical schema; it carries no English label value. Rust resolves display and
accessibility strings for those IDs through the external catalogue.
Qualification enumerates the generated vocabulary rather than relying on a
hard-coded count. The Fluent compile style exists only to make any upstream
widget compilation deterministic. Slint's compiler can silently inject style
defaults when visual builtins or layouts omit properties. A version-pinned
inventory of every compiler path that references `Palette` or `StyleMetrics`—
currently `apply_default_properties_from_style.rs`, `lower_layout.rs`, and
`passes/windows.rs`—therefore feeds generated wrappers and static AST/lowered-IR
checks. PartMan explicitly binds every affected Text, StyledText, TextInput,
Window, and Dialog visual property, layout padding/spacing, and window
background to a typed canonical token or a documented nonvisual value.
Where the current schema lacks a required typography, cursor, or layout value,
WP-030 uses its existing `schemas/design-tokens.json` and `crates/tokens/**`
authority to add the canonical value, independent policy, mutation coverage,
and token-set version change before generating the wrapper. No PartMan visual
property may be waived as a platform-owned default merely because the schema
does not yet name it.
PartMan-owned markup otherwise MUST NOT import Slint `Palette` or
`StyleMetrics`, and uses no standard widget that introduces an ungoverned
application palette. One generated, audited theme adapter may read only
`Palette.color-scheme` and map its `unknown`, `light`, and `dark` values to
canonical PartMan theme IDs; it cannot read Palette brushes or StyleMetrics.
High contrast is not represented by that enum and needs a separate platform
signal and evidence. Operating-system window chrome and native dialog chrome
are recorded platform surfaces, not PartMan-authored visual values.

The spike ports concepts, not the web stack: synthetic topology behavior,
selection retention, exact byte formatting, the external string catalogue,
four-region layout, semantic role vocabulary, focus expectations, and honest
accessibility limitations. It does not retain Tauri host configuration, React,
Vite, CSS, npm application dependencies, WebKit/GTK prerequisites, or the
Tauri-specific advisory exceptions in the Slint candidate graph.
`packages/canonical` and its TypeScript cross-language proof remain because
MODEL-005 still requires them.

The implementation pull request MUST remain unmerged while evidence is being
collected. A successful result follows this exact promotion sequence:

1. Preserve and review the passing spike commit, content-addressed harness,
   normalized evidence, and generated report on the unmerged WP-030 branch.
2. Merge a WP-000 adoption ADR that is explicit that it is accepted but has no
   operative effect while specification 4.1.0 still requires Tauri.
3. Merge a separate WP-000 specification-change pull request labelled
   `spec-change`, with a `Work-Package: WP-000` trailer, that bumps the
   specification to 5.0.0 and updates its Section 0.3 changelog, root
   `CHANGELOG.md`, Section 4.1, all dependent requirements, and traceability.
4. Merge a governance-only WP-030 promotion pull request that edits only
   `docs/work-packages/WP-030.md`, uses the exact trailer
   `Governance: promote WP-030 Slint implementation after Slint adoption`,
   authorizes production paths, and removes obsolete Tauri-only grants.
5. Rebase or update the implementation onto that base, rerun every applicable
   gate, review the changed evidence hashes, and only then merge it.

If the evaluation fails, the implementation pull request closes without
merging. A separate WP-030 evidence-only pull request then lands the normalized
`docs/quality/slint-feasibility-data/**` inputs, the non-product
`tools/slint-feasibility/**` generator and replay harness needed to reproduce
them, and the regenerated `docs/quality/slint-feasibility.md`. It cites the
immutable rejected spike commit and artifact hashes and describes the source as
non-production comparison evidence; it MUST NOT present source paths absent
from main as live implementation evidence. A final governance-only pull request
then edits only `docs/work-packages/WP-030.md`, uses the exact trailer
`Governance: retire failed WP-030 Slint feasibility authority under ADR-0009`,
and removes implementation, root-manifest, `deny.toml`, CI, and dependency-policy
authority while retaining only the exact evidence/generator/report paths that
landed. The comparison branch remains durable, but main never carries two
permanent desktop stacks.

## Qualification gates

Every normative gate below has a stable `G-*` identifier. The result generator
MUST emit exactly one row for every identifier, with an objective assertion,
evidence reference, and `pass`, `fail`, or `inconclusive` result. The decision
algorithm is mechanical:

- any failed hard gate rejects the candidate;
- any inconclusive hard gate blocks adoption without being relabelled pass;
- passing every hard gate and comparative threshold makes the candidate
  eligible for the promotion sequence above, not adopted by this ADR; and
- a `C-*` comparison control does not independently reject the candidate unless
  a named `G-*` threshold consumes it.

The merged WP-000 revision of this ADR is the authoritative gate inventory and
threshold source. The WP-030 generator parses that immutable revision (or a
machine-readable registry generated from it by WP-000) and cannot accept an
implementation-owned `pass` field. It computes every automatable outcome from
normalized evidence. A genuinely manual gate needs a structured attestation
naming the operator, platform/tool version, procedure, evidence hashes,
limitations, date, and an independent reviewer; absent or unreviewed
attestation is inconclusive.

### Configuration reproducibility

| ID | Objective assertion and required evidence |
| --- | --- |
| G-CFG-01 | At the final decision, the exact Slint pin is still the latest stable release supported by upstream's current security policy. If not, re-pin and rerun every gate. Archive the release/API response, pinned `SECURITY.md`, upstream GitHub advisories, RustSec results, and `cargo deny` results. |
| G-CFG-02 | Resolver-3 evidence separates host/build and target/runtime graphs. The target graph contains only approved Slint features and exactly one candidate renderer; its unavoidable `image` JPEG/PNG path and complete `resvg` closure are inventoried, while the marked all-features control is non-shipping and no Qt, system tray, Slint `image-default-formats`, Skia, live preview, MCP, system testing, interpreter, or unstable API is target-reachable. The host graph resolves no `slint-build`; its `i-slint-compiler` capability roots are exactly the empty `default`, `rust`, `display-diagnostics`, and `proc_macro_span` features required by the owned adapter and `slint-macros`, with their dependency-feature closure. Compiler `software-renderer`, `bundle-translations`, default `image` codecs, compile-time texture/font rendering, and any unreviewed downloader are absent. PartMan `.slint` input contains no image/font/translation asset; every host-only package remains subject to licence/advisory/source policy; and any locked feature, codec, or compiler-version drift requires source review and rerunning this gate. The evaluator traverses locked Cargo metadata, manifest features, dependency kinds, and target predicates for each host/target pair; it neither parses `cargo tree` text nor rejects an inactive package-list entry. |
| G-CFG-03 | `.slint` input is compiled AOT by the PartMan-owned pinned-compiler adapter with explicitly configured resources, diagnostics, accessibility, and Fluent style. Its fixture suite proves deterministic generation, nested-import and fixture-only resource dependency tracking, syntax/semantic/warning failure, output containment beneath `OUT_DIR`, and production-source rejection of PartMan image/font/translation/native-style input. The source-derived inventory covers every supported `SLINT_*` build/runtime input; the outer xtask preflight, build guard, and entry-point guard enumerate `vars_os` and reject the entire ambient prefix with Windows' ASCII-case-insensitive name semantics and Unix byte semantics before constructing the compiler or component. Clean builds under the minimal environment, with Qt availability varied, have identical generated-input hashes and remain Qt-free. Hostile build/launch tests cover lowercase, mixed case, Unix non-Unicode, every known name, an unknown future-style name, a cached build-script result, and a PartMan-only rerun nonce; they prove ambient state cannot enable fullscreen, scaling, externalized resources, debug metadata, inlining changes, or live preview. Every Slint re-pin includes an explicit adapter API/source diff and compile proof; no automatic `slint-build` fallback is permitted. |
| G-CFG-04 | A closed enum selects Winit and an exact renderer in code before component creation. Hostile `SLINT_BACKEND` values fail at the G-CFG-03 guard. Every adoption-eligible artifact has exactly one renderer feature, which prevents cross-renderer fallback; a presented frame and accessibility root prove successful initialization while internal same-graph retry may still occur. The combined graph is never eligible, reports its request and the possibility of fallback, and does not treat either as runtime-renderer attestation because Slint exposes no stable public getter. No late FemtoVG-to-software recovery is claimed. |
| G-CFG-05 | The committed Slint token interface is generated from `schemas/design-tokens.json`; regeneration is clean and enumerates every current canonical role, mark, shape, label ID, theme, and contrast pairing without embedding a display label. Rust catalogue tests resolve every label ID. A pinned inventory of every compiler Palette/StyleMetrics reference—not only `apply_default_properties_from_style`—drives generated wrappers plus AST/lowered-IR checks that explicitly bind every affected visual, layout, and window property. Static checks reject raw generated-color access, Palette brushes, StyleMetrics, and unapproved styled widgets; the sole import exception is the generated theme adapter's read of `Palette.color-scheme`. Separate evidence supplies high-contrast state. |
| G-CFG-06 | Raw identifiers remain authoritative Rust values. The full collision-safe escape representation round-trips the hostile UTF-8/WTF-16, control, bidi, literal-backslash, and escape-lookalike corpus. The separately bounded visual representation truncates only on original-grapheme or whole-escape-token boundaries, marks truncation unambiguously, and preserves stable-ID selection. |
| G-CFG-07 | Static and runtime probes show an offline, synthetic-only application with no storage discovery or command bridge, no helper/elevation path, no telemetry/network access, no interpreter/control server, and no successful-operation simulation. The resolved `webbrowser` helper and every URL-launch API have no PartMan import/call site; runtime tracing confirms no external process or socket action. Required local accessibility IPC is inventoried separately and is not relabelled network access. |
| G-CFG-08 | Every new Rust workspace member inherits workspace lints, contains crate-level documentation, has no `unsafe`, and passes the existing offline, supply-chain, and release-profile policy checks. |

### Platform floors

Runtime evidence means launching an interactive release-shaped binary and
exercising rendering, keyboard input, accessibility exposure, scaling, and
clean shutdown. A cross-compile or package build alone is insufficient. Every
proposed shipping renderer uses its own single-renderer artifact. FemtoVG,
software, and the non-shipping combined control are reported separately; no
platform row may be satisfied by an unobserved or silent renderer fallback.

| ID | Platform | Required evidence |
| --- | --- | --- |
| G-PF-01 | Windows 11 23H2+ x86-64 | Single-renderer Winit runs for every candidate; UI Automation; mixed-DPI movement; clean `asInvoker` launch; no silent fallback |
| G-PF-02 | Windows 10 22H2 build 19045 x86-64 | Same core runtime and accessibility checks; no use of a Windows 11-only API |
| G-PF-03 | Windows 10 Enterprise LTSC 2021 x86-64 | Same core runtime and accessibility checks on the separately required LTSC floor |
| G-PF-04 | macOS 13 Ventura, Apple Silicon | Native runtime, VoiceOver, bundle launch, text/path rendering, scaling |
| G-PF-05 | macOS 13 Ventura, Intel | Native x86-64 runtime, VoiceOver, bundle launch; this closes the largest upstream-matrix gap |
| G-PF-06 | Debian 12 and Ubuntu 22.04 | One x86-64 candidate package is built on the Ubuntu 22.04 floor, imports no symbol newer than `GLIBC_2.35`, and is hash-identical in unchanged runtime tests on both distributions under X11 and Wayland; D-Bus and AT-SPI/Orca; declared and dynamically loaded Winit dependencies; this closes the gap below upstream's recent-LTS envelope |
| G-PF-07 | Arch Linux current | Native package build; X11 and Wayland smoke; tool and library versions recorded |
| G-PF-08 | All required Windows x86-64 floors | Debug and release binaries each survive 100 clean-process launches of the 1,000-node scene and its interaction loop without `STATUS_STACK_OVERFLOW`; PE stack-reserve values are recorded. If the default reserve is insufficient, only the desktop package's `build.rs` may emit `cargo::rustc-link-arg-bin=partman-desktop=/STACK:8388608`; workspace `rustflags` remain forbidden. |
| C-PF-01 | Windows 11 aarch64 | Extended release-shaped Winit runtime, renderer, UI Automation, scaling, clean-shutdown, artifact-hash, and host-version evidence equivalent to G-PF-01 because Slint tests this architecture |

`C-PF-01` is comparative only; this ADR does not quietly add an architecture
promise absent from the current PartMan platform table.

### Accessibility and interaction

| ID | Objective assertion and required evidence |
| --- | --- |
| G-AX-01 | The header, device rail, topology, inspector, and plan drawer expose reviewed landmark/group semantics and stable IDs in Windows UI Automation, macOS AX, and Linux AT-SPI tree dumps. Assertions name every required node and property rather than accepting any non-empty tree. |
| G-AX-02 | Device and topology selection expose selectable/selected state, position, count, accessible name, IEC size, and exact bytes. The visual topology uses supported list/group semantics until Slint provides and PartMan qualifies a real TreeView. |
| G-AX-03 | A keyboard-only test covers selection, drawer expansion/collapse, focus restoration, reading order, disabled state, and in-window confirmation. Focus is visibly rendered, never obscured, and target sizes meet applicable WCAG 2.2 AA rules or carry a reviewed criterion exception. No core flow depends on an upstream modal or general-purpose popup. |
| G-AX-04 | NVDA and Narrator on Windows, VoiceOver on both required Mac architectures, and Orca on Linux can traverse and operate every synthetic-shell control without pointer input. Manual transcripts are paired with platform accessibility-tree captures. |
| G-AX-05 | Application zoom at 100%, 125%, 150%, and 200%, narrow-window reflow, and WCAG 1.4.12 text-spacing conditions neither clip nor hide information or controls. Because Slint lacks word-spacing and line-height properties, evidence must demonstrate a custom layout/string strategy that preserves selectable/copyable text and accessibility semantics; otherwise this gate fails. OS scaling and mixed-DPI movement are tested separately. |
| G-AX-06 | Automated rendered-state contrast covers normal, focus, selected, disabled, warning, and error states in system, PartMan light/dark, and high-contrast themes. Color is never the sole signal; reduced motion and color-blind-safe cues are verified. |
| G-AX-07 | IME input, bidi text, bidi controls, combining marks, emoji, CJK, non-Western labels, and the lossless escaped-path corpus render and remain operable under every proposed shipping renderer. |
| G-AX-08 | A generated gallery enumerates every current canonical shape and semantic mark under every proposed shipping renderer. Transforms, shadows, text outlines, clipped rounded corners, and layer opacity are absent unless pixel and accessibility evidence proves equivalent behavior. |
| G-AX-09 | The synthetic presentation exercises every PLAN-004 risk label and every UI-011 progress state, including focus behavior and status/live-region announcements. It does not claim real planning, authorization, execution, verification, or completion. |
| G-AX-10 | Each platform adapter's asserted properties are compared with the platform tree; unsupported or missing schema properties are recorded as failure or inconclusive, never inferred from AccessKit or Slint API availability alone. |

These checks establish only the synthetic shell. ACC-011 remains blocked on the
real plan, confirmation, authorization, progress, and completion flow.

### Packaging, security, and supply chain

All installer probes run in disposable, revertible VMs. Feasibility-only
package definitions and scripts live under
`apps/desktop/packaging-feasibility/**`; they do not occupy a future production
packaging directory.

| ID | Objective assertion and required evidence |
| --- | --- |
| G-PKG-01 | On clean Windows 10 and 11 VMs, the GUI-subsystem application carries `asInvoker`, version, and icon resources; installs and uninstalls without elevation; performs no block-device discovery or mutation; and writes only its explicit evaluation package prefix plus documented installer metadata. A VM and remote session without working OpenGL must still launch the fully functional accessible shell. Any software-renderer artifact or pre-window process selection used to satisfy this is explicit, independently identified, included in package/footprint evidence, and passes all shipping-renderer gates; clean refusal or silent in-process fallback does not satisfy the platform contract. |
| G-PKG-02 | Separate x86-64 and aarch64 macOS 13 bundles launch, and the assembled universal bundle contains both slices. `LSMinimumSystemVersion` and every Mach-O slice's minimum deployment target are 13.0; `codesign --verify --deep --strict` passes for the evaluation signature. Production signing/notarization remains WP-P120. |
| G-PKG-03 | Debian metadata and the candidate x86-64 artifact come from the Ubuntu 22.04 glibc 2.35 floor; the exact artifact hash runs unchanged on Ubuntu 22.04 and Debian 12, and ELF inspection rejects any import newer than `GLIBC_2.35`. The Arch `PKGBUILD` is native. Package metadata and runtime tracing enumerate linked and dynamically loaded X11, Wayland, OpenGL, font, D-Bus, and AT-SPI libraries. Minimal clean X11 and Wayland VMs, including a no-working-OpenGL condition, launch the fully functional accessible shell and pass an Orca smoke. Production package signing remains its packaging work package. |
| G-PKG-04 | Every installer VM starts from and returns to a named snapshot. OS-native causal audit tracing records the application, installer, package-manager service, process/service identity, transaction ID, and every attributed filesystem/registry/configuration write. Three same-duration no-install runs from the same snapshot define ambient actor/path classes: an unattributed event may be classified ambient only when the identical normalized class appears in all three controls and no candidate transaction reaches it; candidate-attributed events are never subtracted, and any other unclassified event makes the gate inconclusive. Attributed writes must match a platform-specific predeclared allow-list containing the evaluation prefix and exact installer metadata. Separately captured cryptographic pre/post inventories of partition tables, boot configuration, and named synthetic block devices must be identical; any change there fails regardless of actor or ambient controls. |
| G-INT-01 | Primary performance thresholds use feature-equivalent, dialog-free Tauri and Slint shells. Separate immutable Tauri and Slint dialog-control commits then implement the same bounded native file-open/save behavior. The dialog may browse, but the application performs no read or write until OS-appropriate held-handle/object-identity checks prove containment beneath the test-owned temporary root. Open accepts only manifested regular synthetic files with the expected object identity/hash and link count; save creates a new file relative to a verified parent handle without following links or overwriting. Symlinks, hard links, junctions/reparse points, root/parent renames, cancellation, and path-race escapes are tested on every required OS. Evidence records each stack's accessibility, dependency, license, portal/runtime, package, and footprint delta and also compares the two equivalent-feature totals; Slint alone cannot carry a dialog cost absent from its baseline. |
| G-SC-01 | `cargo xtask supply-chain` passes on Windows, macOS, and Linux. Every Slint license allowance is exact-package and exact-version scoped; GPL is not added to the global allow-list. |
| G-SC-02 | Lockfiles and resolver-3 host/target feature graphs match G-CFG-02. No build script downloads an unpinned binary; the SBOM/license inventory includes the pinned internal compiler, owned AOT adapter, unavoidable runtime `image` JPEG/PNG path and complete `resvg` closure, and renderer- and platform-specific dependencies. `slint-build`, compiler `software-renderer`/`bundle-translations`, the full `image/default` codec uplift, Skia, and `skia-bindings` are absent. Artifact inspection and clean offline rebuilds agree with that graph. |
| G-LIC-01 | Before any linked candidate binary is uploaded, artifact inspection finds all applicable packaged licenses/notices and a captured, hashed readback of the public PartMan download page proves that Slint's official attribution badge is easy to find, has useful accessible text, and targets the required Slint page. Publication ordering prevents a binary from becoming downloadable before this check passes. |

Slint does not satisfy PKG-001 through PKG-004 by producing an executable. The
evaluation records packaging feasibility and residual work; it does not mark
the production packaging requirements delivered.

### Comparative footprint and responsiveness

Measurements are paired Tauri-versus-Slint trials on every required platform in
G-PF-01 through G-PF-07, not a convenient subset. Each pair uses the same
hardware or cloned VM image, power policy, display and scaling, window size,
source-equivalent synthetic data, release profile, and content-addressed
harness. The report records source commits, toolchains, lockfile hashes, OS
image, WebView/runtime and shared-library versions, renderer feature graph and
programmatic request, process-tree enumeration method, run order, normalized
raw samples, and artifact hashes. Only a single-renderer graph may name the
initialized renderer by construction; a combined control records its request
and presented-frame result without inventing an actual-renderer label. A
deterministic predeclared schedule alternates or randomizes Tauri/Slint order
within each pair. No statistical outlier is removed. A candidate crash or
timeout fails the relevant gate. A harness-proven host interruption may
invalidate only the whole pair; the manifest retains the reason and values and
the pair is rerun before analysis.

A cold sample restores the named clean VM snapshot, boots it, and performs the
first application launch before any prior application run. Collect at least 100
independent cold pairs. After one unmeasured priming run, collect at least 300
warm pairs and 300 pairs for each interaction scenario. First-window latency
ends at the later of first presented frame and a platform accessibility tree
containing the required root landmarks. Interaction latency begins at input
dispatch and ends at the later of the updated presented frame and updated
accessibility selection/status.

For each pair and metric, the primary observation is the Slint value divided by
the Tauri value. The generator uses nearest-rank p95/p99, reports the median,
median absolute deviation, range, and every paired value. Median and p95 receive
a one-sided 95% upper confidence limit from 10,000 paired percentile-bootstrap
resamples. Resampling uses the versioned SHA-256 counter seed derived from the
gate ID, platform ID, and raw-evidence-manifest hash, so it is deterministic
without an ambient random seed. That hash is SHA-256 over the existing PartMan
canonical encoding of a versioned, integer-unit, sorted raw-evidence manifest.
The hash preimage includes source/artifact hashes and raw observations but
excludes its own digest, seeds, generated statistics/results, and rendered
reports. Counter expansion is
`SHA-256(seed || big_endian_u64(counter))`; rejection sampling maps digest words
to indices without modulo bias. Cold results report median and p95. Warm and
interaction results also report p99 and its exact one-sided 95%
distribution-free upper tolerance bound from the binomial/order-statistic rule;
at the minimum 300
pairs this is the sample maximum because `1 - 0.99^300` exceeds 95%. The
generator never substitutes a percentile bootstrap for that unseen-tail bound.
A point estimate beyond a threshold fails. A point estimate inside the
threshold whose applicable upper limit crosses it is inconclusive rather than
pass. For paired launch, interaction, and memory metrics, the independent
launch pair—not multiple samples from one process—is always the resampling unit.

Measure stripped executable and package bytes and clean-system incremental
runtime dependency bytes. For memory, identify the root application process and
all descendants by PID, parent/create time, and platform process group, freeze
that membership at each simultaneous sample, and retain the raw per-process
rows. The primary cross-stack metric is normalized physical footprint at 30 and
60 seconds:

- Windows sums a proportional working-set estimate from `QueryWorkingSetEx`,
  dividing shared resident pages by their observed share count, and separately
  reports `PrivateUsage`/private commit;
- Linux sums PSS from every process's `/proc/<pid>/smaps_rollup` and separately
  reports `Private_Clean` plus `Private_Dirty`; and
- macOS sums process-group `phys_footprint` from the documented VM-ledger or
  `footprint` interface and separately reports private resident bytes.

Ordinary RSS/working-set totals, process count, and CPU remain diagnostics; they
do not decide G-PERF-03 because summing shared WebView pages would bias a
multi-process stack. Collect at least 100 independent paired launches for every
normal/1,000-node screen and renderer condition; the 30- and 60-second readings
from one launch remain one paired observation, not two bootstrap samples.
Measure selection and continuous resize at 100 and 1,000 nodes and at 100% and
200% zoom. After warm-up, run three independent soaks per proposed shipping
renderer, each with at least 10,000 repeated selection/update/resize cycles over
at least 30 minutes. Each run executes a fixed five-minute workload warm-up,
idles for two minutes, and defines its baseline as the median of the final 60
one-second retained-private-byte samples. It then runs the timed workload while
sampling once per second, stops the workload, idles for five minutes, and
defines recovery as the median of the final 60 samples. Use the versioned
Theil-Sen estimator separately for each timed soak, normalized to that run's
baseline. Do not bootstrap autocorrelated observations from one process as if
they were independent; report all three slopes and recovery medians.

| ID | Eligibility assertion |
| --- | --- |
| G-PERF-01 | Every required platform has the paired metadata, sample counts, raw normalized samples, statistics, artifact hashes, and replayable harness specified above; measurement variance or missing endpoints make the gate inconclusive. |
| G-PERF-02 | On every required platform, Slint's application-controlled packaged bytes are no greater than Tauri's. On both required Linux baselines, Slint's clean-system incremental runtime dependency bytes are strictly lower. |
| G-PERF-03 | At both 30 and 60 seconds, for the normal and 1,000-node screens on every required platform, the upper bound of the bootstrapped 95% confidence interval for the Slint/Tauri normalized-physical-footprint ratio is at most 0.70 using the OS-specific metric and 100 independent launch pairs above. |
| G-PERF-04 | On every required platform, the one-sided 95% upper bound for the Slint/Tauri cold first-window median and p95 ratios, and for the warm first-window, selection, and resize median and p95 ratios, is at most 1.10 using the paired bootstrap above. The exact binomial/order-statistic 95% upper tolerance bound for each 300-pair warm/interaction p99 ratio is also at most 1.10. If variance or the observed tail makes either conclusion impossible, the result is inconclusive. |
| G-PERF-05 | Each of the three 10,000-cycle soaks completes without crash; its normalized retained-private-bytes Theil-Sen slope is at most 1% of the fixed baseline per hour; and its final 60-sample recovery median is within 5% of the fixed 60-sample pre-soak baseline median. All fixed-interval observations and each independent slope are retained; no time sample is relabelled an independent launch and no finite run is claimed to prove absence of all future growth. |
| C-PERF-01 | FemtoVG-only, software-only, and combined-control builds receive the same measurements. A combined-control threshold miss fails no gate by itself; any renderer or renderer-selection path proposed for shipping must use a single-renderer artifact and satisfy all applicable `G-*` gates. |

A result outside these bounds may still inform a later decision, but it does
not pass this ADR. Accepting a tradeoff needs a new explicit architecture
decision.

The provisional Windows Tauri baseline from 2026-07-31 is recorded to prevent
the comparison target from drifting silently:

- immutable comparison commit
  `b0f11249903372d9b9cfba76128479ecfd3917f3` on the preserved, unmerged
  `codex/wp-030-desktop-shell-inc2-v2` branch, based on `04cb843`;
- the pre-commit 71-path patch fingerprint
  `dbbd344cf8ae3e5e0d42a0f433ec6f3394d1ea50`, produced by
  `git diff --cached --binary | git hash-object --stdin`;
- Rust 1.96.0 (`ac68faa20`), Cargo 1.96.0 (`30a34c682`), Node 24.18.0,
  npm 11.16.0, Cargo.lock SHA-256
  `8B744D2699641A55B761C1E9907E7D48A6C7F44AB96C0CCF85788185A43306CE`,
  and package-lock SHA-256
  `A8C96CEC6937674ED87D3D59AF7D691B9C0ABAD3B57B7E69E107F795F45C71C6`;
- `cargo xtask desktop` passed at that commit: token drift, lint, color policy,
  type checking, all 31 desktop tests, web build, and native release build;
- the originally measured executable was 7,745,536 bytes with SHA-256
  `7A4EA3AC08BD26EBC61418109F92FF1D33186E7C673D7ACBB3E94C3AA138C013`;
  a post-commit rebuild had the same byte count but SHA-256
  `BA843A5FC9329A7A636C8783F6A4402941B66CFB5FDA827AEEF104D54B814E00`,
  so byte-reproducibility is not assumed and each measured artifact must be
  retained and hashed;
- the original development probe reached a window handle in 197.9 ms cold and
  39.1 ms/42.1 ms on two immediate warm launches; after two seconds the three
  samples had seven processes and working sets of 359.00, 339.58, and
  341.33 MiB.

Those timings are a small development-host probe, not final comparative
evidence. The report must reproduce both candidates with the common
content-addressed harness, required sample protocol, and 30/60-second windows;
the immutable source commit and artifact hashes replace any mutable-index
comparison.

## Consequences

Positive:

- The native option is tested against PartMan's actual platform and safety
  contract instead of selected by taste.
- The strong work in the Tauri shell—tokens, strings, exact-value formatting,
  interaction behavior, and accessibility expectations—remains the acceptance
  baseline without retaining its web runtime permanently.
- A single generated token boundary prevents Slint from growing an independent
  palette.
- A bounded owned AOT adapter avoids `slint-build`'s unused build-time renderer,
  translation bundler, and default image-codec graph without forking Slint.
- Renderer, license, debug-feature, and OS-floor risks are explicit before
  dependencies enter the production graph.

Negative and accepted:

- The evaluation costs one temporary implementation and real testing on old
  operating systems and assistive technologies not available in ordinary
  hosted CI.
- Slint's custom license adds an attribution obligation and an exact
  cargo-deny policy decision even if the technical evaluation succeeds.
- The AOT adapter calls an exact-version internal compiler API. Each Slint
  upgrade therefore requires a deliberate API/source diff, fixture proof, and
  full gate rerun instead of relying on semver compatibility.
- The lightweight software renderer is unlikely to be a complete fallback for
  arbitrary user text without upstream work or a different renderer.
- Slint provides no packaging/updater shortcut; PartMan still owns that work.
- Until the result is known, the existing Tauri branch and the Slint branch
  both require preservation as evidence, though only one may become production.

Migration: none yet. This ADR changes no public interface, schema, canonical
encoding, platform promise, or production stack. A successful result requires
a new adoption ADR, a major specification bump, updated work-package
governance, and removal of obsolete Tauri-specific policy in the implementation
that eventually lands.

## Verification

The feasibility pull request must provide a generated, source-backed
`docs/quality/slint-feasibility.md` with exactly one row per `G-*` and `C-*`
identifier above, carrying the exact source commit, raw-evidence-manifest hash, host
image or hardware, command, artifact hash, result, and limitation. The generator
fails on a missing, duplicate, or unknown gate; verifies every referenced file
and artifact hash; derives automatable outcomes from the ADR's thresholds rather
than trusting result fields; validates structured manual attestations; and
produces byte-identical Markdown from committed normalized evidence under
`docs/quality/slint-feasibility-data/**`. The common generator, measurement
code, replay harness, and their content-addressed manifest live under
`tools/slint-feasibility/**`. Raw system diagnostic logs, secrets, user/device
identifiers, and signing material are not committed; normalized evidence carries
only the reviewed fields needed to reproduce the report. Screenshots and manual
observations supplement but never replace an automatable assertion.

Repository gates:

```text
cargo xtask ci
cargo xtask test --tier 1
cargo xtask desktop
cargo xtask cross-language
cargo xtask supply-chain
cargo xtask traceability
cargo xtask verify-change-ownership --base origin/main
```

`cargo xtask desktop` must compile `.slint` sources, verify generated-token
drift, run the owned AOT adapter fixtures plus Rust view-model and interaction
tests, separately inspect resolver-3 host/build and target/runtime Slint
features, source-derived environment inventory, linked artifact contents, and
style, test hostile Slint environment overrides, build every adoption-eligible
single-renderer variant, validate the complete gate inventory/report inputs,
and produce the
non-privileged native application. Mandatory all-features linting compiles the
marked combined graph for code quality, but neither `cargo xtask desktop` nor
Tier 1 turns its comparative runtime or thresholds into a hard gate. A separate
`cargo xtask slint-controls` invocation records that outcome as `C-PERF-01`; a
well-formed `fail` or `inconclusive` control record is valid, while a broken
harness or fabricated/missing gate inventory is not. Platform qualification and
assistive-technology checks that cannot run in ordinary CI remain separately
recorded gates, never silently skipped.

## Revisit conditions

- Slint changes the royalty-free terms, attribution mechanism, feature names,
  backend defaults, security support policy, or selected renderer behavior.
- Any newer stable Slint release exists before the final decision. Because
  upstream's public security policy supports its latest release, re-pin through
  reviewed dependency evidence and rerun every gate; do not float the
  evaluation. Later changes to the macOS matrix, non-Western software-renderer
  support, modal windows, popups, TreeView, or accessibility also trigger a
  fresh review.
- The macOS 13 Intel or Apple Silicon runtime cannot be obtained for testing.
  The result remains inconclusive rather than lowering PartMan's floor.
- Any shipping renderer corrupts user text, loses accessibility state, or
  requires a debug/control feature.
- Slint fails the comparative thresholds or introduces a supply-chain exception
  less acceptable than the Tauri boundary it is intended to replace.
- PartMan's production platform or accessibility contract changes. Re-evaluate
  the gates against the new contract rather than inheriting this result.
