# Response to the 2026-07-29 project audit

This answers `PROJECT_AUDIT_2026-07-29.md` finding by finding, for the next
reviewer. Every reproduction the audit described was re-run before being acted
on, and every fix was re-tested against the audit's own reproduction. Two pull
requests carry the work: WP-030 evidence remediation (#38, merged), and the
foundation remediation this document lands with.

## Disposition of findings

| Audit finding | Disposition |
| --- | --- |
| **High** — lockfile not enforced at the gate boundary | **Fixed.** `--locked` is in the `xtask` alias itself; the audit's deleted-entry mutation now refuses with "cannot update the lock file" instead of silently regenerating. A Tier-1 test (`the_xtask_alias_enforces_the_lockfile_at_the_gate_boundary`) fails by name if the alias loses the flag. |
| **High** — accessibility input lowers its own standard | **Fixed in #38.** Floors and vocabulary moved to `crates/tokens/src/policy.rs`; the file may restate but must agree. The audit's exact reproduction (threshold 3.0 + dimmed colour) now yields 3 findings where it yielded none. Twelve policy mutations added. |
| **High** — semantic roster deletable | **Fixed in #38.** UI-003/PLAN-004/UI-011 vocabulary is exact in both directions; the audit's `entity.container` deletion now yields 2 findings. `deny_unknown_fields` everywhere; versions validated. |
| **High** — quoted YAML `uses` key bypasses the pin gate | **Fixed.** Quoted keys are recognized and checked; everything else `uses`-shaped outside the enforced subset (flow mappings, block scalars, aliases, anchors, escaped keys, explicit keys, next-line values) is a named violation rather than a silence. Composite actions under `.github/actions/` are scanned. The audit's reproduction now fails: `ci.yml:34: actions/checkout@v7 — not pinned to a full commit SHA`. |
| **High** — fuzz crate unlocked and ungated | **Fixed.** `fuzz/Cargo.lock` committed and un-ignored; `cargo xtask fuzz` verifies it with `--locked` before the nightly toolchain is involved; `cargo xtask supply-chain` checks the fuzz graph under the same `deny.toml`; `/fuzz` Dependabot entry added. NCSA joined the allow-list because `libfuzzer-sys` is `(MIT OR Apache-2.0) AND NCSA` — the AND makes it mandatory — with the rationale commented in `deny.toml`. |
| **High, known** — WP-020 authorization proves a pathname | **Accepted as recorded.** No destructive consumer is being started; WP-020's own increment-2 preconditions (handle lifetime, independent token, Windows link identity) stand. Nothing in this remediation touches it. |
| **Medium** — hosted runner labels vs SEC-010's builder-image rule | **Documented as a deviation**, not fixed: GitHub offers no digest-addressed hosted images. `docs/quality/dependency-policy.md` now carries the deviation, its residual risk, and the revisit condition (release builds under ADR-S1). The audit offered a spec issue or a documented deviation; the deviation was chosen because this is an unsatisfiable requirement on hosted infrastructure, not a conflict between two spec clauses. |
| **Medium** — ownership neither enforceable nor sufficient | **Partially addressed.** WP-030's assignment now owns its status rows (amended in #38, recorded as an amendment). Machine-readable owned paths and generated traceability are filed as a tracked issue rather than silently deferred — see below. Mechanical enforcement remains open, as WP-000's traceability has always said. |
| **Medium** — WP-000 reported complete despite failing the definition of done | **Fixed.** README reclassifies WP-000 as in progress, naming what is delivered and what Section 12 still requires. |
| **Medium** — documentation drift (six items) | **Fixed.** Partial-row count (#38); test-tiers token claim (#38); test-tiers Tier-1 contents (#38); WP-030 overstatements (#38); action-pin completeness claims and the tag-comment review obligation (this PR); fuzz-lock absence now described as covered because it is covered (this PR). |
| **Medium, recorded** — traceability and two licence declarations ungated | **Licence half closed** by `cargo xtask verify-licenses`, which walks every manifest and runs inside `cargo xtask ci`. Generated traceability is the tracked issue below. |

## What was deliberately not done

- **A structural YAML parser was not added.** The audit's first-choice
  correction was to parse workflows with a strict YAML parser. That means
  adding a YAML dependency to the tool that gates dependencies, which is a
  supply-chain decision deserving its own review, not a rider on a remediation
  PR. The implemented alternative satisfies the audit's underlying demand —
  "fail on unsupported structures rather than interpreting them leniently" — by
  refusing, with a named reason, every `uses`-shaped construct outside a small
  enforced subset. The trade is recorded in `docs/quality/dependency-policy.md`;
  if the subset ever pinches, the parser decision reopens with this paragraph
  as its context.
- **No networked verification that a tag comment resolves to its SHA.** Gate
  commands run offline. It is now recorded as a review obligation on every
  action bump rather than implied to be automated.
- **WP-020 increment 2 was not started**, in line with both the audit and the
  handoff: object-lifetime binding and an independent-factor decision come
  first.

## Verification

| Reproduction | Before | After |
| --- | --- | --- |
| Delete `partman-tokens` from `Cargo.lock`, run the gate | Entry silently regenerated; 160 tests passed | Refuses: "cannot update the lock file … because --locked was passed" |
| Lower token-file text floor to 3.0, dim `text.secondary` | Whole gate green at 3.33:1 | 3 findings, gate fails |
| Delete `entity.container` everywhere | Green, 228 checks | 2 findings, gate fails |
| `"uses": actions/checkout@v7` in ci.yml | Success, one fewer reference | Violation naming the reference |
| Fuzz lock stale vs manifest | Silently re-resolved | `cargo xtask fuzz` refuses before fuzzing |

Local gates on the final tree: `cargo xtask ci` (verify-actions 7 references,
verify-licenses 9 manifests, 311 token checks, 172 tests), `cargo xtask
supply-chain` (both graphs: advisories, bans, licences, sources ok).

One observation from the remediation itself, worth keeping: while testing
`cargo deny` against the *stale* fuzz lock, cargo-deny silently repaired the
lock as part of building its graph — the same fail-open shape the audit
describes, demonstrated by the policy tool. The committed lock plus the
`--locked` preflight in `cargo xtask fuzz` is what prevents that from ever
mattering again; a policy check that mutates its subject is one more reason the
lock had to be committed.
