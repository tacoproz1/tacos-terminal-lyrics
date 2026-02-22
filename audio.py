"""
Audio file utilities
Shared by all tools in the lrc-tools package
"""
import subprocess
from pathlib import Path
from typing import Optional, List

# Supported audio extensions
AUDIO_EXTENSIONS = {'.mp3', '.flac', '.m4a', '.ogg', '.opus', '.wav', '.wma', '.aac'}


def get_audio_duration(audio_path: Path) -> Optional[float]:
    """
    Get audio file duration using ffprobe.
    
    Args:
        audio_path: Path to audio file
        
    Returns:
        Duration in seconds, or None if unable to determine
    """
    try:
        result = subprocess.run(
            ['ffprobe', '-v', 'error', '-show_entries', 'format=duration',
             '-of', 'default=noprint_wrappers=1:nokey=1', str(audio_path)],
            capture_output=True, text=True, timeout=5
        )
        if result.returncode == 0 and result.stdout.strip():
            return float(result.stdout.strip())
    except Exception:
        pass
    return None


def find_audio_for_lrc(lrc_path: Path, audio_dir: Path) -> Optional[Path]:
    """
    Find matching audio file for an LRC file.
    Tries exact match first, then recursive search, then case-insensitive.
    
    Args:
        lrc_path: Path to LRC file
        audio_dir: Directory to search for audio files
        
    Returns:
        Path to matching audio file, or None if not found
    """
    base_name = lrc_path.stem
    
    # Try exact match first
    for ext in AUDIO_EXTENSIONS:
        audio_path = audio_dir / f"{base_name}{ext}"
        if audio_path.exists():
            return audio_path
    
    # Try recursive search
    for ext in AUDIO_EXTENSIONS:
        matches = list(audio_dir.rglob(f"{base_name}{ext}"))
        if matches:
            return matches[0]
    
    # Case-insensitive last resort
    for audio_file in audio_dir.rglob("*"):
        if (audio_file.stem.lower() == base_name.lower() and 
            audio_file.suffix.lower() in [e.lower() for e in AUDIO_EXTENSIONS]):
            return audio_file
    
    return None


def get_audio_files(directory: Path) -> List[Path]:
    """
    Recursively find all audio files in directory.
    
    Args:
        directory: Directory to search
        
    Returns:
        List of audio file paths
    """
    audio_files = []
    for root, _, files in directory.walk():
        for file in files:
            path = root / file
            if path.suffix.lower() in AUDIO_EXTENSIONS:
                audio_files.append(path)
    return audio_files


def find_lrc_for_audio(audio_path: Path, lrc_dir: Path, 
                       artist: str = None, title: str = None,
                       is_wlrc: bool = False) -> Optional[Path]:
    """
    Find matching LRC file for an audio file.
    
    Args:
        audio_path: Path to audio file
        lrc_dir: Directory containing LRC files
        artist: Optional artist name for fallback matching
        title: Optional title for fallback matching
        is_wlrc: Whether to look for .wlrc files instead of .lrc
        
    Returns:
        Path to matching LRC file, or None if not found
    """
    if not lrc_dir.exists():
        return None
    
    def normalize(s):
        return ''.join(c.lower() for c in s if c.isalnum())
    
    extension = '.wlrc' if is_wlrc else '.lrc'
    
    # Try exact match with audio filename
    lrc_exact = audio_path.with_suffix(extension)
    if lrc_exact.exists():
        return lrc_exact
    
    # Try in lrc_dir with same filename
    lrc_in_dir = lrc_dir / f"{audio_path.stem}{extension}"
    if lrc_in_dir.exists():
        return lrc_in_dir
    
    # Try artist - title format
    if artist and title:
        normalized_target = normalize(f"{artist}{title}")
        for lrc_file in lrc_dir.rglob(f"*{extension}"):
            normalized_file = normalize(lrc_file.stem)
            if normalized_file == normalized_target:
                return lrc_file
    
    # Try fuzzy match with filename
    audio_normalized = normalize(audio_path.stem)
    for lrc_file in lrc_dir.rglob(f"*{extension}"):
        lrc_normalized = normalize(lrc_file.stem)
        if lrc_normalized == audio_normalized:
            return lrc_file
    
    return None
