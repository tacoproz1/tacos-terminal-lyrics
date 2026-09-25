//! ytmgo fallback backend.
//!
//! ytmgo plays through an mpv without MPRIS, but it keeps everything we
//! need elsewhere:
//!   - current queue + track title/artist/duration: SQLite (ytmgo.db)
//!   - playback position + pause state: mpv's JSON IPC socket
//!     (/run/user/<uid>/ytmgo-mpv-*.sock)
//!   - synced lyrics: its own lyrics_cache table, already LRC
//!
//! We open the database read-only so ytmgo keeps full ownership.

use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

use rusqlite::OpenFlags;

use crate::player::TrackInfo;

fn db_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".local/share")
        });
    let p = base.join("ytmgo").join("ytmgo.db");
    p.is_file().then_some(p)
}

/// Newest ytmgo mpv IPC socket.
fn mpv_socket() -> Option<PathBuf> {
    let uid = unsafe { libc::getuid() };
    let dir = PathBuf::from("/run/user").join(uid.to_string());
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for e in std::fs::read_dir(&dir).ok()?.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with("ytmgo-mpv-") && name.ends_with(".sock") {
            if let Ok(mtime) = e.metadata().and_then(|m| m.modified()) {
                if best.as_ref().is_none_or(|(t, _)| mtime > *t) {
                    best = Some((mtime, e.path()));
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

fn mpv_property(sock: &PathBuf, prop: &str) -> Option<serde_json::Value> {
    use std::io::{BufRead, BufReader, Write};
    let mut stream = UnixStream::connect(sock).ok()?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    let req = format!("{{\"command\":[\"get_property\",\"{prop}\"]}}\n");
    stream.write_all(req.as_bytes()).ok()?;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    for _ in 0..5 {
        line.clear();
        reader.read_line(&mut line).ok()?;
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
            if v.get("error").and_then(|e| e.as_str()) == Some("success") {
                return v.get("data").cloned();
            }
        }
    }
    None
}

/// Is ytmgo running with a playable mpv right now?
pub fn is_running() -> bool {
    mpv_socket().is_some()
}

/// Current track from ytmgo's queue table.
pub fn current_track() -> Option<TrackInfo> {
    let db = db_path()?;
    let conn = rusqlite::Connection::open_with_flags(
        &db,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()?;
    // read-only + immutable-ish access; ytmgo owns the WAL
    let sql = "SELECT t.value FROM queue_state s, json_each(s.tracks) t \
               WHERE s.id = (SELECT MAX(id) FROM queue_state) \
               AND t.key = (SELECT current_idx FROM queue_state \
                            WHERE id = (SELECT MAX(id) FROM queue_state))";
    let raw: String = conn.query_row(sql, [], |r| r.get(0)).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let title = v.get("title").and_then(|x| x.as_str())?.to_string();
    let artist = v
        .get("artist")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let album = v.get("album").and_then(|x| x.as_str()).map(str::to_string);
    let length = v
        .get("duration_sec")
        .and_then(|x| x.as_f64())
        .map(|s| (s * 1_000_000.0) as u64);
    Some(TrackInfo {
        title,
        artist,
        album,
        length,
    })
}

/// Playback position in seconds from the mpv IPC socket.
pub fn position_secs() -> Option<f64> {
    let sock = mpv_socket()?;
    mpv_property(&sock, "playback-time")?.as_f64()
}

/// Pause state from the mpv IPC socket (true = paused).
pub fn is_paused() -> Option<bool> {
    let sock = mpv_socket()?;
    mpv_property(&sock, "pause")?.as_bool()
}

/// ytmgo's own synced-lyrics cache for the current track id, if present.
pub fn cached_lyrics() -> Option<String> {
    let db = db_path()?;
    let conn = rusqlite::Connection::open_with_flags(
        &db,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .ok()?;
    let sql = "SELECT l.lyrics FROM queue_state s, json_each(s.tracks) t, lyrics_cache l \
               WHERE s.id = (SELECT MAX(id) FROM queue_state) \
               AND t.key = (SELECT current_idx FROM queue_state \
                            WHERE id = (SELECT MAX(id) FROM queue_state)) \
               AND l.track_id = json_extract(t.value, '$.id') \
               AND l.synced = 1 AND length(l.lyrics) > 0";
    let raw: String = conn.query_row(sql, [], |r| r.get(0)).ok()?;
    (!raw.trim().is_empty()).then_some(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    // These hit the real environment (may or may not have ytmgo running),
    // so they only assert no panics and consistent types.
    #[test]
    fn track_shape_if_present() {
        if let Some(t) = current_track() {
            assert!(!t.title.is_empty(), "title must not be empty");
        }
    }

    #[test]
    fn lyrics_parseable_if_present() {
        if let Some(lrc) = cached_lyrics() {
            let parsed = crate::lrc::parse_lrc(&lrc);
            assert!(
                !parsed.lines.is_empty(),
                "cached lyrics should parse to lines"
            );
        }
    }
}
