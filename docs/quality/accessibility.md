# Accessibility evidence and limits

This document records the accessibility status of WP-030's bounded Slint
candidate at immutable checkpoint
`359e33101b8fe6ad017d51d7c1fc0f9e5c501288`. It does not qualify Slint for
production and does not claim ACC-011. The authoritative mechanical outcomes
are generated in [the ADR-0009 report](slint-feasibility.md); this file explains
what the current automated evidence can and cannot support.

## What is automated

The unprivileged checks establish a useful static and model-level foundation:

- `cargo xtask tokens` validates the canonical visual vocabulary, WCAG contrast
  pairings, redundant non-colour channels, typography/layout/cursor contracts,
  stable externalized label IDs, and byte-deterministic generated Slint ABI.
- A closed Rust catalogue resolves every canonical label ID. Identifier tests
  preserve arbitrary byte and WTF-16 values through collision-safe displays,
  and bounded displays cut only at whole grapheme or escape-token boundaries.
- The renderer-neutral synthetic view model keeps authoritative stable IDs,
  preserves selection across device changes, exposes IEC and exact-byte facts,
  and rejects malformed, forged, duplicate, or stale callbacks.
- AOT fixtures and source/lowered-IR checks reject ungoverned styled widgets,
  compiler-injected visual defaults, direct user-facing string literals, and
  PartMan image/font/translation/native-style inputs.
- Both single-renderer configurations and the non-shipping combined control
  compile with accessibility enabled. Compilation proves neither a presented
  frame nor a platform accessibility tree.

These checks are Tier 1 because they read repository/registry/build data and
use synthetic records only. They do not enumerate storage, contact a helper,
request elevation, or enable a destructive tier.

## ADR-0009 accessibility matrix

Every accessibility gate is currently **inconclusive**. No row is omitted or
rounded up from API availability, compiler success, or a non-empty tree.

| Gate | Evidence still required |
| --- | --- |
| G-AX-01 | Reviewed Windows UI Automation, macOS AX, and Linux AT-SPI trees with every required landmark/group, stable ID, and property asserted. |
| G-AX-02 | Platform selection state, position/count, accessible name, IEC size, and exact-byte readback for device and topology rows. |
| G-AX-03 | Keyboard-only selection, drawer, focus restoration/visibility, reading order, disabled state, in-window confirmation, and target-size evidence. |
| G-AX-04 | Paired tree captures and manual transcripts for NVDA, Narrator, VoiceOver on Intel and Apple Silicon, and Orca. |
| G-AX-05 | Application zoom at 100/125/150/200%, narrow reflow, WCAG text spacing, selectable/copyable text, OS scaling, and mixed-DPI movement. Slint's missing word-spacing and line-height controls are not assumed away. |
| G-AX-06 | Rendered normal/focus/selected/disabled/warning/error contrast in system, PartMan light/dark, and high-contrast modes, plus reduced-motion and non-colour-cue proof. |
| G-AX-07 | IME, bidi/control, combining, emoji, CJK, non-Western, and escaped-path rendering and operation under every proposed renderer. |
| G-AX-08 | A complete generated mark/shape gallery under each renderer, with any transforms, shadows, outlines, clipping, or opacity either absent or proven equivalent. |
| G-AX-09 | Every PLAN-004 risk and UI-011 progress state, including focus and status/live-region announcements, while preserving the no-real-operation boundary. |
| G-AX-10 | Each asserted adapter property compared against the actual platform tree; unsupported properties recorded rather than inferred from Slint or AccessKit APIs. |

No platform tree dump, assistive-technology transcript, rendered-pixel capture,
high-contrast operating-system signal, text-spacing run, mixed-DPI run, or
renderer-qualified interaction transcript is committed. The development host's
Windows registry/build observation is not one of ADR-0009's named clean
platform images and is not substituted for one.

## Decision and future use

The candidate already fails hard supply-chain gates, so collecting the missing
accessibility matrix cannot make this exact candidate eligible. The gaps remain
recorded because they are product requirements, not because more testing is
scheduled for the rejected branch. A future Slint release can be reconsidered
only through a new governed evaluation that replays every gate; an adopted
desktop stack must then perform the platform matrix above before claiming the
M0 accessibility criterion complete. ACC-011 additionally needs the real plan,
authorization, apply, progress, verification, and completion flows, none of
which this synthetic shell implements.
