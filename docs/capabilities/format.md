# The CAP-006 qualification store

- Spec version: 11.1.0
- Requirement IDs: CAP-003, CAP-006, Section 9, Section 16
- Owner: WP-050 (`docs/work-packages/WP-050.md`); evidence rows are
  operator-qualified artifacts this package defines the schema for and
  never invents
- Consumed by: `crates/capability` (the engine whose `supported` status
  is constructible only from a qualifying row here)

This directory is the store CAP-006 requires — "tested capability
fixtures for every advertised platform/file-system combination" — and
the home Section 9 names for per-tool version floors. Two documents live
here, each schema-versioned (MODEL-003), each held by the Tier-1 store
test in `crates/capability/src/store_tests.rs`, which is the CI gate: a
malformed row, an unknown field, or a row claiming qualification without
its evidence fails the build.

## 1. `qualifications.json`

```
{
  "schema": "partman.capability.qualification-store",
  "schema_version": 1,
  "advertised": [ <row>, ... ]
}
```

Each row names one advertised platform/file-system/operation combination
and its qualification state:

| Field | Content |
| --- | --- |
| `platform` | A Section 9 platform label, verbatim from the floors table. |
| `file_system` | A file-system kind tag from `schemas/domain/node-entry-format.md` §3a. |
| `operation` | A CAP-002 operation name in kebab-case (`detect` … `wipe`). |
| `state` | `"unqualified"`, or `"qualified"` with the evidence fields below. |

A `"qualified"` row must carry `evidence`: the fixture identifier, the
tier-2 acceptance or matrix run that produced it, the date, and the
transcript digest — the Section 16 rule that no capability is advertised
stable without matrix fixture and acceptance evidence, held as data. The
store test refuses a `"qualified"` row whose evidence fields are absent
or malformed, so qualification cannot arrive silently; it arrives as a
reviewed diff that also updates the test's expected qualification count.

**The advertised set is empty, and the vacuity is named rather than
rounded up:** nothing is advertised while no apply path exists anywhere
in the product. Advertising a combination is itself a reviewed act that
adds its row here `"unqualified"`; qualifying it is a second reviewed
act that fills the row's evidence. `supported` stays unreachable in the
engine until both have happened for some row — which is CAP-003's own
definition doing the gating.

## 2. `tool-version-floors.json`

```
{
  "schema": "partman.capability.tool-version-floors",
  "schema_version": 1,
  "floors": [ {"tool": <name>, "floor": <version>, "basis": <text>}, ... ]
}
```

Per-tool version floors, as Section 9 directs ("Per-tool version floors
live with the capability fixtures (CAP-006) in `docs/capabilities/`").
The floors list is empty for the same reason the advertised set is: no
storage tool is invoked anywhere in the product yet — WP-035's doctor
probes only the repository's own toolchain — and a floor for a tool
nobody calls would be an assertion nobody can test. A tool's floor
arrives with the first package that invokes it, under review, with its
basis stated.

## 3. What this store is not

- Not a runtime artifact: the shipped engine never reads this
  directory. A consumer that embeds qualification evidence does so at
  its own build boundary, under its own grant, and the engine's
  evidence token stays unmintable until such a consumer and a
  qualifying row both exist.
- Not a matrix claim: a row's existence advertises intent; only its
  evidence fields qualify it, and only the engine's evidence-token path
  turns qualification into `supported`.
- Not self-authorizing: this document describes the format; a change to
  what is advertised or qualified is a reviewed change to the data
  files, gated by the store test.
