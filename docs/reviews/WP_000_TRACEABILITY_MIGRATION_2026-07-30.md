# WP-000 traceability migration ledger — 2026-07-30

This is the evidence-preservation proof for PR #65. It compares the 31 evidence
rows in the hand-written `docs/traceability/WP-000.md` on `origin/main` at
`b2800a57d59a20337e039a8257b7d04d56791747` with the generated document on the
PR branch.

The old “33-row” count included the Markdown table header and separator. The
first generator produced 13 evidence rows, so the unexplained evidence-row
difference was 18. The current generator produces more rows because one source
relationship carrying several requirements renders once for each requirement.
Equal row counts are therefore neither necessary nor sufficient.

The acceptance rule is:

> Every old evidence relationship has a generated equivalent, an intentional
> consolidation or expansion, a named narrative destination, or a documented
> supersession. No row may disappear without one of those dispositions.

## Result

- Old hand-written evidence rows: **31**
- Current generated evidence rows: **46**
- Behavioural annotations: **15**
- Structured evidence declarations: **26**
- Old rows with no disposition: **0**
- Old rows still unsupported by the generator: **0**

The generated count is informational. The last two counts are the merge gate.

## Row-by-row disposition

| Old row | Previous evidence | Disposition | Generated or narrative destination |
| ---: | --- | --- | --- |
| 1 | `rust-toolchain.toml`, `Cargo.lock` | Generated equivalent | SEC-010 structured row also binds `cargo xtask verify-toolchain` |
| 2 | `verify_toolchain` implementation | Intentionally consolidated | Same SEC-010 toolchain row uses the owned xtask source through its validated command surface |
| 3 | `deny.toml`, `cargo xtask supply-chain` | Generated equivalent | SEC-010 structured advisory/licence/source-policy row |
| 4 | `.github/workflows/ci.yml` | Expanded | Generated under both Section 13's three-platform milestone contract and SEC-010's immutable dependency contract |
| 5 | Action-pin happy path and missing/empty-root tests | Expanded | Existing SEC-010 annotations plus the structured real-workflow and discovery-root row |
| 6 | `.gitattributes` | Generated equivalent | Section 13 structured row with `cargo xtask fmt-check` |
| 7 | Task-parser acceptance/refusal tests | Generated equivalent | Section 13 structured single-entry-point row |
| 8 | Tier-parser tests and unavailable-tier refusal | Expanded | SAFE-007 structured parser row plus the existing SAFE-005/SAFE-007 refusal annotation |
| 9 | `docs/adr/0000-template.md` | Generated equivalent | Section 12 structured ADR row |
| 10 | `.github/CODEOWNERS` | Generated equivalent | Section 1.10 structured review-ownership row |
| 11 | Licence texts, manifest declarations, and `deny.toml` policy | Expanded | SEC-005 structured licence-artifact row plus semantic manifest annotations |
| 12 | Locked xtask alias and its regression | Generated equivalent | SEC-010 structured `.cargo/config.toml` and alias-test row |
| 13 | Structural YAML discovery regressions | Expanded | SEC-010 annotation for the three original bypasses plus structured spelling and malformed-input tests. The stale tests that the old row explicitly said were removed remain excluded rather than being resurrected by metadata |
| 14 | Container and Dockerfile dependency regressions | Expanded | Two SEC-010 annotations. The complete bypass roster remains executable inside `a_dockerfile_action_is_followed_to_its_base_images` rather than being copied into output prose |
| 15 | Tree-wide npm advisory discovery | Generated equivalent | Existing SEC-010 annotation |
| 16 | Workspace lint inheritance | Generated equivalent | Existing SAFE-009 annotation |
| 17 | Per-occurrence release comments | Generated equivalent | SEC-010 structured row for both regressions |
| 18 | Contained, terminating local-action recursion | Generated equivalent | SEC-010 structured row for both regressions |
| 19 | Fuzz lock, preflight, audit graph, and Dependabot coverage | Corrected and generated | SEC-010 structured row cites WP-000-owned xtask/dependabot evidence and validated commands. The old row's direct `fuzz/Cargo.lock` citation is not copied because that path belongs to WP-010; WP-000 proves the policy that consumes it without claiming another package's artifact |
| 20 | Semantic licence verification command and mutations | Expanded | Existing SEC-005 annotation plus structured command/removal mutation row |
| 21 | Inventory ownership command and regressions | Generated equivalent | Section 1.10 structured row |
| 22 | Change ownership command and base-revision regressions | Generated equivalent | Section 1.10 structured row |
| 23 | Derived lockfile rules | Generated equivalent | Section 1.10 structured row |
| 24 | Per-commit trailers and merge exemption | Generated equivalent | Section 1.10 structured row |
| 25 | Rename endpoints and raw path handling | Generated equivalent | Section 1.10 structured row |
| 26 | Traceability command and spec-owned vocabulary | Expanded | Section 11.7 command row plus annotation proving IDs, headings, and actual Section 1 contract items come from the specification |
| 27 | Generator failure mutations and passing control | Expanded | Section 12 structured row plus the new structured-evidence mutation annotations |
| 28 | Orphaned annotation and measured helper mutation | Expanded | Section 11.7 structured row for orphan, slid-annotation, and pre-render routing refusals |
| 29 | Positional binding's undetectable-rename limitation | Superseded, narrative retained | The limitation was closed by the explicit `// Evidence:` name. The measured failure of the first version and why redundancy is load-bearing remain in `docs/work-packages/WP-000.md`; the generated table carries the replacement regressions rather than presenting an old limitation as a current guarantee |
| 30 | Cross-platform handling of `cfg`-gated tests | Generated equivalent | Section 11.7 structured row |
| 31 | Promotion of a matching reserved path into ownership | Generated equivalent | Section 1.10 structured row |

## Independent checks

The migration does not trust this ledger to validate itself:

- Requirement IDs come from their definition sites in `AGENT_BUILD_SPEC.md`.
- Section references come from numeric headings, except `Section 1.N`, which is
  accepted only when that exact numbered operating-contract item exists.
- Tests come from `cargo test --workspace --all-targets --locked -- --list`.
- Paths must be tracked, normalized, and owned by the declaring package.
- Commands must be accepted by xtask's real parser.
- The generated file is checked by byte equality in `cargo xtask ci`.
- Hand-editing the generated output, inventing `Section 1.99`, naming a missing
  test or path, using another package's path, supplying an invalid command,
  duplicating a source block, or leaving it unclosed is refused by an automated
  regression.

This ledger proves migration completeness, not semantic truth. Whether each
piece of evidence actually establishes the requirement remains a review
obligation, stated in the generated document itself.
