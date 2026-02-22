"""
Lyrics fetching utilities for lrc-tools
Handles metadata extraction, LRCLIB API search, and syncedlyrics fallback
"""
import json
import logging
import time
from pathlib import Path
from typing import Optional, Dict, List, Tuple
from urllib import parse, request
from urllib.error import HTTPError, URLError

# Suppress syncedlyrics verbose logging
logging.getLogger('syncedlyrics').setLevel(logging.CRITICAL)

try:
    from mutagen.flac import FLAC
    from mutagen.mp3 import MP3
    from mutagen.mp4 import MP4
    from mutagen.oggvorbis import OggVorbis
    from mutagen.oggopus import OggOpus
    MUTAGEN_AVAILABLE = True
except ImportError:
    MUTAGEN_AVAILABLE = False

try:
    from syncedlyrics import search as syncedlyrics_search
    SYNCEDLYRICS_AVAILABLE = True
except ImportError:
    SYNCEDLYRICS_AVAILABLE = False

# Version/remix patterns to strip from titles
_VERSION_PATTERNS = [
    ' - Nightcore', ' - Slowed', ' - slowed', ' - Sped Up', ' - SPED UP',
    ' - Slowed Down', ' - Slowed & Reverb', ' - Slowed & Reverbed',
    ' - sped up', ' - speed up', ' - spedup', ' - super slowed',
    ' - super spedup', ' - ultra slowed', ' - Instrumental',
    ' - Radio Edit', ' - Extended', ' - extended',
    ' - Single Version', ' - Album Version', ' - Remaster', ' - Remastered',
    ' - Official Audio', ' - Official Video', ' - Lyrics', ' - lyric video',
    ' - Audio', ' - Video', ' - Visualizer', ' - Music Video',
    ' - NIGHTCORE', ' - Hardstyle Slowed', ' - original', ' - old mix',
    ' - v2', ' - v3', ' - Mega Mix', ' - Mega Mix Slowed', ' - VIRAL SLOWED',
    ' - 2009 N3WGR0UNDZ V3RS10N!',
]

_FEAT_MARKERS = [' (feat.', ' (ft.', ' (From ', ' (w ', ' (with ', ' (feat ', ' (ft ']
_BONUS_MARKERS = [' (Bonus)', ' (Bonus Track)', ' (BONUS TRACK)']
_TRAILING_ARTIFACTS = [' 3', ' -3', ' _.', ' -.', ' -)', ' -c', ' 33']
_QUALITY_TAGS = ['[FLAC]', '[MP3]', '[320]', '[256]', '[128]']


# ---------------------------------------------------------------------------
# Metadata extraction
# ---------------------------------------------------------------------------

def get_audio_metadata(filepath: Path) -> Optional[Dict[str, str]]:
    """
    Read artist and title from embedded audio tags via mutagen.
    Returns None if mutagen is unavailable or tags are missing.
    """
    if not MUTAGEN_AVAILABLE:
        return None

    try:
        ext = filepath.suffix.lower()

        if ext == '.flac':
            audio = FLAC(filepath)
        elif ext == '.mp3':
            audio = MP3(filepath)
        elif ext in ('.m4a', '.mp4'):
            audio = MP4(filepath)
        elif ext in ('.ogg', '.oga'):
            audio = OggVorbis(filepath)
        elif ext == '.opus':
            audio = OggOpus(filepath)
        else:
            return None

        if ext in ('.m4a', '.mp4'):
            # MP4 atoms use different keys
            artist = audio.get('\xa9ART', [None])[0]
            title = audio.get('\xa9nam', [None])[0]
        else:
            artist_tag = audio.get('artist') or audio.get('ARTIST')
            title_tag = audio.get('title') or audio.get('TITLE')
            artist = (artist_tag[0] if isinstance(artist_tag, list) else artist_tag) if artist_tag else None
            title = (title_tag[0] if isinstance(title_tag, list) else title_tag) if title_tag else None

        if artist and title:
            return {'artist': str(artist), 'title': str(title)}

    except Exception:
        pass

    return None


def _clean_title(raw_title: str) -> str:
    """Strip version/remix/quality noise from a title string."""
    title = raw_title

    for pattern in _VERSION_PATTERNS:
        if pattern in title:
            title = title.split(pattern)[0]

    for marker in _FEAT_MARKERS:
        if marker in title:
            title = title.split(marker)[0]

    if title.endswith(' (Remix)'):
        title = title[:-8]
    elif ' - Remix' in title:
        title = title.split(' - Remix')[0]

    for marker in _BONUS_MARKERS:
        if marker in title:
            title = title.split(marker)[0]

    title = title.strip()

    for suffix in _TRAILING_ARTIFACTS:
        if title.endswith(suffix):
            title = title[:-len(suffix)].strip()

    if ' #' in title:
        title = title.split(' #')[0].strip()

    title = title.replace('✞︎', '').replace('♬♪', '').strip()
    return title


def extract_metadata_from_filename(filepath: Path) -> Dict[str, Optional[str]]:
    """
    Parse artist and title from filename.
    Expects 'Artist - Title' format; falls back to using the whole stem as title.

    Returns dict with keys: artist, title, full_artist, original_title
    (full_artist and original_title are None when they match the cleaned values)
    """
    stem = filepath.stem
    for tag in _QUALITY_TAGS:
        stem = stem.replace(tag, '')
    stem = stem.strip()

    if ' - ' not in stem:
        return {'artist': '', 'title': stem, 'full_artist': None, 'original_title': None}

    full_artist, full_title = stem.split(' - ', 1)
    full_artist = full_artist.strip()
    full_title = full_title.strip()

    # Take only first credited artist
    artist = full_artist.split(', ')[0].strip() if ', ' in full_artist else full_artist
    title = _clean_title(full_title)

    return {
        'artist': artist,
        'title': title,
        'full_artist': full_artist if full_artist != artist else None,
        'original_title': full_title if full_title != title else None,
    }


def extract_metadata(filepath: Path) -> Dict[str, Optional[str]]:
    """
    Extract artist and title for a file.
    Prefers embedded tags; falls back to filename parsing.
    Always returns full_artist and original_title keys for fallback search strategies.
    """
    tag_meta = get_audio_metadata(filepath)
    filename_meta = extract_metadata_from_filename(filepath)

    if tag_meta:
        return {
            'artist': tag_meta['artist'],
            'title': tag_meta['title'],
            # Keep filename-derived fallbacks even when tags succeed
            'full_artist': filename_meta.get('full_artist'),
            'original_title': filename_meta.get('original_title'),
        }

    return filename_meta


# ---------------------------------------------------------------------------
# API search
# ---------------------------------------------------------------------------

def search_lrclib(
    artist: str,
    title: str,
    duration: Optional[float] = None,
    max_retries: int = 3,
    retry_backoff: float = 0.5,
) -> List[Dict]:
    """
    Query the LRCLIB API. Returns list of result dicts (may be empty).
    Each dict may contain 'syncedLyrics' and/or 'plainLyrics'.
    """
    params: Dict[str, str] = {'artist_name': artist, 'track_name': title}
    if duration is not None:
        params['duration'] = str(int(duration))

    url = f"https://lrclib.net/api/search?{parse.urlencode(params)}"

    for attempt in range(max_retries):
        try:
            with request.urlopen(url, timeout=15) as resp:
                data = json.loads(resp.read().decode())
                return data if isinstance(data, list) else []
        except (HTTPError, URLError, TimeoutError):
            if attempt == max_retries - 1:
                return []
            time.sleep(retry_backoff * (attempt + 1))

    return []


def _pick_lyrics(result: Dict, prefer_synced: bool = True) -> Optional[str]:
    """Extract the best available lyrics string from an LRCLIB result dict."""
    if prefer_synced:
        return result.get('syncedLyrics') or result.get('plainLyrics')
    return result.get('plainLyrics') or result.get('syncedLyrics')


def search_syncedlyrics(artist: str, title: str) -> Optional[str]:
    """
    Fallback search using the syncedlyrics library (multi-source).
    Returns raw LRC string or None.
    """
    if not SYNCEDLYRICS_AVAILABLE:
        return None
    try:
        result = syncedlyrics_search(f"{artist} {title}", providers=['lrclib', 'netease'])
        return result or None
    except Exception:
        return None


def search_song(
    song_info: Tuple[Path, Dict],
    prefer_synced: bool = True,
    request_delay: float = 0.05,
    max_retries: int = 3,
    retry_backoff: float = 0.5,
) -> Dict:
    """
    Search for lyrics for a single song using multiple fallback strategies.
    Stores content at search time to avoid a second fetch during download.

    Returns dict with keys:
        status:  'found' | 'not_found' | 'error'
        song:    the original song_info tuple
        source:  'lrclib' | 'syncedlyrics'  (only when found)
        content: lyrics string              (only when found)
    """
    filepath, metadata = song_info
    time.sleep(request_delay)

    def _lrclib_hit(artist, title) -> Optional[str]:
        results = search_lrclib(
            artist, title,
            max_retries=max_retries,
            retry_backoff=retry_backoff,
        )
        if results:
            return _pick_lyrics(results[0], prefer_synced)
        return None

    try:
        artist = metadata['artist']
        title = metadata['title']

        # 1. Cleaned metadata
        content = _lrclib_hit(artist, title)
        if content:
            return {'status': 'found', 'song': song_info, 'source': 'lrclib', 'content': content}

        # 2. Original (uncleaned) title
        original_title = metadata.get('original_title')
        if original_title and original_title != title:
            content = _lrclib_hit(artist, original_title)
            if content:
                return {'status': 'found', 'song': song_info, 'source': 'lrclib', 'content': content}

        # 3. Full artist string (multi-artist files)
        full_artist = metadata.get('full_artist')
        if full_artist and full_artist != artist:
            content = _lrclib_hit(full_artist, title)
            if content:
                return {'status': 'found', 'song': song_info, 'source': 'lrclib', 'content': content}

        # 4. syncedlyrics fallback
        content = search_syncedlyrics(artist, title)
        if content:
            return {'status': 'found', 'song': song_info, 'source': 'syncedlyrics', 'content': content}

        return {'status': 'not_found', 'song': song_info}

    except Exception:
        return {'status': 'error', 'song': song_info}


# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------

def resolve_output_path(
    filepath: Path,
    audio_dir: Path,
    output_dir: Path,
    preserve_structure: bool = True,
) -> Path:
    """
    Determine the .lrc output path for a given audio file.

    If preserve_structure is True, mirrors the relative subdirectory from
    audio_dir into output_dir to prevent filename collisions.
    """
    if preserve_structure:
        try:
            rel = filepath.relative_to(audio_dir)
            out = output_dir / rel.with_suffix('.lrc')
        except ValueError:
            out = output_dir / filepath.with_suffix('.lrc').name
    else:
        out = output_dir / filepath.with_suffix('.lrc').name

    return out


def save_lyrics(content: str, output_path: Path) -> bool:
    """
    Write lyrics content to output_path. Creates parent directories as needed.
    Returns True on success.
    """
    try:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    except IOError:
        return False
