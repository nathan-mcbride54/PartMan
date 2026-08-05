#!/bin/sh
# PartMan — increment 6 macOS matrix, cell M10: the privileged comparison leg.
#
# Instrument: docs/quality/observability.md, "Increment 6 macOS matrix".
# M10 asks whether the helper-side view can read what the client cannot,
# ON THE SAME ATTACHED FIXTURE. So this script captures BOTH sides of every
# attachment itself: the client interfaces are re-run here as an unprivileged
# user, because the M1-M8 sitting ran on a different machine and a
# cross-machine comparison would not be like-for-like.
#
# Usage (as root, in a disposable or hosted macOS environment ONLY):
#
#     ./macos-m10.sh --disposable-environment <unprivileged-user>
#
# The flag is a required assertion, not a convenience. M10's invalidation
# condition is "any attempt on the ordinary host", and a script that runs
# without being told where it is cannot honour that.

set -u

FLAG="${1:-}"
CLIENT_USER="${2:-}"

if [ "$FLAG" != "--disposable-environment" ] || [ -z "$CLIENT_USER" ]; then
  echo "usage: $0 --disposable-environment <unprivileged-user>" >&2
  echo "" >&2
  echo "Refusing to run without an explicit disposable-environment assertion." >&2
  echo "M10 may not run on an ordinary Mac: that is its invalidation" >&2
  echo "condition, not a preference." >&2
  exit 2
fi

HERE="$(cd -- "$(/usr/bin/dirname -- "$0")" && pwd)"
FIXTURES="$HERE/fixtures"
OUTDIR="$HERE/out-m10"
TRANSCRIPT="$OUTDIR/00-transcript.txt"
TIMEOUT=60
MAXBYTES=1048576

# ---------------------------------------------------------------- tools ----
T_SWVERS=/usr/bin/sw_vers
T_CSRUTIL=/usr/bin/csrutil
T_ID=/usr/bin/id
T_DISKUTIL=/usr/sbin/diskutil
T_HDIUTIL=/usr/bin/hdiutil
T_IOREG=/usr/sbin/ioreg
T_PLUTIL=/usr/bin/plutil
T_STAT=/usr/bin/stat
T_DD=/bin/dd
T_SHASUM=/usr/bin/shasum
T_MOUNT=/sbin/mount
T_SYSCTL=/usr/sbin/sysctl
T_UNAME=/usr/bin/uname
T_SUDO=/usr/bin/sudo
T_XXD=/usr/bin/xxd
T_SED=/usr/bin/sed
T_HEAD=/usr/bin/head
T_MKTEMP=/usr/bin/mktemp
T_MKDIR=/bin/mkdir
T_RM=/bin/rm
T_CP=/bin/cp
T_TEE=/usr/bin/tee
T_DATE=/bin/date
T_CHMOD=/bin/chmod
T_SLEEP=/bin/sleep
T_MV=/bin/mv

ALL_TOOLS="$T_SWVERS $T_CSRUTIL $T_ID $T_DISKUTIL $T_HDIUTIL $T_IOREG $T_PLUTIL $T_STAT $T_DD $T_SHASUM $T_MOUNT $T_SYSCTL $T_UNAME $T_SUDO $T_XXD $T_SED $T_HEAD $T_MKTEMP $T_MKDIR $T_RM $T_CP $T_TEE $T_DATE $T_CHMOD $T_SLEEP $T_MV"

"$T_MKDIR" -p "$OUTDIR" || exit 1
: > "$TRANSCRIPT"

log() { printf '%s\n' "$*" | "$T_TEE" -a "$TRANSCRIPT" ; }
sec() { log "" ; log "=== $* ===" ; }

run() {
  _id="$1"; shift
  _out="$OUTDIR/$_id.out"
  _err="$OUTDIR/$_id.err"
  "$@" >"$_out" 2>"$_err" &
  _pid=$!
  ( "$T_SLEEP" "$TIMEOUT"; kill -9 "$_pid" 2>/dev/null ) >/dev/null 2>&1 &
  _watch=$!
  wait "$_pid" 2>/dev/null
  _rc=$?
  kill "$_watch" 2>/dev/null
  wait "$_watch" 2>/dev/null
  for _f in "$_out" "$_err"; do
    _sz=$("$T_STAT" -f%z "$_f" 2>/dev/null || echo 0)
    if [ "$_sz" -gt "$MAXBYTES" ]; then
      "$T_HEAD" -c "$MAXBYTES" "$_f" > "$_f.trunc" 2>/dev/null
      printf '\n[TRUNCATED at %s bytes]\n' "$MAXBYTES" >> "$_f.trunc"
      "$T_MV" "$_f.trunc" "$_f"
    fi
  done
  if [ "$_rc" -eq 137 ]; then
    log "step $_id: exit 137 — TIMEOUT → void(timeout)"
  else
    log "step $_id: exit $_rc   argv: $*"
  fi
  return 0
}

pl() { "$T_PLUTIL" -extract "$1" raw -o - "$2" 2>/dev/null ; }

# =========================================================== PHASE A =======
sec "M10 PHASE A — environment record, $("$T_DATE" -u '+%Y-%m-%dT%H:%M:%SZ')"

if [ "$("$T_ID" -u)" != "0" ]; then
  log "REFUSED: M10 is the privileged leg and must run as root."
  exit 1
fi
log "effective uid: 0 (root) — this is the privileged comparison leg"
log "disposable-environment assertion: GIVEN by the operator on the command line"
log "unprivileged client user for the paired half: $CLIENT_USER"

if ! "$T_ID" "$CLIENT_USER" >/dev/null 2>&1; then
  log "REFUSED: client user '$CLIENT_USER' does not exist."
  exit 1
fi
if [ "$("$T_ID" -u "$CLIENT_USER")" = "0" ]; then
  log "REFUSED: client user '$CLIENT_USER' is uid 0. The paired half must be"
  log "unprivileged or the comparison is vacuous."
  exit 1
fi

if [ -n "${SSH_CONNECTION:-}" ] || [ -n "${SSH_TTY:-}" ]; then
  SESSION="ssh"
else
  SESSION="console"
fi
log "session type: $SESSION"

run A01-swvers   "$T_SWVERS"
run A02-uname    "$T_UNAME" -a
run A03-csrutil  "$T_CSRUTIL" status
run A04-id-root  "$T_ID"
run A05-boottime "$T_SYSCTL" -n kern.boottime
run A06-cpu      "$T_SYSCTL" -n machdep.cpu.brand_string

sec "M10 PHASE A — tool identity"
for t in $ALL_TOOLS; do
  if [ -x "$t" ]; then
    "$T_SHASUM" -a 256 "$t" | "$T_TEE" -a "$TRANSCRIPT"
  else
    log "tool MISSING: $t — voids the cells naming it"
  fi
done

sec "M10 PHASE A — capture-script digest (before any attach)"
"$T_SHASUM" -a 256 "$0" | "$T_TEE" -a "$TRANSCRIPT"

sec "M10 PHASE A — fixture manifest"
if [ ! -f "$FIXTURES/MANIFEST" ]; then
  log "MANIFEST ABSENT — every row is void(no-manifest)"
  exit 1
fi
"$T_SHASUM" -a 256 "$FIXTURES/MANIFEST" | "$T_TEE" -a "$TRANSCRIPT"
"$T_CP" "$FIXTURES/MANIFEST" "$OUTDIR/A07-MANIFEST.copy"
for f in "$FIXTURES"/*.img; do
  [ -e "$f" ] || continue
  "$T_SHASUM" -a 256 "$f" | "$T_TEE" -a "$TRANSCRIPT"
done

# =========================================================== PHASE B =======
sec "M10 PHASE B — paired client/helper capture per fixture"

SCRATCH="$("$T_MKTEMP" -d)"
"$T_CHMOD" 755 "$SCRATCH"
log "scratch: $SCRATCH"
DETACH_FAILED=0

# Reads a byte range from a device or file and prints its SHA-256. The point
# of M10 is byte-level: not "does a tool name the signature" but "can this
# side read the bytes at all".
range_digest() {
  # $1 = path, $2 = skip (512-blocks), $3 = count (512-blocks), $4 = label
  "$T_DD" if="$1" bs=512 skip="$2" count="$3" 2>/dev/null | "$T_SHASUM" -a 256 | "$T_SED" "s|-|$4|"
}

m10_row() {
  _name="$1"
  _img="$FIXTURES/$_name.img"

  if [ "$DETACH_FAILED" -ne 0 ]; then
    log "$_name: SKIPPED — an earlier detach failed"
    return 0
  fi
  [ -f "$_img" ] || { log "$_name: fixture absent — void(fixture-absent)"; return 0; }

  sec "M10 row: $_name"
  "$T_CP" "$_img" "$SCRATCH/$_name.img"
  "$T_CHMOD" 644 "$SCRATCH/$_name.img"
  log "--- digest before attach ---"
  "$T_SHASUM" -a 256 "$SCRATCH/$_name.img" | "$T_TEE" -a "$TRANSCRIPT"

  _sz=$("$T_STAT" -f%z "$SCRATCH/$_name.img")
  _sectors=$((_sz / 512))
  _tail_skip=$((_sectors - 128))
  log "image: $_sz bytes, $_sectors sectors of 512; tail window starts at sector $_tail_skip"

  run "M10-$_name-01-attach" "$T_HDIUTIL" attach \
      -imagekey diskimage-class=CRawDiskImage -nomount -readonly -plist \
      "$SCRATCH/$_name.img"

  _dev=""
  i=0
  while [ "$i" -lt 16 ]; do
    e="$(pl "system-entities.$i.dev-entry" "$OUTDIR/M10-$_name-01-attach.out")"
    [ -z "$e" ] && break
    case "$_dev" in
      "") _dev="$e" ;;
      *) [ "${#e}" -lt "${#_dev}" ] && _dev="$e" ;;
    esac
    i=$((i+1))
  done
  [ -z "$_dev" ] && { log "$_name: no dev-entry → denied/not-attached"; return 0; }
  _raw="$(printf '%s' "$_dev" | "$T_SED" 's|/dev/disk|/dev/rdisk|')"
  log "$_name: attached $_dev (raw node $_raw), attached BY ROOT"

  # ---- CLIENT HALF, unprivileged, on THIS attachment ----
  log "--- CLIENT half (uid of $CLIENT_USER), same attachment ---"
  run "M10-$_name-10-client-info" "$T_SUDO" -u "$CLIENT_USER" "$T_DISKUTIL" info -plist "$_dev"
  run "M10-$_name-11-client-list" "$T_SUDO" -u "$CLIENT_USER" "$T_DISKUTIL" list -plist "$_dev"
  run "M10-$_name-12-client-ddhead" "$T_SUDO" -u "$CLIENT_USER" "$T_DD" "if=$_raw" bs=512 count=1 of=/dev/null
  run "M10-$_name-13-client-stat" "$T_STAT" -f "%N mode=%Sp owner=%Su group=%Sg" "$_dev" "$_raw"

  # ---- HELPER HALF, root, same attachment ----
  log "--- HELPER half (root), same attachment ---"
  run "M10-$_name-20-helper-head" "$T_DD" "if=$_raw" bs=512 count=128 "of=$OUTDIR/M10-$_name-head.bin"
  run "M10-$_name-21-helper-tail" "$T_DD" "if=$_raw" bs=512 "skip=$_tail_skip" count=128 "of=$OUTDIR/M10-$_name-tail.bin"

  log "--- byte-range digests: device (helper) vs source image (ground truth) ---"
  range_digest "$_raw"                  0            128 "  HELPER-head  $_name" | "$T_TEE" -a "$TRANSCRIPT"
  range_digest "$SCRATCH/$_name.img"    0            128 "  SOURCE-head  $_name" | "$T_TEE" -a "$TRANSCRIPT"
  range_digest "$_raw"                  "$_tail_skip" 128 "  HELPER-tail  $_name" | "$T_TEE" -a "$TRANSCRIPT"
  range_digest "$SCRATCH/$_name.img"    "$_tail_skip" 128 "  SOURCE-tail  $_name" | "$T_TEE" -a "$TRANSCRIPT"

  # A small, bounded hexdump of the first sector, so the record can show what
  # the helper saw rather than only assert that it saw something.
  run "M10-$_name-22-helper-hexhead" "$T_XXD" -l 512 "$OUTDIR/M10-$_name-head.bin"

  run "M10-$_name-30-mount" "$T_MOUNT"
  run "M10-$_name-40-detach" "$T_HDIUTIL" detach "$_dev"
  if [ -e "$_dev" ]; then
    log "$_name: DETACH FAILED — blocks every later row"
    DETACH_FAILED=1
  else
    log "$_name: detach confirmed"
  fi
  log "--- digest after detach ---"
  "$T_SHASUM" -a 256 "$SCRATCH/$_name.img" | "$T_TEE" -a "$TRANSCRIPT"
}

# The decisive SI-35 pair, and the signatures the client reported as
# indistinguishable from blank. These are exactly the cases where a
# helper/client asymmetry would show.
m10_row gpt-basic-512
m10_row gpt-conflicting-tables-512
m10_row blank-512
m10_row ext4-with-stale-mdraid-090-512
m10_row mdraid-1.2-member-512
m10_row luks2-whole-disk-512
m10_row lvm2-pv-orphan-512

sec "M10 — closing assertions"
run Z01-hdiutil-info "$T_HDIUTIL" info
run Z02-mount-final  "$T_MOUNT"
"$T_RM" -rf "$SCRATCH"
log "scratch removed"

sec "M10 DONE"
log "Hand back the whole out-m10 directory."
log ""
log "This leg establishes only what it measured: the helper-side byte view of"
log "the same attachment the client saw. It does not decide SI-34, choose an"
log "SI-35 option, or extend to interfaces not exercised here."
