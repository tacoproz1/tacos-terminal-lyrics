"""
LRC file parsing utilities
Shared by all tools in the lrc-tools package
"""
import re
from pathlib import Path
from typing import List, Dict, Tuple


def parse_lrc(lrc_path: Path) -> List[Dict]:
    """
    Parse LRC file into structured format with timestamps and text.
    
    Args:
        lrc_path: Path to LRC file
        
    Returns:
        List of dicts with 'timestamp' (float) and 'text' (str) keys
    """
    lines = []
    pattern = re.compile(r'\[(\d{1,2}):(\d{2}(?:\.\d{1,3})?)\](.*)')
    
    with open(lrc_path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            
            match = pattern.match(line)
            if match:
                minutes, seconds, text = match.groups()
                timestamp = float(minutes) * 60 + float(seconds)
                text = text.strip()
                if text:
                    lines.append({
                        'timestamp': timestamp,
                        'text': text
                    })
    
    return sorted(lines, key=lambda x: x['timestamp'])


def parse_lrc_simple(lrc_path: Path) -> List[Tuple[float, str]]:
    """
    Parse LRC file into simple tuple format.
    
    Args:
        lrc_path: Path to LRC file
        
    Returns:
        List of (timestamp, text) tuples
    """
    lines = []
    pattern = re.compile(r'\[(\d{1,2}):(\d{2}\.\d{1,3})\](.+)')
    
    with open(lrc_path, 'r', encoding='utf-8') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#'):
                continue
            
            match = pattern.match(line)
            if match:
                minutes, seconds, text = match.groups()
                timestamp = int(minutes) * 60 + float(seconds)
                text = text.strip()
                if text:
                    lines.append((timestamp, text))
    
    return sorted(lines, key=lambda x: x[0])


def format_timestamp(seconds: float) -> str:
    """
    Format seconds into LRC timestamp format [MM:SS.xx]
    
    Args:
        seconds: Time in seconds
        
    Returns:
        Formatted timestamp string
    """
    minutes = int(seconds // 60)
    secs = seconds % 60
    return f"[{minutes:02d}:{secs:05.2f}]"


def write_lrc(output_path: Path, lines: List[Dict], metadata: Dict = None, header_comments: List[str] = None):
    """
    Write LRC file with proper formatting.
    
    Args:
        output_path: Path to output LRC file
        lines: List of dicts with 'timestamp' and 'text' keys
        metadata: Optional metadata dict (title, artist, etc.)
        header_comments: Optional list of header comment lines
    """
    with open(output_path, 'w', encoding='utf-8') as f:
        # Write header comments if provided
        if header_comments:
            for comment in header_comments:
                if not comment.startswith('#'):
                    comment = '# ' + comment
                f.write(f"{comment}\n")
        
        # Write metadata if provided
        if metadata:
            if 'title' in metadata:
                f.write(f"[ti:{metadata['title']}]\n")
            if 'artist' in metadata:
                f.write(f"[ar:{metadata['artist']}]\n")
            if 'album' in metadata:
                f.write(f"[al:{metadata['album']}]\n")
            if 'by' in metadata:
                f.write(f"[by:{metadata['by']}]\n")
            f.write("\n")
        
        # Write lyrics
        for line in sorted(lines, key=lambda x: x['timestamp']):
            timestamp = format_timestamp(line['timestamp'])
            f.write(f"{timestamp}{line['text']}\n")
