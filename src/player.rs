use mpris::{Metadata, PlaybackStatus as MprisStatus, Player, PlayerFinder};

use crate::lrc::PlaybackStatus;

/// A track snapshot taken from an MPRIS player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackInfo {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub length: Option<u64>, // microseconds
}

impl TrackInfo {
    pub fn from_metadata(md: &Metadata) -> Option<Self> {
        let title = md.title()?.trim().to_string();
        if title.is_empty() {
            return None;
        }
        let artist = md
            .artists()
            .map(|v| v.join(", "))
            .or_else(|| md.album_artists().map(|v| v.join(", ")))
            .unwrap_or_default();
        Some(Self {
            title,
            artist,
            album: md.album_name().map(str::to_string),
            length: md.length().map(|d| d.as_micros() as u64),
        })
    }

    /// Stable key used for the lyrics cache: "artist - title" lowercased.
    pub fn cache_key(&self) -> String {
        let key = if self.artist.is_empty() {
            self.title.clone()
        } else {
            format!("{} - {}", self.artist, self.title)
        };
        key.trim().to_lowercase()
    }
}

/// A currently active MPRIS player and its current track.
#[derive(Debug, Clone)]
pub struct ActivePlayer {
    pub name: String,
    pub track: TrackInfo,
}

/// Detect the "most relevant" MPRIS player, mirroring playerctl logic:
/// prefer one that is Playing, else one that is Paused, else any with a
/// track. Browsers (Chrome/Firefox expose per-tab MPRIS) and terminal
/// players (ytmgo, mpv, spotify, vlc...) are all MPRIS, so everything is
/// picked up the same way with zero configuration.
///
/// Broken/half-dead players on the bus are skipped instead of failing the
/// whole scan (find_all() would abort on the first bad player).
pub fn find_active_player() -> Option<ActivePlayer> {
    let finder = PlayerFinder::new().ok()?;

    let mut best: Option<(Player, i32)> = None;
    for p in finder.iter_players().ok()?.flatten() {
        let Ok(md) = p.get_metadata() else { continue };
        if TrackInfo::from_metadata(&md).is_none() {
            continue; // no usable track in this player
        }
        // score: playing=3, paused=2, has-track=1
        let score = match p.get_playback_status() {
            Ok(MprisStatus::Playing) => 3,
            Ok(MprisStatus::Paused) => 2,
            _ => 1,
        };
        if best.as_ref().is_none_or(|(_, s)| score > *s) {
            best = Some((p, score));
        }
    }

    let (p, _) = best?;
    let md = p.get_metadata().ok()?;
    let track = TrackInfo::from_metadata(&md)?;
    Some(ActivePlayer {
        name: p.bus_name_trimmed().to_string(),
        track,
    })
}

/// Resolve a player by its trimmed bus name (or identity), freshly.
/// Errors on individual players are skipped, not fatal.
fn resolve(bus_name: &str) -> Option<Player> {
    let finder = PlayerFinder::new().ok()?;
    finder
        .iter_players()
        .ok()?
        .flatten()
        .find(|p| p.bus_name_trimmed() == bus_name || p.identity() == bus_name)
}

/// Read the current position (seconds) from `player` by its bus name.
pub fn position_of(bus_name: &str) -> Option<f64> {
    resolve(bus_name)?
        .get_position()
        .ok()
        .map(|d| d.as_secs_f64())
}

/// Read the current status for a player by bus name.
pub fn status_of(bus_name: &str) -> Option<PlaybackStatus> {
    match resolve(bus_name)?.get_playback_status() {
        Ok(MprisStatus::Playing) => Some(PlaybackStatus::Playing),
        Ok(MprisStatus::Paused) => Some(PlaybackStatus::Paused),
        Ok(MprisStatus::Stopped) => Some(PlaybackStatus::Stopped),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_normalizes() {
        let t = TrackInfo {
            title: "Song X".into(),
            artist: "Artist".into(),
            album: None,
            length: None,
        };
        assert_eq!(t.cache_key(), "artist - song x");
        let t2 = TrackInfo {
            title: "Solo".into(),
            artist: String::new(),
            album: None,
            length: None,
        };
        assert_eq!(t2.cache_key(), "solo");
    }
}
