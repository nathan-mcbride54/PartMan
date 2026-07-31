# PartMan desktop shell

This package is the WP-030 read-only Tauri 2 foundation. It renders a synthetic
device rail, topology map, inspector, and illustrative pending-plan drawer. It
does not discover devices, call storage APIs, create plans, expose Apply, invoke
a native command, or request elevation.

## Local development

Use the repository-pinned Rust toolchain and Node 24.18.0 or newer. On Debian or
Ubuntu, Tauri's native build also needs:

```text
libwebkit2gtk-4.1-dev
libxdo-dev
libssl-dev
libayatana-appindicator3-dev
librsvg2-dev
```

From this directory:

```text
npm ci
npm run dev
```

The Vite view is suitable for responsive presentation work. To open the native
development webview instead, run:

```text
npm run tauri -- dev
```

Neither mode gains storage or native-command permissions.

## Required gate

From the repository root:

```text
cargo xtask desktop
```

The command reinstalls the exact npm lock, checks generated design-token drift,
lint, typed colour policy, TypeScript, and tests, builds the Vite assets, and
finishes with a native `tauri build --no-bundle` release build whose Cargo
runner receives `--locked`.

The shared UI consumes only generated typed accessors from
`schemas/design-tokens.json`. Run `npm run tokens:write` only when deliberately
changing the canonical token source; generated output is committed and reviewed.

Accessibility evidence and explicit remaining limits are in
`docs/quality/accessibility.md`.
