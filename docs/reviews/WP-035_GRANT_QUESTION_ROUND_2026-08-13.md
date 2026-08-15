# The WP-035 observability-share grant question — recommendation round, 2026-08-13

Untracked session artifact, docs/reviews convention. For the decision
owner: issue #318's governance question, plus a regularization the
question's own investigation surfaced.

## The question as filed

WP-035's `docs/quality/observability.md` share is an **enumerated**
grant (WP-035.md, Boundaries): its SI-33/SI-35 material, increment 6's
named macOS and real-partitioned-Linux protocols and rows, the
SI-33/SI-28 successor protocol's arms, and the file's global status.
Issue #318 asks whether that enumeration extends to a fabric-versus-
local **transport-discrimination protocol** row (ADR-0018's evidence
obligation 2) — and observes it visibly does not.

## What the investigation surfaced beside it

The #318 obligation rows this arc landed — the 2026-08-13 readback
(PR #321) and the floor-rows preregistration and sitting (PRs #324,
#325) — rest on the WP-L100 assignment's *filed obligations on
WP-035* plus merge review, not on the share's enumeration, which
names neither. The ownership checker passes (the path is owned
wholesale at the checker level; the enumeration is prose governance),
but the prose enumeration has lagged what the package now actually
records. An enumerated grant that no longer enumerates its contents
is the stale-count shape applied to governance.

## Routes

- **(a) Read the existing grant expansively** — "observability.md is
  the measurement record; protocols live there; therefore any
  measurement protocol is in-grant." Rejected: it dissolves the
  enumeration, which exists precisely so this file's normative-adjacent
  content moves only by named decision. The register's discipline
  distrusts exactly this move.
- **(b) Extend the share's enumeration explicitly** (recommended for
  what has landed): one WP-035.md Boundaries edit adding "rows filed
  on this package as obligations by another package's assignment,
  including issue #318's readback and floor-rows records" to the
  enumerated list. An assignment self-edit under the ordinary trailer
  — the WP-060 increment-7 precedent — reviewed by merge. Cost: one
  small PR. It regularizes the landed work and gives future filed
  obligations a named home without opening the file wholesale.
- **(c) House the transport-discrimination protocol with its
  consumer, when the consumer exists** (recommended for the protocol):
  the row's consumer is the transport-classification work — WP-040's
  unrecorded route decisions, and any future ADR-0034-pattern
  designation extension. Minting a protocol grant before the protocol
  has a sponsoring package invites a row nobody owns the consequences
  of. When WP-040's assignment (or a successor) records its transport
  route, that assignment files the protocol on WP-035 the same way
  WP-L100 filed #318's items — which lands inside (b)'s new clause
  with no further governance.

## The adversarial pass

1. **Does (b) legitimize scope creep by generalizing "filed
   obligations"?** The clause admits only rows another assignment
   *explicitly files*, each such filing being itself a reviewed
   governance act in the filing package's assignment. Two reviewed
   acts (the filing, the recording) bracket every row. Sustained as
   acceptable.
2. **Was the FR sitting itself out of grant?** Under the strictest
   reading of the prose enumeration, yes — it rested on the #318
   filing and merge review rather than the enumerated list. That is
   this round's motivating defect, recorded rather than argued away;
   (b) regularizes it retroactively and the decision owner's review
   of this round is the acceptance or rejection of that
   regularization.
3. **Does deferring the protocol (route c) block SI-28 or ADR-0018?**
   No: ADR-0018's fabric-versus-local rows are outstanding on every
   platform and nothing currently consumes them; WP-L100's
   `Unrecognized`-everywhere answer is recorded as the correct
   meanwhile state. The protocol becomes urgent exactly when a
   package takes the transport route decision, which is when (c)
   houses it.

## Recommendation

Adopt (b) now — one WP-035.md Boundaries edit, drafted and ready —
and (c) for the transport-discrimination protocol, deferring it to
the package that first records a transport route decision. The
decision is the owner's; this round's job was to make both the
defect and the routes explicit.
