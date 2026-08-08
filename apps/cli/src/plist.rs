//! A bounded reader for the XML property lists `diskutil` emits, and
//! nothing more general than that.
//!
//! WP-035's charter and the increment 8 pattern govern the shape: **refusal
//! over generality**. This module reads exactly the grammar the measured
//! `diskutil list -plist` and `diskutil info -plist` captures use — the XML
//! declaration, one DOCTYPE, and `plist`/`dict`/`key`/`string`/`integer`/
//! `true`/`false`/`array` elements — and refuses everything else with a
//! typed error: `data`, `date`, `real`, comments, CDATA, processing
//! instructions past the prolog, numeric character references, a DOCTYPE
//! internal subset, duplicate dictionary keys, non-UTF-8 bytes, oversize
//! values, over-depth nesting. A mangled or guessed value is not the raw
//! string the interface reported, so nothing here substitutes, truncates,
//! or skips-and-continues: any unexpected byte fails the whole read.
//!
//! Pure by construction — no I/O, no process, no environment. The bytes
//! arrive from the launcher seam, already bounded by its per-stream limit;
//! the caps here are the parser's own, so the guarantee does not depend on
//! who called it. This is a parser of externally supplied bytes, and the
//! Section 11.4 posture applies: it must stay `unsafe`-free (the workspace
//! lint denies `unsafe` crate-wide) and carries a fuzz obligation, which
//! lands in the `fuzz/` crate under its own ownership.

/// The largest input this reader will consider, in bytes. The launcher's
/// enumeration bound is the operational cap; this one keeps the parser's
/// promise independent of the caller.
pub const INPUT_LIMIT: usize = 4 * 1024 * 1024;

/// The deepest container nesting accepted. The measured captures nest four
/// levels; sixteen is headroom, not generality.
pub const DEPTH_LIMIT: usize = 16;

/// The most parsed values (scalars and containers) accepted in one
/// document, a fail-closed bound on adversarial inputs made of tiny nodes.
pub const NODE_LIMIT: usize = 65_536;

/// The longest accepted text run — one string value, one key, one integer —
/// matching the enumeration adapters' per-value bound.
pub const VALUE_LIMIT: usize = 4096;

/// Why a read was refused. Every variant is a refusal of the whole input:
/// this reader never returns a partial answer.
#[derive(Debug, PartialEq, Eq)]
pub enum PlistRefusal {
    /// The input is not valid UTF-8. Nothing is substituted.
    NotUtf8,
    /// The input exceeds [`INPUT_LIMIT`].
    OverSize,
    /// Containers nest deeper than [`DEPTH_LIMIT`].
    OverDepth,
    /// More than [`NODE_LIMIT`] values in one document.
    OverNodeCount,
    /// A text run exceeds [`VALUE_LIMIT`].
    OverValueLength,
    /// A construct this reader deliberately does not implement. The payload
    /// names it, compile-time and closed.
    Unsupported(&'static str),
    /// The input does not follow the accepted grammar. The payload names
    /// the expectation that failed, compile-time and closed.
    Malformed(&'static str),
}

impl PlistRefusal {
    /// One human-actionable sentence, safe for in-band error reporting.
    #[must_use]
    pub fn detail(&self) -> String {
        match self {
            Self::NotUtf8 => {
                "the plist bytes are not UTF-8; refused rather than substituted".to_owned()
            }
            Self::OverSize => {
                format!("the plist exceeds {INPUT_LIMIT} bytes; refused rather than truncated")
            }
            Self::OverDepth => format!("the plist nests deeper than {DEPTH_LIMIT} levels; refused"),
            Self::OverNodeCount => {
                format!("the plist holds more than {NODE_LIMIT} values; refused")
            }
            Self::OverValueLength => {
                format!("a plist value exceeds {VALUE_LIMIT} bytes; refused rather than truncated")
            }
            Self::Unsupported(what) => {
                format!("the plist uses a construct this bounded reader refuses: {what}")
            }
            Self::Malformed(what) => {
                format!("the plist does not follow the accepted grammar: {what}")
            }
        }
    }
}

/// One parsed value. Strings and integers stay raw text — nothing is
/// numerically interpreted, matching the charter's raw-identifier rule.
pub enum Value {
    /// `<dict>`: keys in document order, each with its value.
    Dict(Vec<(String, Value)>),
    /// `<array>`.
    Array(Vec<Value>),
    /// `<string>`, entity-decoded, otherwise verbatim.
    String(String),
    /// `<integer>`, as the raw digit run the document spelled.
    Integer(String),
    /// `<true/>` or `<false/>`.
    Bool(bool),
}

/// Parse one complete plist document.
///
/// # Errors
///
/// Any deviation from the accepted grammar refuses the whole input; see
/// [`PlistRefusal`].
pub fn parse(bytes: &[u8]) -> Result<Value, PlistRefusal> {
    if bytes.len() > INPUT_LIMIT {
        return Err(PlistRefusal::OverSize);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| PlistRefusal::NotUtf8)?;
    let mut cursor = Cursor {
        rest: text,
        nodes: 0,
    };
    cursor.skip_whitespace();
    if cursor.take_prefix("<?xml") {
        cursor.scan_past("?>", "an unterminated XML declaration")?;
    }
    cursor.skip_whitespace();
    if cursor.take_prefix("<!DOCTYPE") {
        // An internal subset can define entities, which this reader will
        // not expand; refusing the subset is what keeps that honest.
        let doctype = cursor.scan_past(">", "an unterminated DOCTYPE")?;
        if doctype.contains('[') {
            return Err(PlistRefusal::Unsupported("a DOCTYPE internal subset"));
        }
    }
    cursor.skip_whitespace();
    if !cursor.take_prefix("<plist") {
        return Err(PlistRefusal::Malformed("no <plist> root element"));
    }
    cursor.scan_past(">", "an unterminated <plist> open tag")?;
    cursor.skip_whitespace();
    let value = cursor.value(0)?;
    cursor.skip_whitespace();
    if !cursor.take_prefix("</plist>") {
        return Err(PlistRefusal::Malformed("content after the root value"));
    }
    cursor.skip_whitespace();
    if !cursor.rest.is_empty() {
        return Err(PlistRefusal::Malformed("bytes after </plist>"));
    }
    Ok(value)
}

/// Extract the `WholeDisks` string array from a `diskutil list -plist`
/// document: the whole-device names, exactly as the interface spelled them.
///
/// # Errors
///
/// Refuses on any parse refusal, a non-dict root, a missing or non-array
/// `WholeDisks` key, or a non-string element.
pub fn whole_disks(bytes: &[u8]) -> Result<Vec<String>, PlistRefusal> {
    let Value::Dict(entries) = parse(bytes)? else {
        return Err(PlistRefusal::Malformed("the list root is not a dict"));
    };
    let mut found = None;
    for (key, value) in entries {
        if key == "WholeDisks" {
            found = Some(value);
        }
    }
    let Some(Value::Array(elements)) = found else {
        return Err(PlistRefusal::Malformed(
            "no WholeDisks array in the list output",
        ));
    };
    let mut names = Vec::with_capacity(elements.len());
    for element in elements {
        let Value::String(name) = element else {
            return Err(PlistRefusal::Malformed(
                "a WholeDisks element is not a string",
            ));
        };
        names.push(name);
    }
    Ok(names)
}

/// One top-level entry of a `diskutil info -plist` dictionary, as the
/// adapter consumes it.
pub enum InfoValue {
    /// A scalar the adapter may report raw: string, integer text, or the
    /// textual form of a boolean.
    Scalar(String),
    /// A positively empty string value — present, and empty.
    EmptyString,
    /// A nested container. Present, but not a scalar; the adapter reports
    /// that shape honestly instead of flattening it.
    Container,
}

/// Extract the top-level entries of a `diskutil info -plist` document.
///
/// Containers are parsed (they must be well-formed for the document to be
/// accepted) but returned as [`InfoValue::Container`], never flattened.
///
/// # Errors
///
/// Refuses on any parse refusal or a non-dict root.
pub fn info_fields(bytes: &[u8]) -> Result<Vec<(String, InfoValue)>, PlistRefusal> {
    let Value::Dict(entries) = parse(bytes)? else {
        return Err(PlistRefusal::Malformed("the info root is not a dict"));
    };
    Ok(entries
        .into_iter()
        .map(|(key, value)| {
            let info = match value {
                Value::String(text) if text.is_empty() => InfoValue::EmptyString,
                Value::String(text) | Value::Integer(text) => InfoValue::Scalar(text),
                Value::Bool(flag) => {
                    InfoValue::Scalar(if flag { "true" } else { "false" }.to_owned())
                }
                Value::Dict(_) | Value::Array(_) => InfoValue::Container,
            };
            (key, info)
        })
        .collect())
}

/// The parse state: a shrinking suffix of the input and a node budget.
struct Cursor<'a> {
    rest: &'a str,
    nodes: usize,
}

impl Cursor<'_> {
    fn skip_whitespace(&mut self) {
        self.rest = self.rest.trim_start_matches([' ', '\t', '\r', '\n']);
    }

    /// Consume `prefix` if present, reporting whether it was.
    fn take_prefix(&mut self, prefix: &str) -> bool {
        match self.rest.strip_prefix(prefix) {
            Some(rest) => {
                self.rest = rest;
                true
            }
            None => false,
        }
    }

    /// Consume through the next occurrence of `terminator`, returning the
    /// skipped text (terminator excluded).
    fn scan_past(&mut self, terminator: &str, missing: &'static str) -> Result<&str, PlistRefusal> {
        let Some(position) = self.rest.find(terminator) else {
            return Err(PlistRefusal::Malformed(missing));
        };
        let skipped = &self.rest[..position];
        self.rest = &self.rest[position + terminator.len()..];
        Ok(skipped)
    }

    /// Parse one value at the given container depth. The root value is
    /// depth 0, so the check refuses the container that would *start*
    /// nesting level `DEPTH_LIMIT + 1` — sixteen levels parse, seventeen
    /// refuse.
    fn value(&mut self, depth: usize) -> Result<Value, PlistRefusal> {
        if depth >= DEPTH_LIMIT {
            return Err(PlistRefusal::OverDepth);
        }
        self.nodes += 1;
        if self.nodes > NODE_LIMIT {
            return Err(PlistRefusal::OverNodeCount);
        }
        if self.take_prefix("<dict>") {
            return self.dict_body(depth);
        }
        if self.take_prefix("<dict/>") {
            return Ok(Value::Dict(Vec::new()));
        }
        if self.take_prefix("<array>") {
            return self.array_body(depth);
        }
        if self.take_prefix("<array/>") {
            return Ok(Value::Array(Vec::new()));
        }
        if self.take_prefix("<string>") {
            let text = self.text_until("</string>")?;
            return Ok(Value::String(text));
        }
        if self.take_prefix("<string/>") {
            return Ok(Value::String(String::new()));
        }
        if self.take_prefix("<integer>") {
            let text = self.text_until("</integer>")?;
            let digits = text.strip_prefix('-').unwrap_or(&text);
            if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(PlistRefusal::Malformed("a non-digit integer body"));
            }
            return Ok(Value::Integer(text));
        }
        if self.take_prefix("<true/>") {
            return Ok(Value::Bool(true));
        }
        if self.take_prefix("<false/>") {
            return Ok(Value::Bool(false));
        }
        // Name the constructs the format defines and this reader refuses,
        // so the refusal is legible; anything else is malformed.
        for (prefix, name) in [
            ("<data", "a data element"),
            ("<date", "a date element"),
            ("<real", "a real element"),
            ("<!--", "a comment"),
            ("<![CDATA[", "a CDATA section"),
            ("<?", "a processing instruction past the prolog"),
        ] {
            if self.rest.starts_with(prefix) {
                return Err(PlistRefusal::Unsupported(name));
            }
        }
        Err(PlistRefusal::Malformed("an unrecognized element"))
    }

    /// Parse `<key>…</key>` / value pairs through `</dict>`.
    fn dict_body(&mut self, depth: usize) -> Result<Value, PlistRefusal> {
        let mut entries: Vec<(String, Value)> = Vec::new();
        loop {
            self.skip_whitespace();
            if self.take_prefix("</dict>") {
                return Ok(Value::Dict(entries));
            }
            if !self.take_prefix("<key>") {
                return Err(PlistRefusal::Malformed("a dict entry without a <key>"));
            }
            let key = self.text_until("</key>")?;
            // Two values under one key is an ambiguity, not a list. The
            // format's own tooling keeps the last; keeping either silently
            // would decide which report is "the" value, so neither is kept.
            if entries.iter().any(|(existing, _)| *existing == key) {
                return Err(PlistRefusal::Malformed("a duplicate dict key"));
            }
            self.skip_whitespace();
            let value = self.value(depth + 1)?;
            entries.push((key, value));
        }
    }

    /// Parse values through `</array>`.
    fn array_body(&mut self, depth: usize) -> Result<Value, PlistRefusal> {
        let mut elements = Vec::new();
        loop {
            self.skip_whitespace();
            if self.take_prefix("</array>") {
                return Ok(Value::Array(elements));
            }
            elements.push(self.value(depth + 1)?);
        }
    }

    /// Read text through the given closing tag, decoding exactly the five
    /// predefined XML entities. A numeric character reference or an
    /// undefined entity refuses: expanding one would put bytes in the
    /// output that the interface did not spell, and declining to expand it
    /// would report a raw `&#…;` as though the interface said that.
    fn text_until(&mut self, closing: &str) -> Result<String, PlistRefusal> {
        let raw = self.scan_past(closing, "an unterminated text element")?;
        if raw.len() > VALUE_LIMIT {
            return Err(PlistRefusal::OverValueLength);
        }
        if raw.contains('<') {
            return Err(PlistRefusal::Malformed("markup inside a text element"));
        }
        let mut out = String::with_capacity(raw.len());
        let mut rest = raw;
        while let Some(position) = rest.find('&') {
            out.push_str(&rest[..position]);
            rest = &rest[position..];
            let mut decoded = None;
            for (entity, replacement) in [
                ("&lt;", '<'),
                ("&gt;", '>'),
                ("&amp;", '&'),
                ("&quot;", '"'),
                ("&apos;", '\''),
            ] {
                if let Some(after) = rest.strip_prefix(entity) {
                    decoded = Some((replacement, after));
                    break;
                }
            }
            let Some((replacement, after)) = decoded else {
                return Err(PlistRefusal::Unsupported(
                    "a character reference beyond the five predefined entities",
                ));
            };
            out.push(replacement);
            rest = after;
        }
        out.push_str(rest);
        Ok(out)
    }
}
