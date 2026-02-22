"""
LRC Visualizer - Main display loop
Synchronizes lyrics with media player
"""
import time
import threading
from pathlib import Path
from typing import Optional


class SyncData:
    """Shared synchronization data for visualizer"""

    def __init__(self):
        self.position: Optional[float] = None
        self.should_resync: bool = False
        self.running: bool = True
        self.current_title: Optional[str] = None
        self.paused: bool = False


def position_monitor(sync_data: SyncData, get_position_func, get_track_func, get_status_func):
    """Background thread to monitor playback position and detect seeking."""
    last_check = time.time()
    expected_pos = None

    while sync_data.running:
        time.sleep(0.2)

        track_info = get_track_func()
        if track_info and sync_data.current_title:
            if track_info[1] != sync_data.current_title:
                sync_data.should_resync = True
                continue

        if sync_data.position is None:
            continue

        actual_pos = get_position_func()
        if actual_pos is None:
            continue

        status = get_status_func()
        if status == 'Paused':
            sync_data.position = actual_pos
            sync_data.should_resync = True
            sync_data.paused = True
            expected_pos = None
            continue

        sync_data.paused = False

        if expected_pos is None:
            expected_pos = actual_pos
        else:
            elapsed = time.time() - last_check
            expected_pos += elapsed

        if abs(actual_pos - expected_pos) > 1.0:
            sync_data.position = actual_pos
            sync_data.should_resync = True

        expected_pos = actual_pos
        last_check = time.time()


def run_visualizer(
    lrc_dir: Path,
    audio_dir: Optional[Path] = None,
    is_wlrc: bool = False,
    font_data: dict = None,
    refresh_rate: float = 0.05
):
    """Run the LRC visualizer main loop."""
    from .visualizer_player import get_position, get_track, get_status, get_audio_file_info
    from .visualizer_display import display_text, display_waiting, hide_cursor, show_cursor, clear_screen
    from .parser import parse_lrc_simple
    from .audio import find_lrc_for_audio

    hide_cursor()
    clear_screen()
    display_waiting()

    sync_data = SyncData()

    monitor_thread = threading.Thread(
        target=position_monitor,
        args=(sync_data, get_position, get_track, get_status),
        daemon=True
    )
    monitor_thread.start()

    try:
        last_title = None

        while sync_data.running:
            track = get_track()
            if not track:
                time.sleep(1)
                continue

            artist, title = track
            song_changed = last_title and title != last_title
            last_title = title

            audio_file = get_audio_file_info()
            lrc = find_lrc_for_audio(
                audio_file if audio_file else Path(title),
                lrc_dir,
                artist,
                title,
                is_wlrc=is_wlrc
            )

            if not lrc:
                time.sleep(1)
                continue

            sync_data.current_title = title
            lines = parse_lrc_simple(lrc)
            if not lines:
                time.sleep(1)
                continue

            pos = get_position()
            if pos is None:
                time.sleep(1)
                continue

            if song_changed and pos > 5.0:
                for _ in range(20):
                    pos = get_position()
                    if pos is not None and pos < 5.0:
                        break
                    time.sleep(0.1)

                pos = get_position()
                if pos is None:
                    time.sleep(1)
                    continue

            idx = 0
            for i, (start, _) in enumerate(lines):
                if pos < start:
                    break
                idx = i

            start_time = time.time()
            start_pos = pos
            sync_data.position = pos

            while idx < len(lines):
                if sync_data.should_resync:
                    new_track = get_track()
                    if not new_track or new_track[1] != title:
                        break

                    new_pos = sync_data.position
                    start_time = time.time()
                    start_pos = new_pos

                    idx = 0
                    for i, (start, _) in enumerate(lines):
                        if new_pos >= start:
                            idx = i
                        else:
                            break

                    sync_data.should_resync = False

                elapsed = time.time() - start_time
                current_pos = start_pos + elapsed

                _, text = lines[idx]
                display_text(text, use_block_letters=True, font_data=font_data, clear=True)

                if idx + 1 < len(lines):
                    next_start, _ = lines[idx + 1]
                    if current_pos >= next_start:
                        idx += 1
                        continue

                time.sleep(refresh_rate)

    except KeyboardInterrupt:
        pass
    finally:
        sync_data.running = False
        show_cursor()
        clear_screen()
