# Fuzzing

Section 11.4 of `AGENT_BUILD_SPEC.md` requires `cargo-fuzz` targets for every
parser of on-disk or externally supplied bytes, short smoke runs gating pull
requests, scheduled long runs accumulating corpora, and a release gate of zero
untriaged crashes or hangs.

## Targets

`fuzz/fuzz_targets/` currently holds four. The first two drive the `pce/1`
codec, which is a plan and journal deserializer under Section 11.4's list;
the third drives the CLI's bounded plist reader, a parser of externally
supplied subprocess bytes; the fourth drives the table parser, the SI-35
resolution's classifier of raw on-disk table bytes.

### `decode_is_canonical`

Asserts the property that makes the plan hash an authorization boundary:

> For any input, `decode` either fails, or returns a value whose `encode`
> reproduces the input **byte for byte**.

If the decoder ever accepted a non-canonical encoding, an attacker could submit
bytes that decode to an approved plan yet hash differently, so the bytes a user
authorized would not be the bytes describing what executes. The target also
asserts that hashing an accepted value agrees with hashing the bytes it came
from, and that re-encoded bytes decode back.

Every decoded array is also passed through the schema-level canonical-set
validator. Descending or duplicate arrays remain valid semantic `pce/1` arrays
and are allowed to fail that schema check; when the validator accepts one as a
set, the set producer must reproduce the input byte for byte.

### `roundtrip_value`

Drives the encoder from structured values rather than bytes, reaching shapes
random bytes almost never produce: deep nesting, maps whose key order differs
between insertion and encoding, and integers at every argument-width boundary.
Asserts `decode(encode(v)) == v` and that encoding is a fixed point.

For generated top-level arrays, the same target also drives the schema-set
producer. Unique elements must sort into bytes that decode as a strictly ordered
set and form their own fixed point; duplicate logical elements must be refused
rather than removed. This extends the two existing targets rather than adding a
third target that would exercise the same value generator and parser.

`Value` is built from raw entropy inside the target rather than by deriving
`Arbitrary` on the domain type, so the domain crate carries no fuzzing
dependency.

### `plist_bounds_hold`

Drives `apps/cli`'s bounded XML plist reader — WP-035 increment 9's parser
for `diskutil` output, and the target that increment recorded as in flight
when it landed. Asserts the property that makes the reader's promise a
property rather than prose:

> For any input, `parse` either refuses with a typed error, or returns a
> value inside every bound the module declares — container depth, total
> value count, and per-text-run length.

The two extraction entry points (`whole_disks`, `info_fields`) run on every
input too: they must never panic, and an input either accepts must be an
input `parse` accepts, so extraction stays a view over the grammar rather
than a second grammar. Reachability under the engine's 4,096-byte input cap
is stated exactly, because a bound the fuzzer cannot reach is asserted
prose, not a searched property: the **depth** cap is reachable (seventeen
nested containers fit in ~120 bytes) and is what this target genuinely
searches; the oversize, over-value, and over-node refusals all need inputs
larger than the cap, so they rest on the stable unit tests, where all
three are covered — the node cap's boundary test landed with the WP-035
change that followed this target, closing the gap this sentence once
recorded. What the target
adds beyond the caps is the panic-freedom and extractor-consistency search
over the whole grammar. The CLI crate carries no fuzzing dependency:
like the codec targets, this one lives here, and `fuzz/` alone depends on
`partman-cli` — the shipped binary's empty dependency closure is untouched.

What runs a target is `FUZZ_TARGETS` in `tools/xtask/src/main.rs`, WP-000's
row, which registered this target in the change that followed its landing —
so the recorded hours-wide gap between "exists and builds" and "driven by
every smoke and scheduled run" opened and closed as designed, two owners,
two changes, neither ahead of its code.

### `table_claims_never_vanish`

Drives `crates/table-parser` — the SI-35 resolution's raw-sector
classifier, ADR-0014's contract, and a Section 11.4 parser of on-disk
metadata in the most literal sense. Asserts panic-freedom over arbitrary
windows and geometry, that broken calls land in typed refusals, and the
load-bearing safety line as a searched property:

> A claimed table never classifies as `Absent`: if the head carries a
> protective-MBR `0xEE` entry or a GPT magic at LBA 1, the answer is
> `Present` or `Indeterminate`, never blank.

That line is what PART-001's categorical invariant will key off, and a
byte pattern that smuggled a claimed-but-mangled table into `Absent`
would be the unreadable-collapses-into-absent conflation ADR-C4 refused.
Reachability under the engine's 4,096-byte cap, stated exactly: claim
shapes (magic, protective and hybrid MBRs) and every refusal arm are
easily reachable; a fully CRC-valid GPT copy is corpus-dependent, so the
`Present` paths rest primarily on the stable unit suite, which classifies
every catalogue fixture and mutation-verifies both `Indeterminate` arms.
What the target genuinely searches is the claimed-never-`Absent` line and
panic-freedom over the grammar nobody hand-writes.

What runs a target is `FUZZ_TARGETS` in `tools/xtask/src/main.rs`,
WP-000's row, which registered this target in the change that followed
its landing — the recorded two-owner gap opened and closed as designed a
second time, neither change ahead of its code.

## The same property, on stable

`crates/domain/tests/canonicality.rs` asserts the *same* property as
`decode_is_canonical`, on the pinned stable toolchain, over a bounded
deterministic mutation space: every single-bit flip, truncation, and boundary
byte substitution of every known-good encoding, plus every one- and two-byte
input exhaustively.

That is not a substitute for fuzzing, and does not claim to be. It exists so
that the property is verifiable on any developer machine without a nightly
toolchain, and so a regression fails `cargo xtask ci` rather than waiting for a
scheduled fuzz run. Fuzzing searches far deeper; the stable test guarantees the
floor.

## Toolchain exception

`cargo-fuzz` needs nightly for libFuzzer. `rust-toolchain.toml` pins one stable
release, and `AGENTS.md` states the workspace toolchain is pinned there — so
fuzzing is an explicit, bounded exception to that rule:

- The nightly is pinned **by exact date**, `nightly-2026-07-01`, for the same
  reason the stable toolchain is pinned by version: an unpinned `nightly`
  changes under CI without a commit, which would make a fuzz failure
  unreproducible.
- The pin appears in `FUZZ_TOOLCHAIN` in `tools/xtask/src/main.rs` and in
  `.github/workflows/ci.yml`. They must move together; neither is covered by
  Dependabot.
- `fuzz/` is **excluded from the workspace**, so `cargo xtask ci` never attempts
  to build it on stable and the exception cannot leak into ordinary builds.
- `cargo-fuzz` itself is pinned to 0.13.2 and installed with `--locked`.
- Exclusion from the workspace also excludes this crate from the root
  `Cargo.lock` and from the supply-chain gates that read it — a gap the
  2026-07-29 audit demonstrated, since the fuzz lock was gitignored and every
  fresh CI run resolved `libfuzzer-sys` and `arbitrary` to whatever the
  registry served that day. `fuzz/Cargo.lock` is now **committed**;
  `cargo xtask fuzz` refuses to run if it no longer matches the manifest;
  `cargo xtask supply-chain` checks this graph against the same `deny.toml` as
  the workspace; and a dedicated `/fuzz` Dependabot entry updates it. Pinning
  the runner was never the same thing as pinning the code it builds.

Nothing outside `fuzz/` may require nightly.

## Running it

```text
cargo xtask fuzz
```

Runs each target for 60 seconds. `--seconds <n>` overrides that for a longer
local run. The command is deliberately not part of `cargo xtask ci`, because it
needs a toolchain the rest of the repository does not.

Every invocation also passes an explicit resource contract to libFuzzer:

- inputs are limited to 4,096 bytes;
- one input has 25 seconds before it is reported as a timeout;
- one allocation of 256 MiB is reported as a failure; and
- the in-process fuzzer has a 4 GiB aggregate RSS ceiling.

The single-allocation and aggregate limits are intentionally separate. Raising
the aggregate ceiling must not permit a hostile input to request an
unreasonably large allocation in one call.

Prerequisites:

```text
rustup toolchain install nightly-2026-07-01 --profile minimal
cargo install cargo-fuzz --version 0.13.2 --locked
```

Note that libFuzzer support is Linux and macOS; the CI job runs on
`ubuntu-24.04`. On Windows the stable canonicality test still runs under
`cargo xtask ci`.

## Corpora and crash artifacts

`fuzz/corpus/` and `fuzz/artifacts/` are git-ignored. Section 11.3 keeps binary
fixtures out of the repository, and a fuzzing corpus is exactly that. A crash
leaves a reproducer in `fuzz/artifacts/`, which the CI job uploads on failure so
it can be attached to a report rather than committed.

The `Maintenance` workflow runs every Monday at 06:00 UTC and is also manually
triggerable. It restores the newest earlier `fuzz/corpus/` cache, gives each
current target 900 seconds, and saves the expanded corpus under an immutable
per-run key. A scheduled run therefore explores for 60 minutes across the
four current targets and starts from discoveries made by earlier successful
runs; the pull-request job remains a fresh 60-second-per-target smoke pass.

The [first full maintenance
run](https://github.com/nathan-mcbride54/PartMan/actions/runs/30582127980/job/91004698510)
proved the duration was materially different from the smoke pass: after the
decoder target completed 15 minutes, `roundtrip_value` stopped after 1.9 million
cases at libFuzzer's default 2 GiB RSS ceiling. Its diagnostic reported about
26 MiB of live heap and a largest live allocation of about 21 MiB, so this was
the long-lived AddressSanitizer process's high-water mark rather than evidence
that the uploaded final input alone requested 2 GiB. The explicit 4 GiB RSS
ceiling accommodates that measured instrumentation overhead on GitHub's
[16 GiB public Ubuntu
runner](https://docs.github.com/en/actions/reference/runners/github-hosted-runners#standard-github-hosted-runners-for-public-repositories).
The independent 256 MiB single-allocation limit ensures the repair does not
classify an input-specific allocation explosion as acceptable.

The corpus cache is an optimization, not evidence of correctness and not a
release artifact. GitHub may evict it. Stable deterministic canonicality tests
remain the floor, and crash reproducers are uploaded separately on failure.

## Not yet done

Section 11.4 requires more than exists today, and the gaps are recorded rather
than implied:

- **Parsers that do not exist yet** have no targets: GPT, MBR, and APM headers,
  file-system probes, and LVM, LUKS, and mdraid metadata. Each arrives with its
  own work package and must bring its own target.
- **The release gate** of zero untriaged crashes or hangs has no automated
  check, because there is no release pipeline yet.
