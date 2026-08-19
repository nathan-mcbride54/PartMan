//! The byte layer (increment 2): the head and tail windows of a whole
//! device, read through a read-only handle, classified by
//! `partman-table-parser` — HLP-002's "re-discovers independently" for
//! the one on-disk verdict input the product parses today (ADR-0014: the
//! table state is the helper's to author; ADR-0018: "the helper's own
//! bounded parsers over raw device bytes").
//!
//! **What is read.** Exactly two windows of [`WINDOW_BYTES`] each (the
//! shape M10 measured as separating and the parser's documented caller
//! shape), at the start and at the end of the medium whose geometry the
//! caller states — the adapter's sysfs `size` × 512 and
//! `queue/logical_block_size` (DR21 measured both against the block
//! layer and against the image); a medium shorter than two windows is
//! read whole. No other byte of a device is read before the first write.
//! **How.** `std::fs::File` opened read-only on the device node — no
//! `unsafe`, no ioctl, no write mode — and, before any byte is read, the
//! opened handle's device number checked against the number the sysfs
//! entry states, so the bytes are the bytes of the device the entry
//! describes and never of a node renamed underneath the open (DR21's
//! bracketing). Reads are bounded and exact; a short read is a typed
//! refusal, never a shorter window handed to the parser.
//!
//! **What is not decided here.** The parser's answer is the answer: this
//! module maps nothing, guesses nothing, and reports the classification
//! verbatim. Everything platform-specific — where the node lives, which
//! number it must carry — is the [`DeviceReader`] implementor's; the pure
//! windowing over any `Read + Seek` is what the Tier-1 suite runs over the
//! catalogue's images written to files.

use std::io::{Read, Seek, SeekFrom};

use partman_table_parser::{Classification, Geometry, ParseRefusal, classify};

/// Each window's length, in bytes: 64 KiB, the parser's real-caller shape.
pub const WINDOW_BYTES: u64 = 64 * 1024;

/// The two windows and the geometry they were read under.
#[derive(Clone, Debug)]
pub struct Windows {
    /// The first `min(WINDOW_BYTES, total)` bytes.
    pub head: Vec<u8>,
    /// The last `min(WINDOW_BYTES, total)` bytes.
    pub tail: Vec<u8>,
    /// The geometry the caller stated and the windows were cut by.
    pub geometry: Geometry,
}

/// Why the windows could not be read. Typed; each arm names the step, and
/// none carries a path or an identifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ByteRefusal {
    /// The stated geometry cannot be windowed: a zero sector size, a zero
    /// length, or a byte total that does not fit.
    GeometryUnusable,
    /// The node could not be opened read-only.
    Open {
        /// The I/O error kind, as its `Debug` name.
        kind: String,
    },
    /// The opened handle is not a block device, or carries a device number
    /// other than the one the sysfs entry states — the node is not the
    /// device the entry describes (the DR21 bracketing refused).
    NotTheDevice {
        /// What the entry states.
        expected: String,
        /// What the handle carries.
        found: String,
    },
    /// A seek or read failed.
    Io {
        /// Which step.
        step: &'static str,
        /// The I/O error kind, as its `Debug` name.
        kind: String,
    },
    /// A read returned fewer bytes than the window asks for: the medium is
    /// shorter than its stated geometry, or the device stopped answering.
    ShortRead {
        /// Which window.
        window: &'static str,
        /// Bytes wanted.
        wanted: u64,
        /// Bytes read.
        got: u64,
    },
}

impl std::fmt::Display for ByteRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GeometryUnusable => write!(f, "the stated geometry cannot be windowed"),
            Self::Open { kind } => {
                write!(f, "the device node could not be opened read-only: {kind}")
            }
            Self::NotTheDevice { expected, found } => write!(
                f,
                "the opened node is not the device the entry describes (entry {expected}, node {found})"
            ),
            Self::Io { step, kind } => write!(f, "{step} failed: {kind}"),
            Self::ShortRead {
                window,
                wanted,
                got,
            } => write!(f, "the {window} window read {got} of {wanted} bytes"),
        }
    }
}

/// Cut the two windows from any seekable reader under a stated geometry.
/// Pure over the reader: the Tier-1 suite runs it over the catalogue's
/// images in files; the Linux reader runs it over the opened node.
///
/// # Errors
///
/// [`ByteRefusal`]: the geometry unusable, a seek or read failing, or a
/// short read.
pub fn read_windows<R: Read + Seek>(
    reader: &mut R,
    geometry: Geometry,
) -> Result<Windows, ByteRefusal> {
    let total = u64::from(geometry.sector_size)
        .checked_mul(geometry.total_sectors)
        .filter(|total| *total > 0 && geometry.sector_size > 0)
        .ok_or(ByteRefusal::GeometryUnusable)?;
    let window = WINDOW_BYTES.min(total);
    let head = read_exact_at(reader, 0, window, "head")?;
    let tail = read_exact_at(reader, total - window, window, "tail")?;
    Ok(Windows {
        head,
        tail,
        geometry,
    })
}

fn read_exact_at<R: Read + Seek>(
    reader: &mut R,
    offset: u64,
    length: u64,
    window: &'static str,
) -> Result<Vec<u8>, ByteRefusal> {
    reader
        .seek(SeekFrom::Start(offset))
        .map_err(|e| ByteRefusal::Io {
            step: "seek",
            kind: format!("{:?}", e.kind()),
        })?;
    // `length` is at most WINDOW_BYTES, so the allocation is bounded
    // before any byte arrives.
    let mut buffer =
        vec![0_u8; usize::try_from(length).map_err(|_| ByteRefusal::GeometryUnusable)?];
    let mut filled = 0_usize;
    while filled < buffer.len() {
        match reader.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => {
                return Err(ByteRefusal::Io {
                    step: "read",
                    kind: format!("{:?}", e.kind()),
                });
            }
        }
    }
    if filled < buffer.len() {
        return Err(ByteRefusal::ShortRead {
            window,
            wanted: length,
            got: filled as u64,
        });
    }
    Ok(buffer)
}

/// Classify windows: the parser's verdict, verbatim. A refusal here is the
/// caller's contract violation (a sector size the parser does not define,
/// a medium too small to hold a table), never a statement about the
/// medium — the consumer records it as such.
///
/// # Errors
///
/// [`ParseRefusal`], the parser's own.
pub fn classify_windows(windows: &Windows) -> Result<Classification, ParseRefusal> {
    classify(&windows.head, &windows.tail, windows.geometry)
}

/// What opens and windows a device for the capture. The Linux reader
/// opens `<dev>/<entry>` read-only and brackets it by device number; the
/// Tier-1 fake hands back the catalogue's bytes. Nothing else in the
/// helper touches a device.
pub trait DeviceReader {
    /// Read the two windows of the device the sysfs entry names, under the
    /// stated geometry, refusing unless the node carries `device_number`.
    ///
    /// # Errors
    ///
    /// [`ByteRefusal`].
    fn windows(
        &self,
        entry: &str,
        device_number: &str,
        geometry: Geometry,
    ) -> Result<Windows, ByteRefusal>;
}

/// A reader that refuses every device — the fail-closed default where no
/// device may be opened (every non-Linux build, and any caller that wants
/// a capture without the byte layer).
pub struct NoDevices;

impl DeviceReader for NoDevices {
    fn windows(
        &self,
        _entry: &str,
        _device_number: &str,
        _geometry: Geometry,
    ) -> Result<Windows, ByteRefusal> {
        Err(ByteRefusal::Open {
            kind: "Unsupported".to_owned(),
        })
    }
}
