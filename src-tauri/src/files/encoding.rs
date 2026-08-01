//! File encoding and line-ending detection (DEVELOPMENT_PLAN.md §6.1, M1
//! acceptance: "打开 UTF-8、UTF-8 BOM、LF、CRLF 文件后保存不发生意外编码变化").
//!
//! LiteMark treats every Markdown file as UTF-8 with an optional BOM. Any other
//! encoding is rejected with `FILE_ENCODING_UNSUPPORTED` rather than silently
//! transcoded (silent transcoding risks data loss on round-trip). Line endings
//! (LF / CRLF) are detected and preserved on write.

use crate::error::{ErrorCode, SidecarError};

/// The byte order mark for UTF-8.
pub const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];

/// Detected encoding of a file. LiteMark only reads/writes UTF-8 (with or
/// without BOM); anything else is unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Encoding {
    /// UTF-8 without a BOM.
    #[serde(rename = "utf-8")]
    Utf8,
    /// UTF-8 with a leading BOM (preserved on write).
    #[serde(rename = "utf-8-bom")]
    Utf8Bom,
}

impl Encoding {
    /// Whether a BOM should be written for this encoding.
    pub fn has_bom(self) -> bool {
        matches!(self, Encoding::Utf8Bom)
    }
}

/// Detected line-ending style of a file's content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineEnding {
    /// `\n`
    Lf,
    /// `\r\n`
    Crlf,
}

impl LineEnding {
    /// The bytes used to separate lines for this style.
    pub fn as_str(self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}

/// The decoded result of reading a file: its encoding, line-ending style, and
/// the text content (with line endings normalized to `\n` for in-memory work).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedFile {
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    /// Content with all line endings normalized to `\n`.
    pub content: String,
}

/// Sniff the encoding from raw bytes. Recognizes the UTF-8 BOM; everything else
/// must be valid UTF-8 (otherwise `FILE_ENCODING_UNSUPPORTED`). No other
/// legacy encodings (GBK, Shift-JIS, Latin-1, …) are auto-detected — they are
/// rejected to avoid lossy round-trips.
pub fn sniff_encoding(bytes: &[u8]) -> Result<Encoding, SidecarError> {
    let encoding = if bytes.starts_with(UTF8_BOM) {
        Encoding::Utf8Bom
    } else {
        Encoding::Utf8
    };
    // Validate that the payload (after stripping an optional BOM) is valid UTF-8.
    let payload = if encoding.has_bom() {
        &bytes[3..]
    } else {
        bytes
    };
    if std::str::from_utf8(payload).is_err() {
        return Err(SidecarError::new(
            ErrorCode::FileEncodingUnsupported,
            "file is not valid UTF-8 (BOM-stripped). Other encodings are not auto-converted to avoid data loss.",
        ));
    }
    Ok(encoding)
}

/// Detect the dominant line-ending style of a string (content already
/// normalized to `\n` internally is *not* assumed here — this scans the raw
/// normalized text). If any `\r\n` is present, the file is treated as CRLF;
/// otherwise LF. A mixed file is normalized to CRLF on write if it contained
/// any CRLF, else LF — matching "preserve, don't silently change" semantics.
pub fn detect_line_ending(content: &str) -> LineEnding {
    if content.contains("\r\n") {
        LineEnding::Crlf
    } else {
        LineEnding::Lf
    }
}

/// Decode raw file bytes into a [`DecodedFile`]. Content is returned with line
/// endings normalized to `\n`; the original style is recorded in
/// `line_ending` so the writer can restore it.
pub fn decode(bytes: &[u8]) -> Result<DecodedFile, SidecarError> {
    let encoding = sniff_encoding(bytes)?;
    let payload = if encoding.has_bom() {
        &bytes[3..]
    } else {
        bytes
    };
    let raw = std::str::from_utf8(payload).map_err(|_| {
        SidecarError::new(
            ErrorCode::FileEncodingUnsupported,
            "file is not valid UTF-8 after BOM stripping",
        )
    })?;
    let line_ending = detect_line_ending(raw);
    // Normalize all line endings to \n for in-memory work.
    let content = normalize_to_lf(raw);
    Ok(DecodedFile {
        encoding,
        line_ending,
        content,
    })
}

/// Normalize a string's line endings to `\n` (from `\r\n` or lone `\r`).
pub fn normalize_to_lf(s: &str) -> String {
    // Replace CRLF first, then any lone CR (old Mac style).
    s.replace("\r\n", "\n").replace('\r', "\n")
}

/// Re-apply the detected line-ending style to normalized (`\n`) content.
pub fn apply_line_ending(content: &str, line_ending: LineEnding) -> String {
    match line_ending {
        LineEnding::Lf => content.to_string(),
        LineEnding::Crlf => content.replace('\n', "\r\n"),
    }
}

/// Encode in-memory content (line endings normalized to `\n`) back to file
/// bytes, restoring the original encoding (BOM if present) and line-ending
/// style. This is the inverse of [`decode`] and guarantees a round-trip for
/// files that were themselves UTF-8 with a single consistent line-ending style.
pub fn encode(content: &str, encoding: Encoding, line_ending: LineEnding) -> Vec<u8> {
    let normalized = normalize_to_lf(content);
    let with_endings = apply_line_ending(&normalized, line_ending);
    let mut bytes = Vec::with_capacity(with_endings.len() + 3);
    if encoding.has_bom() {
        bytes.extend_from_slice(UTF8_BOM);
    }
    bytes.extend_from_slice(with_endings.as_bytes());
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decoded_eq(decoded: &DecodedFile, content: &str) -> bool {
        decoded.content == content
    }

    #[test]
    fn decodes_utf8_without_bom_lf() {
        let bytes = b"line one\nline two\n";
        let d = decode(bytes).unwrap();
        assert_eq!(d.encoding, Encoding::Utf8);
        assert_eq!(d.line_ending, LineEnding::Lf);
        assert!(decoded_eq(&d, "line one\nline two\n"));
    }

    #[test]
    fn decodes_utf8_with_bom_crlf() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(UTF8_BOM);
        bytes.extend_from_slice(b"hello\r\nworld\r\n");
        let d = decode(&bytes).unwrap();
        assert_eq!(d.encoding, Encoding::Utf8Bom);
        assert_eq!(d.line_ending, LineEnding::Crlf);
        // content normalized to LF internally
        assert_eq!(d.content, "hello\nworld\n");
    }

    #[test]
    fn roundtrip_preserves_bom_and_crlf() {
        let original = {
            let mut v = Vec::new();
            v.extend_from_slice(UTF8_BOM);
            v.extend_from_slice(b"# Title\r\nbody\r\n");
            v
        };
        let d = decode(&original).unwrap();
        let reencoded = encode(&d.content, d.encoding, d.line_ending);
        assert_eq!(reencoded, original);
    }

    #[test]
    fn roundtrip_preserves_no_bom_and_lf() {
        let original = b"# Title\nbody\n";
        let d = decode(original).unwrap();
        let reencoded = encode(&d.content, d.encoding, d.line_ending);
        assert_eq!(reencoded.as_slice(), original.as_ref());
    }

    #[test]
    fn empty_file_roundtrips() {
        // Empty file: no BOM, LF, empty content.
        let d = decode(b"").unwrap();
        assert_eq!(d.encoding, Encoding::Utf8);
        assert_eq!(d.line_ending, LineEnding::Lf);
        assert_eq!(d.content, "");
        let reencoded = encode(&d.content, d.encoding, d.line_ending);
        assert!(reencoded.is_empty());
    }

    #[test]
    fn rejects_non_utf8_bytes() {
        // Invalid UTF-8 continuation (0xFF 0xFE is a BOM-ish but not UTF-8).
        let bytes = &[0xFF, 0xFE, b'x'];
        let err = decode(bytes).unwrap_err();
        assert_eq!(err.code, ErrorCode::FileEncodingUnsupported);
    }

    #[test]
    fn rejects_latin1_as_unsupported() {
        // 0xE9 is 'é' in Latin-1 but invalid as standalone UTF-8.
        let bytes = &[b'h', 0xE9, b'l', b'l', b'o'];
        let err = decode(bytes).unwrap_err();
        assert_eq!(err.code, ErrorCode::FileEncodingUnsupported);
    }

    #[test]
    fn detects_crlf_when_present() {
        // A file with mixed endings still classifies as CRLF (has any CRLF).
        let s = "a\nb\r\nc";
        assert_eq!(detect_line_ending(s), LineEnding::Crlf);
    }

    #[test]
    fn detects_lf_when_no_crlf() {
        let s = "a\nb\nc";
        assert_eq!(detect_line_ending(s), LineEnding::Lf);
    }

    #[test]
    fn normalize_handles_lone_cr() {
        assert_eq!(normalize_to_lf("a\rb\r\n"), "a\nb\n");
    }

    #[test]
    fn apply_crlf_then_normalize_is_identity() {
        let s = "# H\n中文内容\n";
        let crlf = apply_line_ending(s, LineEnding::Crlf);
        assert_eq!(normalize_to_lf(&crlf), s);
    }

    #[test]
    fn unicode_content_roundtrips() {
        // Chinese + emoji in the content body.
        let original = "你好世界 🌍\n第二行\n";
        let d = decode(original.as_bytes()).unwrap();
        let reencoded = encode(&d.content, d.encoding, d.line_ending);
        assert_eq!(reencoded.as_slice(), original.as_bytes());
    }
}
