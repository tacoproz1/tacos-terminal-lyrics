"""
Media player integration using playerctl
Handles communication with media players via MPRIS
"""
import subprocess
from pathlib import Path
from typing import Optional, Tuple


def get_position() -> Optional[float]:
    """
    Get current playback position in seconds.
    
    Returns:
        Current position in seconds, or None if unavailable
    """
    try:
        result = subprocess.run(
            ['playerctl', 'position'], 
            capture_output=True, 
            text=True, 
            timeout=0.5
        )
        return float(result.stdout.strip()) if result.returncode == 0 else None
    except Exception:
        return None


def get_track() -> Optional[Tuple[str, str]]:
    """
    Get currently playing track information.
    
    Returns:
        Tuple of (artist, title), or None if unavailable
    """
    try:
        result = subprocess.run(
            ['playerctl', 'metadata', '--format', '{{artist}}|||{{title}}'], 
            capture_output=True, 
            text=True, 
            timeout=0.5
        )
        if result.returncode == 0:
            parts = result.stdout.strip().split('|||')
            return (parts[0], parts[1]) if len(parts) == 2 else None
    except Exception:
        return None


def get_status() -> Optional[str]:
    """
    Get current playback status.
    
    Returns:
        Status string ('Playing', 'Paused', 'Stopped'), or None if unavailable
    """
    try:
        result = subprocess.run(
            ['playerctl', 'status'], 
            capture_output=True, 
            text=True, 
            timeout=0.5
        )
        return result.stdout.strip() if result.returncode == 0 else None
    except Exception:
        return None


def get_audio_file_info() -> Optional[Path]:
    """
    Get currently playing audio file path.
    
    Returns:
        Path to audio file, or None if unavailable
    """
    try:
        result = subprocess.run(
            ['playerctl', 'metadata', '--format', '{{xesam:url}}'], 
            capture_output=True, 
            text=True, 
            timeout=0.5
        )
        if result.returncode == 0:
            url = result.stdout.strip()
            if url.startswith('file://'):
                return Path(url[7:])
    except Exception:
        pass
    return None


def is_paused() -> bool:
    """
    Check if playback is currently paused.
    
    Returns:
        True if paused, False otherwise
    """
    status = get_status()
    return status == 'Paused' if status else False


def is_playing() -> bool:
    """
    Check if playback is currently active.
    
    Returns:
        True if playing, False otherwise
    """
    status = get_status()
    return status == 'Playing' if status else False
