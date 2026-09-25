use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use crate::lrc::{parse_lrc, Lyrics};
use crate::player::TrackInfo;

const LRCLIB_API: &str = "https://lrclib.net/api";
const USER_AGENT: &str = concat!(
    "tacos-terminal-lyrics-rust/",
    env!("CARGO_PKG_VERSION"),
    " (github.com/mikuri12/tacos-terminal-lyrics-rust)"
);

/// Lyrics store: local disk cache first, lrclib.net as the source.
pub struct LyricsFinder {
    cache_dir: PathBuf,
    agent: ureq::Agent,
}

impl LyricsFinder {
    pub fn new() -> Self {
        let cache_dir = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache")
            })
            .join("tacos-lyrics");
        let _ = fs::create_dir_all(&cache_dir);
        Self {
            cache_dir,
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(15))
                .user_agent(USER_AGENT)
                .build(),
        }
    }

    /// Get lyrics for a track: cache hit, else fetch from lrclib and store.
    /// Returns None when the song is unknown to lrclib (result cached too,
    /// as an empty file, so we don't re-query every loop for the same song).
    pub fn get(&self, track: &TrackInfo) -> Option<Lyrics> {
        let path = self.cache_path(track);
        if let Ok(text) = fs::read_to_string(&path) {
            return Some(parse_lrc(&text)).filter(|l| !l.lines.is_empty());
        }
        let fetched = self.fetch(track);
        // Persist (even empty results, as negative cache; empty file == known miss)
        let _ = fs::write(&path, fetched.as_deref().unwrap_or(""));
        fetched
            .map(|text| parse_lrc(&text))
            .filter(|l| !l.lines.is_empty())
    }

    fn cache_path(&self, track: &TrackInfo) -> PathBuf {
        let mut key = track.cache_key();
        key.retain(|c| c.is_alphanumeric() || c == ' ' || c == '-');
        while key.len() > 80 {
            key.pop();
        }
        let key = key.trim();
        if key.is_empty() {
            self.cache_dir.join("_.lrc")
        } else {
            self.cache_dir.join(format!("{key}.lrc"))
        }
    }

    /// Query lrclib with progressively looser search strategies.
    fn fetch(&self, track: &TrackInfo) -> Option<String> {
        let duration = track.length.map(|us| us / 1_000_000);

        // 1. Exact /get with full metadata (best match, honors duration)
        if let Some(text) = self.api_get(track, duration) {
            return Some(text);
        }

        // 2. Search with cleaned title, pick first synced result
        let clean = clean_title(&track.title);
        if clean != track.title {
            if let Some(text) = self.api_search(&track.artist, &clean, duration) {
                return Some(text);
            }
        }

        // 3. Plain search with raw metadata
        if let Some(text) = self.api_search(&track.artist, &track.title, duration) {
            return Some(text);
        }

        // 4. No artist (e.g. browser only gives a title): query string search
        if track.artist.is_empty() {
            if let Some(text) = self.api_query(&track.title, duration) {
                return Some(text);
            }
        }

        None
    }

    fn api_get(&self, track: &TrackInfo, duration: Option<u64>) -> Option<String> {
        let mut url = format!(
            "{LRCLIB_API}/get?track_name={}&artist_name={}",
            url_encode(&track.title),
            url_encode(&track.artist),
        );
        if let Some(album) = &track.album {
            if !album.is_empty() {
                url.push_str(&format!("&album_name={}", url_encode(album)));
            }
        }
        if let Some(d) = duration {
            url.push_str(&format!("&duration={d}"));
        }
        let json = self.get_json(&url)?;
        synced_or_plain(&json)
    }

    fn api_search(&self, artist: &str, title: &str, duration: Option<u64>) -> Option<String> {
        let mut url = format!(
            "{LRCLIB_API}/search?track_name={}&artist_name={}",
            url_encode(title),
            url_encode(artist),
        );
        if let Some(d) = duration {
            url.push_str(&format!("&duration={d}"));
        }
        let json = self.get_json(&url)?;
        let arr = json.as_array()?;
        for entry in arr {
            if let Some(text) = synced_or_plain(entry) {
                return Some(text);
            }
        }
        None
    }

    fn api_query(&self, q: &str, duration: Option<u64>) -> Option<String> {
        let mut url = format!("{LRCLIB_API}/search?q={}", url_encode(q));
        if let Some(d) = duration {
            url.push_str(&format!("&duration={d}"));
        }
        let json = self.get_json(&url)?;
        let arr = json.as_array()?;
        for entry in arr {
            if let Some(text) = synced_or_plain(entry) {
                return Some(text);
            }
        }
        None
    }

    fn get_json(&self, url: &str) -> Option<serde_json::Value> {
        let resp = self.agent.get(url).call().ok()?;
        if resp.status() != 200 {
            return None;
        }
        resp.into_json::<serde_json::Value>().ok()
    }
}

impl Default for LyricsFinder {
    fn default() -> Self {
        Self::new()
    }
}

fn synced_or_plain(entry: &serde_json::Value) -> Option<String> {
    let synced = entry.get("syncedLyrics").and_then(|v| v.as_str());
    if let Some(s) = synced {
        if !s.trim().is_empty() {
            return Some(s.to_string());
        }
    }
    None // plain (unsynced) lyrics are useless for a synced visualizer
}

/// Strip "(Official Video)", "- Remastered", "feat. ..." etc. from titles.
fn clean_title(raw: &str) -> String {
    const NOISE: &[&str] = &[
        "(Official Video)",
        "(Official Music Video)",
        "(Official Audio)",
        "(Official Lyric Video)",
        "(Lyric Video)",
        "(Music Video)",
        "(Official Visualizer)",
        "(Visualizer)",
        "(Audio)",
        "(Video)",
        "- Remastered",
        "- Radio Edit",
        "- Single Version",
        "- Album Version",
        "- Extended Version",
        "- Nightcore",
        "- Slowed",
        "- Slowed Down",
        "- Sped Up",
    ];
    let mut title = raw.to_string();
    for pat in NOISE {
        let lower = title.to_lowercase();
        if lower.contains(&pat.to_lowercase()) {
            if let Some(pos) = lower.find(&pat.to_lowercase()) {
                title.truncate(pos);
                title = title.trim_end_matches(['-', ' ', ')']).to_string();
            }
        }
    }
    for marker in [" (feat.", " (ft.", " [feat.", " (with "] {
        if let Some(pos) = title.to_lowercase().find(marker) {
            title.truncate(pos);
        }
    }
    title.trim().to_string()
}

fn url_encode(s: &str) -> String {
    // Encode everything that is not an unreserved URL char.
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_noisy_titles() {
        assert_eq!(clean_title("Song (Official Video)"), "Song");
        assert_eq!(clean_title("Song - Remastered 2011"), "Song");
        assert_eq!(clean_title("Song (feat. Someone)"), "Song");
        assert_eq!(clean_title("Song"), "Song");
    }

    #[test]
    fn encodes_urls() {
        assert_eq!(url_encode("a b&c"), "a%20b%26c");
        assert_eq!(url_encode("möp"), "m%C3%B6p");
    }

    #[test]
    fn picks_synced_over_plain() {
        let v: serde_json::Value =
            serde_json::from_str(r#"{"syncedLyrics":"[00:01.00]hi","plainLyrics":"hi"}"#).unwrap();
        assert_eq!(synced_or_plain(&v).as_deref(), Some("[00:01.00]hi"));
        let p: serde_json::Value = serde_json::from_str(r#"{"plainLyrics":"hi"}"#).unwrap();
        assert!(synced_or_plain(&p).is_none());
    }
}
