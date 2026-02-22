"""
Display utilities for LRC visualizer
Handles rendering text in various styles to terminal
"""
import sys
import os
from typing import List


def get_terminal_size() -> tuple:
    """
    Get terminal dimensions.
    
    Returns:
        Tuple of (columns, rows)
    """
    try:
        size = os.get_terminal_size()
        return size.columns, size.lines
    except Exception:
        return 80, 24  # Default fallback


def clear_screen():
    """Clear the terminal screen"""
    sys.stdout.write('\033[2J\033[H')
    sys.stdout.flush()


def hide_cursor():
    """Hide terminal cursor"""
    sys.stdout.write('\033[?25l')
    sys.stdout.flush()


def show_cursor():
    """Show terminal cursor"""
    sys.stdout.write('\033[?25h')
    sys.stdout.flush()


def render_block_text(text: str, font_data: dict) -> str:
    """
    Render text using block letters.
    
    Args:
        text: Text to render
        font_data: Font dictionary mapping characters to line arrays
        
    Returns:
        Rendered text as string
    """
    text = text.upper()
    
    # Get height from first available character
    height = len(font_data.get('A', ['']))
    lines = ['' for _ in range(height)]
    
    for char in text:
        if char in font_data:
            char_lines = font_data[char]
            for i in range(height):
                if i < len(char_lines):
                    lines[i] += char_lines[i] + ' '
                else:
                    lines[i] += ' ' * (len(char_lines[0]) + 1)
        else:
            # Unknown character - use space
            space_width = len(font_data.get(' ', ['    '])[0])
            for i in range(height):
                lines[i] += ' ' * (space_width + 1)
    
    # Center in terminal
    cols, rows = get_terminal_size()
    
    # Scale if needed to fit
    max_width = max(len(line) for line in lines) if lines else 0
    if max_width > cols:
        # Simple scaling - just truncate for now
        lines = [line[:cols] for line in lines]
    
    # Center vertically
    pad_top = max(0, (rows - len(lines)) // 2)
    
    output = []
    for i in range(rows):
        if pad_top <= i < pad_top + len(lines):
            line = lines[i - pad_top]
            pad_left = max(0, (cols - len(line)) // 2)
            centered = ' ' * pad_left + line
            output.append(centered.ljust(cols))
        else:
            output.append(' ' * cols)
    
    return '\n'.join(output)


def render_simple_text(text: str, centered: bool = True) -> str:
    """
    Render text in simple format (no block letters).
    
    Args:
        text: Text to render
        centered: Whether to center the text
        
    Returns:
        Rendered text as string
    """
    cols, rows = get_terminal_size()
    
    if centered:
        pad_top = rows // 2
        pad_left = max(0, (cols - len(text)) // 2)
        
        output = []
        for i in range(rows):
            if i == pad_top:
                output.append(' ' * pad_left + text)
            else:
                output.append(' ' * cols)
        
        return '\n'.join(output)
    else:
        return text


def render_waiting() -> str:
    """
    Render waiting/loading indicator.
    
    Returns:
        Rendered waiting text
    """
    return render_simple_text("•••", centered=True)


def display_text(text: str, use_block_letters: bool = True, font_data: dict = None, clear: bool = True):
    """
    Display text in the terminal.
    
    Args:
        text: Text to display
        use_block_letters: Whether to use block letter rendering
        font_data: Font to use for block letters
        clear: Whether to clear screen first
    """
    if clear:
        sys.stdout.write('\033[2J\033[H')
    
    if use_block_letters and font_data:
        output = render_block_text(text, font_data)
    else:
        output = render_simple_text(text)
    
    sys.stdout.write(output)
    sys.stdout.flush()


def display_waiting(clear: bool = True):
    """
    Display waiting indicator.
    
    Args:
        clear: Whether to clear screen first
    """
    if clear:
        sys.stdout.write('\033[2J\033[H')
    
    output = render_waiting()
    sys.stdout.write(output)
    sys.stdout.flush()
