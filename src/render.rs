//! Pure rendering: block/compact fonts ported from the original project,
//! layout, and karaoke word timing. No terminal I/O here.

// ---------------------------------------------------------------------------
// Fonts (ported from the Python original)
// ---------------------------------------------------------------------------

const BLOCK_A: &[&str] = &["  ███  ", " ██ ██ ", "███████", "██   ██", "██   ██"];
const BLOCK_B: &[&str] = &["██████ ", "██   ██", "██████ ", "██   ██", "██████ "];
const BLOCK_C: &[&str] = &[" █████ ", "██   ██", "██     ", "██   ██", " █████ "];
const BLOCK_D: &[&str] = &["██████ ", "██   ██", "██   ██", "██   ██", "██████ "];
const BLOCK_E: &[&str] = &["███████", "██     ", "█████  ", "██     ", "███████"];
const BLOCK_F: &[&str] = &["███████", "██     ", "█████  ", "██     ", "██     "];
const BLOCK_G: &[&str] = &[" █████ ", "██     ", "██  ███", "██   ██", " █████ "];
const BLOCK_H: &[&str] = &["██   ██", "██   ██", "███████", "██   ██", "██   ██"];
const BLOCK_I: &[&str] = &["███", " ██", " ██", " ██", "███"];
const BLOCK_J: &[&str] = &["     ██", "     ██", "     ██", "██   ██", " █████ "];
const BLOCK_K: &[&str] = &["██   ██", "██  ██ ", "█████  ", "██  ██ ", "██   ██"];
const BLOCK_L: &[&str] = &["██     ", "██     ", "██     ", "██     ", "███████"];
const BLOCK_M: &[&str] = &["██   ██", "███ ███", "███████", "██ █ ██", "██   ██"];
const BLOCK_N: &[&str] = &["██   ██", "███  ██", "████ ██", "██ ████", "██   ██"];
const BLOCK_O: &[&str] = &[" █████ ", "██   ██", "██   ██", "██   ██", " █████ "];
const BLOCK_P: &[&str] = &["██████ ", "██   ██", "██████ ", "██     ", "██     "];
const BLOCK_Q: &[&str] = &[" █████ ", "██   ██", "██   ██", "██  ███", " ██████"];
const BLOCK_R: &[&str] = &["██████ ", "██   ██", "██████ ", "██  ██ ", "██   ██"];
const BLOCK_S: &[&str] = &[" █████ ", "██     ", " █████ ", "     ██", " █████ "];
const BLOCK_T: &[&str] = &["███████", "  ██   ", "  ██   ", "  ██   ", "  ██   "];
const BLOCK_U: &[&str] = &["██   ██", "██   ██", "██   ██", "██   ██", " █████ "];
const BLOCK_V: &[&str] = &["██   ██", "██   ██", "██   ██", " ██ ██ ", "  ███  "];
const BLOCK_W: &[&str] = &["██   ██", "██   ██", "██ █ ██", "███████", "███ ███"];
const BLOCK_X: &[&str] = &["██   ██", " ██ ██ ", "  ███  ", " ██ ██ ", "██   ██"];
const BLOCK_Y: &[&str] = &["██   ██", " ██ ██ ", "  ███  ", "  ██   ", "  ██   "];
const BLOCK_Z: &[&str] = &["███████", "    ██ ", "  ███  ", " ██    ", "███████"];
const BLOCK_SPACE: &[&str] = &["    ", "    ", "    ", "    ", "    "];
const BLOCK_0: &[&str] = &[" █████ ", "██   ██", "██   ██", "██   ██", " █████ "];
const BLOCK_1: &[&str] = &["  ██   ", " ███   ", "  ██   ", "  ██   ", "███████"];
const BLOCK_2: &[&str] = &[" █████ ", "██   ██", "   ███ ", " ██    ", "███████"];
const BLOCK_3: &[&str] = &[" █████ ", "██   ██", "  ████ ", "██   ██", " █████ "];
const BLOCK_4: &[&str] = &["██   ██", "██   ██", "███████", "     ██", "     ██"];
const BLOCK_5: &[&str] = &["███████", "██     ", "██████ ", "     ██", "██████ "];
const BLOCK_6: &[&str] = &[" █████ ", "██     ", "██████ ", "██   ██", " █████ "];
const BLOCK_7: &[&str] = &["███████", "     ██", "    ██ ", "   ██  ", "  ██   "];
const BLOCK_8: &[&str] = &[" █████ ", "██   ██", " █████ ", "██   ██", " █████ "];
const BLOCK_9: &[&str] = &[" █████ ", "██   ██", " ██████", "     ██", " █████ "];
const BLOCK_BRACKET_O: &[&str] = &["███", "██ ", "██ ", "██ ", "███"];
const BLOCK_BRACKET_C: &[&str] = &["███", " ██", " ██", " ██", "███"];
const BLOCK_APOS: &[&str] = &["██", "██", "  ", "  ", "  "];
const BLOCK_COMMA: &[&str] = &["  ", "  ", "  ", "██", "█ "];
const BLOCK_DOT: &[&str] = &["  ", "  ", "  ", "  ", "██"];
const BLOCK_EXCL: &[&str] = &["██", "██", "██", "  ", "██"];
const BLOCK_QUES: &[&str] = &[" ███ ", "█   █", "   █ ", "     ", "  █  "];
const BLOCK_DASH: &[&str] = &["      ", "      ", "██████", "      ", "      "];
const BLOCK_PAR_O: &[&str] = &[" ██", "██ ", "██ ", "██ ", " ██"];
const BLOCK_PAR_C: &[&str] = &["██ ", " ██", " ██", " ██", "██ "];
const BLOCK_COLON: &[&str] = &["  ", "██", "  ", "██", "  "];
const BLOCK_SEMI: &[&str] = &["  ", "██", "  ", "██", "█ "];
const BLOCK_SLASH: &[&str] = &["    ██", "   ██ ", "  ██  ", " ██   ", "██    "];
const BLOCK_BACKSLASH: &[&str] = &["██    ", " ██   ", "  ██  ", "   ██ ", "    ██"];
const BLOCK_QUOTE: &[&str] = &["██ ██", "██ ██", "     ", "     ", "     "];
const BLOCK_AMP: &[&str] = &[" ███  ", "█   █ ", " ███  ", "█ █ █ ", " ███ █"];
const BLOCK_HASH: &[&str] = &[" █ █ ", "█████", " █ █ ", "█████", " █ █ "];
const BLOCK_STAR: &[&str] = &["█ █ █", " ███ ", "█████", " ███ ", "█ █ █"];
const BLOCK_PLUS: &[&str] = &["  ██  ", "  ██  ", "██████", "  ██  ", "  ██  "];
const BLOCK_EQ: &[&str] = &["      ", "██████", "      ", "██████", "      "];
const BLOCK_LT: &[&str] = &["   ██ ", "  ██  ", " ██   ", "  ██  ", "   ██ "];
const BLOCK_GT: &[&str] = &[" ██   ", "  ██  ", "   ██ ", "  ██  ", " ██   "];
const BLOCK_AT: &[&str] = &[" ████ ", "█    █", "█ ██ █", "█ ██ █", " ████ "];
const BLOCK_BULLET: &[&str] = &["      ", "  ██  ", "  ██  ", "      ", "      "];

fn block_glyph(c: char) -> Option<&'static [&'static str]> {
    // Accented letters render as their base letter (no font space for
    // every variant, and lyrics are full of them in Spanish etc.)
    let c = match c {
        'Á' => 'A',
        'À' => 'A',
        'Ä' => 'A',
        'Â' => 'A',
        'Ã' => 'A',
        'Å' => 'A',
        'É' => 'E',
        'È' => 'E',
        'Ë' => 'E',
        'Ê' => 'E',
        'Í' => 'I',
        'Ì' => 'I',
        'Ï' => 'I',
        'Î' => 'I',
        'Ó' => 'O',
        'Ò' => 'O',
        'Ö' => 'O',
        'Ô' => 'O',
        'Õ' => 'O',
        'Ú' => 'U',
        'Ù' => 'U',
        'Ü' => 'U',
        'Û' => 'U',
        'Ñ' => 'N',
        'Ç' => 'C',
        'Ý' => 'Y',
        '¡' => '!',
        '¿' => '?',
        _ => c,
    };
    Some(match c {
        'A' => BLOCK_A,
        'B' => BLOCK_B,
        'C' => BLOCK_C,
        'D' => BLOCK_D,
        'E' => BLOCK_E,
        'F' => BLOCK_F,
        'G' => BLOCK_G,
        'H' => BLOCK_H,
        'I' => BLOCK_I,
        'J' => BLOCK_J,
        'K' => BLOCK_K,
        'L' => BLOCK_L,
        'M' => BLOCK_M,
        'N' => BLOCK_N,
        'O' => BLOCK_O,
        'P' => BLOCK_P,
        'Q' => BLOCK_Q,
        'R' => BLOCK_R,
        'S' => BLOCK_S,
        'T' => BLOCK_T,
        'U' => BLOCK_U,
        'V' => BLOCK_V,
        'W' => BLOCK_W,
        'X' => BLOCK_X,
        'Y' => BLOCK_Y,
        'Z' => BLOCK_Z,
        ' ' => BLOCK_SPACE,
        '0' => BLOCK_0,
        '1' => BLOCK_1,
        '2' => BLOCK_2,
        '3' => BLOCK_3,
        '4' => BLOCK_4,
        '5' => BLOCK_5,
        '6' => BLOCK_6,
        '7' => BLOCK_7,
        '8' => BLOCK_8,
        '9' => BLOCK_9,
        '[' => BLOCK_BRACKET_O,
        ']' => BLOCK_BRACKET_C,
        '\'' => BLOCK_APOS,
        ',' => BLOCK_COMMA,
        '.' => BLOCK_DOT,
        '!' => BLOCK_EXCL,
        '?' => BLOCK_QUES,
        '-' => BLOCK_DASH,
        '(' => BLOCK_PAR_O,
        ')' => BLOCK_PAR_C,
        ':' => BLOCK_COLON,
        ';' => BLOCK_SEMI,
        '/' => BLOCK_SLASH,
        '\\' => BLOCK_BACKSLASH,
        '"' => BLOCK_QUOTE,
        '&' => BLOCK_AMP,
        '#' => BLOCK_HASH,
        '*' => BLOCK_STAR,
        '+' => BLOCK_PLUS,
        '=' => BLOCK_EQ,
        '<' => BLOCK_LT,
        '>' => BLOCK_GT,
        '@' => BLOCK_AT,
        '•' => BLOCK_BULLET,
        _ => return None,
    })
}

const COMPACT_A: &[&str] = &["▄▀▄", "█▀█", "▀ ▀"];
const COMPACT_B: &[&str] = &["█▀▄", "█▀▄", "▀▀ "];
const COMPACT_C: &[&str] = &["▄▀▀", "█  ", "▀▀▀"];
const COMPACT_D: &[&str] = &["█▀▄", "█ █", "▀▀ "];
const COMPACT_E: &[&str] = &["█▀▀", "█▀ ", "▀▀▀"];
const COMPACT_F: &[&str] = &["█▀▀", "█▀ ", "▀  "];
const COMPACT_G: &[&str] = &["▄▀▀", "█ ▀", "▀▀▀"];
const COMPACT_H: &[&str] = &["█ █", "█▀█", "▀ ▀"];
const COMPACT_I: &[&str] = &["█", "█", "▀"];
const COMPACT_J: &[&str] = &[" █", " █", "▀ "];
const COMPACT_K: &[&str] = &["█ █", "█▀ ", "▀ ▀"];
const COMPACT_L: &[&str] = &["█  ", "█  ", "▀▀▀"];
const COMPACT_M: &[&str] = &["█▄▀▄█", "█ ▀ █", "▀   ▀"];
const COMPACT_N: &[&str] = &["█▄█", "█ █", "▀ ▀"];
const COMPACT_O: &[&str] = &["▄▀▀▄", "█  █", "▀▀▀ "];
const COMPACT_P: &[&str] = &["█▀▄", "█▀ ", "▀  "];
const COMPACT_Q: &[&str] = &["▄▀▀▄", "█ ▀█", "▀▀ ▀"];
const COMPACT_R: &[&str] = &["█▀▄", "█▀▄", "▀ ▀"];
const COMPACT_S: &[&str] = &["▄▀▀", " ▀▄", "▀▀ "];
const COMPACT_T: &[&str] = &["▀█▀", " █ ", " ▀ "];
const COMPACT_U: &[&str] = &["█ █", "█ █", "▀▀▀"];
const COMPACT_V: &[&str] = &["█ █", "█ █", " ▀ "];
const COMPACT_W: &[&str] = &["█   █", "█ ▄ █", "▀▀▀▀▀"];
const COMPACT_X: &[&str] = &["█ █", " ▀ ", "▀ ▀"];
const COMPACT_Y: &[&str] = &["█ █", " ▀ ", " ▀ "];
const COMPACT_Z: &[&str] = &["▀▀█", " █ ", "█▀▀"];
const COMPACT_SPACE: &[&str] = &["   ", "   ", "   "];

fn compact_glyph(c: char) -> Option<&'static [&'static str]> {
    Some(match c {
        'A' => COMPACT_A,
        'B' => COMPACT_B,
        'C' => COMPACT_C,
        'D' => COMPACT_D,
        'E' => COMPACT_E,
        'F' => COMPACT_F,
        'G' => COMPACT_G,
        'H' => COMPACT_H,
        'I' => COMPACT_I,
        'J' => COMPACT_J,
        'K' => COMPACT_K,
        'L' => COMPACT_L,
        'M' => COMPACT_M,
        'N' => COMPACT_N,
        'O' => COMPACT_O,
        'P' => COMPACT_P,
        'Q' => COMPACT_Q,
        'R' => COMPACT_R,
        'S' => COMPACT_S,
        'T' => COMPACT_T,
        'U' => COMPACT_U,
        'V' => COMPACT_V,
        'W' => COMPACT_W,
        'X' => COMPACT_X,
        'Y' => COMPACT_Y,
        'Z' => COMPACT_Z,
        ' ' => COMPACT_SPACE,
        _ => return None,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    Block,
    Compact,
}

impl Font {
    pub fn glyph(self, c: char) -> Option<&'static [&'static str]> {
        match self {
            Font::Block => block_glyph(c),
            Font::Compact => compact_glyph(c),
        }
    }

    fn height(self) -> usize {
        match self {
            Font::Block => 5,
            Font::Compact => 3,
        }
    }
}

/// Columns one char occupies in the big render: glyph width + 1 separator.
fn glyph_cols(c: char, font: Font) -> usize {
    font.glyph(c)
        .or_else(|| font.glyph(' '))
        .and_then(|g| g.first().map(|s| s.chars().count()))
        .unwrap_or(1)
        + 1
}

/// Render `text` with `font`, one plain string per font row.
/// Rows keep their full width (glyph + separator) so every row is the same
/// length and vertical alignment is exact when centering.
pub fn render_big(text: &str, font: Font) -> Vec<String> {
    let height = font.height();
    let mut rows: Vec<String> = vec![String::new(); height];
    for c in text.to_uppercase().chars() {
        let glyph = font.glyph(c).unwrap_or_else(|| font.glyph(' ').unwrap());
        for (i, row) in rows.iter_mut().enumerate() {
            if let Some(line) = glyph.get(i) {
                row.push_str(line);
            } else {
                let w = glyph.first().map(|l| l.chars().count()).unwrap_or(3);
                row.push_str(&" ".repeat(w));
            }
            row.push(' ');
        }
    }
    rows
}

/// Width (in terminal cells) of a rendered big line.
pub fn big_width(text: &str, font: Font) -> usize {
    let mut w = 0;
    for c in text.to_uppercase().chars() {
        w += glyph_cols(c, font);
    }
    w
}

/// Shorten a lyric line (front-truncating words) so its big render fits `cols`.
pub fn fit_big(text: &str, font: Font, cols: usize) -> String {
    if big_width(text, font) <= cols {
        return text.to_string();
    }
    // drop leading words until it fits (keep the part being sung)
    let words: Vec<&str> = text.split_whitespace().collect();
    for start in 0..words.len() {
        let candidate = words[start..].join(" ");
        if big_width(&candidate, font) <= cols {
            return candidate;
        }
    }
    let single = words.last().copied().unwrap_or("");
    if big_width(single, font) <= cols {
        return single.to_string();
    }
    "...".to_string()
}

/// Byte ranges (start, len) of the words in a line, whitespace between them.
pub fn words_of(line: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut chars = line.char_indices().peekable();
    while let Some(&(i, c)) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        let start = i;
        let mut end = i;
        while let Some(&(j, c)) = chars.peek() {
            if c.is_whitespace() {
                break;
            }
            end = j + c.len_utf8();
            chars.next();
        }
        out.push((start, end - start));
    }
    out
}

/// Column spans of each word in the big render: [(start_col, end_col_exclusive)].
/// Same word grouping/order as `words_of` / `word_times`.
pub fn word_cols(line: &str, font: Font) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut col = 0usize;
    let mut cur: Option<(usize, usize)> = None;
    for c in line.to_uppercase().chars() {
        let w = glyph_cols(c, font);
        if c.is_whitespace() {
            if let Some(span) = cur.take() {
                out.push(span);
            }
        } else {
            let start = cur.map(|(s, _)| s).unwrap_or(col);
            cur = Some((start, col + w));
        }
        col += w;
    }
    if let Some(span) = cur.take() {
        out.push(span);
    }
    out
}

/// Even-distribute word start times across the line's [start, end) window.
/// Mirrors the original's wlrc even-distribution mode, but computed on the
/// fly so no preprocessing step is needed.
pub fn word_times(line: &str, start: f64, end: f64) -> Vec<(f64, String)> {
    let words = words_of(line);
    let n = words.len();
    if n == 0 {
        return Vec::new();
    }
    let dur = (end - start).max(0.5);
    let step = dur / n as f64;
    words
        .into_iter()
        .enumerate()
        .map(|(i, (s, l))| {
            let text = line[s..s + l].to_string();
            (start + step * i as f64, text)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_render_shape() {
        let rows = render_big("AB", Font::Block);
        assert_eq!(rows.len(), 5);
        // A(7)+sep(1)+B(7) = 15, plus B's trailing separator = 16
        assert_eq!(rows[0].chars().count(), 16);
        assert!(rows[0].contains("██"));
        // every row is the same width -> vertical alignment is exact
        assert!(rows.iter().all(|r| r.chars().count() == 16));
    }

    #[test]
    fn compact_render_shape() {
        let rows = render_big("A", Font::Compact);
        assert_eq!(rows.len(), 3);
        // glyph + trailing separator space (rows keep uniform width)
        assert_eq!(rows[0], "▄▀▄ ");
    }

    #[test]
    fn unknown_char_is_space() {
        let rows = render_big("Ω", Font::Block);
        assert!(rows.iter().all(|r| r.trim().is_empty()));
    }

    #[test]
    fn width_matches_render() {
        for (t, f) in [("HELLO", Font::Block), ("HEY", Font::Compact)] {
            assert_eq!(big_width(t, f), render_big(t, f)[0].chars().count());
        }
    }

    #[test]
    fn fit_truncates_words() {
        let long = "one two three four five six seven eight";
        let fit = fit_big(long, Font::Block, 40);
        assert!(big_width(&fit, Font::Block) <= 40);
        assert!(long.contains(fit.trim()) || fit == "...");
    }

    #[test]
    fn words_split() {
        let v = word_times("la la la", 10.0, 13.0);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0].1, "la");
        assert!((v[0].0 - 10.0).abs() < 1e-9);
        assert!((v[1].0 - 11.0).abs() < 1e-9);
        assert!((v[2].0 - 12.0).abs() < 1e-9);
    }

    #[test]
    fn words_with_punct() {
        let v = word_times("don't stop!", 0.0, 2.0);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].1, "don't");
        assert_eq!(v[1].1, "stop!");
    }

    #[test]
    fn word_cols_align_with_words() {
        let line = "la la";
        let wc = word_cols(line, Font::Block);
        assert_eq!(wc.len(), 2);
        // block glyphs are 7 cols wide + 1 separator: L(8) + A(8) = 16
        assert_eq!(wc[0], (0, 16));
        // space glyph 4+1=5, so second word starts at 16+5=21
        assert_eq!(wc[1].0, 21);
        assert_eq!(wc[1].1, 21 + 16);
    }

    #[test]
    fn word_cols_count_matches_word_times() {
        let line = "one two three";
        assert_eq!(
            word_cols(line, Font::Block).len(),
            word_times(line, 0.0, 9.0).len()
        );
        let wc = word_cols(line, Font::Block);
        assert!(wc.windows(2).all(|w| w[0].1 <= w[1].0));
        // last word's end == total render width (incl. trailing separator)
        assert_eq!(wc[2].1, big_width(line, Font::Block));
    }
}
