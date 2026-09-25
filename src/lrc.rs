use std::fmt;

/// Current playback state as reported by the MPRIS player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    Stopped,
}

/// A single timed lyric line parsed from an LRC file.
#[derive(Debug, Clone)]
pub struct LyricLine {
    pub time: f64,
    pub text: String,
}

/// Full lyrics for a track: timed lines plus metadata.
#[derive(Debug, Clone, Default)]
pub struct Lyrics {
    pub lines: Vec<LyricLine>,
    /// Track title from LRC `[ti:...]` metadata (informational).
    #[allow(dead_code)]
    pub title: Option<String>,
    /// Artist from LRC `[ar:...]` metadata (informational).
    #[allow(dead_code)]
    pub artist: Option<String>,
}

impl Lyrics {
    /// Index of the line active at `t` seconds (the last line whose time <= t).
    pub fn line_at(&self, t: f64) -> Option<usize> {
        // binary search: first line with time > t, step back one
        let idx = self.lines.partition_point(|l| l.time <= t);
        idx.checked_sub(1)
    }
}

impl fmt::Display for PlaybackStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlaybackStatus::Playing => write!(f, "Playing"),
            PlaybackStatus::Paused => write!(f, "Paused"),
            PlaybackStatus::Stopped => write!(f, "Stopped"),
        }
    }
}

/// Parse one LRC document into timed lines.
///
/// Understands standard `[mm:ss.xx]` timestamps (1-3 fraction digits,
/// optional full or two-digit year in timestamp order), skips metadata
/// tags such as `[ti:...]`, `[ar:...]`, `[offset:...]` and comments.
/// Multiple timestamps on one line (`[00:01.00][00:05.00]text`) get
/// separate entries.
pub fn parse_lrc(input: &str) -> Lyrics {
    let mut lines: Vec<LyricLine> = Vec::new();
    let mut title = None;
    let mut artist = None;

    for raw in input.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Collect every leading [..] tag on this line.
        let mut times: Vec<f64> = Vec::new();
        let mut rest = line;
        while let Some(close) = rest.find(']') {
            let body = &rest[1..close];
            if let Some(t) = parse_timestamp(body) {
                times.push(t);
            } else if let Some((k, v)) = body.split_once(':') {
                match k.trim().to_ascii_lowercase().as_str() {
                    "ti" => title = Some(v.trim().to_string()),
                    "ar" => artist = Some(v.trim().to_string()),
                    _ => {}
                }
            }
            rest = &rest[close + 1..];
            if !rest.starts_with('[') {
                break;
            }
        }

        let text = rest.trim();
        for t in times {
            lines.push(LyricLine {
                time: t,
                text: text.to_string(),
            });
        }
    }

    lines.sort_by(|a, b| {
        a.time
            .partial_cmp(&b.time)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Lyrics {
        lines,
        title,
        artist,
    }
}

/// Parse `[mm:ss(.frac)?]` into seconds. Returns None for non-timestamps.
fn parse_timestamp(body: &str) -> Option<f64> {
    let (min, sec) = body.split_once(':')?;
    let min: f64 = min.trim().parse().ok()?;
    if !(0.0..=99.0).contains(&min) {
        return None;
    }
    let sec = sec.trim();
    let sec: f64 = sec.parse().ok()?;
    if !(0.0..=60.0).contains(&sec) {
        return None;
    }
    Some(min * 60.0 + sec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_lrc() {
        let lrc = parse_lrc("[00:01.00]hello\n[00:05.50] world \n[ti:Song]\n[ar:Someone]\n");
        assert_eq!(lrc.lines.len(), 2);
        assert_eq!(lrc.lines[0].time, 1.0);
        assert_eq!(lrc.lines[0].text, "hello");
        assert_eq!(lrc.lines[1].time, 5.5);
        assert_eq!(lrc.lines[1].text, "world");
        assert_eq!(lrc.title.as_deref(), Some("Song"));
        assert_eq!(lrc.artist.as_deref(), Some("Someone"));
    }

    #[test]
    fn multi_timestamp_line() {
        let lrc = parse_lrc("[00:10.00][01:20.50]chorus\n");
        assert_eq!(lrc.lines.len(), 2);
        assert_eq!(lrc.lines[1].time, 80.5);
    }

    #[test]
    fn line_at_current() {
        let lrc = parse_lrc("[00:01.00]one\n[00:05.00]two\n[00:09.00]three\n");
        assert_eq!(lrc.line_at(0.0), None);
        assert_eq!(lrc.line_at(1.0), Some(0));
        assert_eq!(lrc.line_at(6.2), Some(1));
        assert_eq!(lrc.line_at(999.0), Some(2));
    }

    #[test]
    fn ignores_plain_lines_and_junk() {
        let lrc = parse_lrc("no timestamps here\n[garbage tag]\n[offset:500]\n");
        assert!(lrc.lines.is_empty());
    }

    #[test]
    fn three_fraction_digits() {
        let lrc = parse_lrc("[00:01.123]x\n");
        assert!((lrc.lines[0].time - 1.123).abs() < 1e-9);
    }
}
