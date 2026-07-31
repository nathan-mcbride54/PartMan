# Desktop icon provenance

`../app-icon.png` is the committed lossless source for this icon set. It was
created with OpenAI ImageGen in **generate** mode for WP-030 increment 2. The
prompt requested an original, text-free desktop application icon combining a
disk platter and partition arcs with a protective shield, using a restrained
blue/cyan-on-charcoal palette and a square composition that remains legible at
small sizes.

The platform assets in this directory were produced from that source from
`apps/desktop/` with:

```text
npm exec tauri icon src-tauri/app-icon.png
```

Only desktop outputs are committed (`.png`, `.ico`, `.icns`, and Windows tile
sizes). The command also offered Android and iOS trees; those were intentionally
omitted because this package is a Windows, macOS, and Linux desktop shell.

Regenerate the complete set after changing the source rather than editing an
individual platform size. These files are application artwork, not signing
material.
