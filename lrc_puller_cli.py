"""
LRC Puller CLI - Batch download synchronized lyrics from LRCLIB
"""
import argparse
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path


def _progress_bar(processed: int, total: int, found: int, errors: int):
    bar_len = 40
    filled = int(bar_len * processed / total)
    bar = '█' * filled + '░' * (bar_len - filled)
    pct = (processed / total) * 100
    print(
        f"\r  [{bar}] {processed}/{total} ({pct:.1f}%)"
        f" — Found: {found} | Errors: {errors}",
        end='', flush=True
    )


def main():
    parser = argparse.ArgumentParser(
        description="Batch download LRC lyrics from LRCLIB (with syncedlyrics fallback)"
    )
    parser.add_argument('--audio-dir', type=Path, required=True,
                        help='Directory containing audio files')
    parser.add_argument('--output-dir', type=Path, required=True,
                        help='Directory to save .lrc files')
    parser.add_argument('--search-threads', type=int, default=None,
                        help='Concurrent search threads (default: 5)')
    parser.add_argument('--download-threads', type=int, default=None,
                        help='Concurrent download threads (default: 5)')
    parser.add_argument('--overwrite', action='store_true',
                        help='Overwrite existing .lrc files')
    parser.add_argument('--no-preserve-structure', action='store_true',
                        help='Flatten output dir instead of mirroring audio dir structure')
    parser.add_argument('--plain-only', action='store_true',
                        help='Prefer plain lyrics over synced')
    parser.add_argument('--config', type=Path,
                        help='Path to config.yaml')

    args = parser.parse_args()

    audio_dir = args.audio_dir
    output_dir = args.output_dir

    if not audio_dir.exists():
        print(f"Error: audio directory '{audio_dir}' does not exist")
        return 1

    output_dir.mkdir(parents=True, exist_ok=True)

    # Load config, let CLI args override
    try:
        from .config import Config
        cfg = Config(args.config).puller
    except ImportError:
        cfg = None

    prefer_synced = not args.plain_only
    preserve_structure = not args.no_preserve_structure
    overwrite = args.overwrite
    def prompt_threads(label, default):
        if default:
            return default
        while True:
            try:
                val = input(f"  {label} threads (1-40, default {cfg.search_threads if cfg else 5}): ").strip()
                if not val:
                    return cfg.search_threads if cfg else 5
                n = int(val)
                if 1 <= n <= 40:
                    return n
                print("  Enter a number between 1 and 40")
            except ValueError:
                print("  Enter a valid number")
            except KeyboardInterrupt:
                print("\nCancelled")
                sys.exit(0)

    search_threads = prompt_threads("Search", args.search_threads)
    download_threads = prompt_threads("Download", args.download_threads)

    if cfg and not args.overwrite:
        overwrite = cfg.overwrite
    if cfg and not args.no_preserve_structure:
        preserve_structure = cfg.preserve_structure
    if cfg and not args.plain_only:
        prefer_synced = cfg.prefer_synced

    try:
        from .audio import get_audio_files
        from .puller import (
            extract_metadata, search_song, save_lyrics,
            resolve_output_path, MUTAGEN_AVAILABLE, SYNCEDLYRICS_AVAILABLE
        )
    except ImportError as e:
        print(f"Error: could not import required module — {e}")
        return 1

    print(f"\n{'='*60}")
    print(f"LRC PULLER")
    print(f"{'='*60}")
    print(f"Audio dir:     {audio_dir}")
    print(f"Output dir:    {output_dir}")
    print(f"Threads:       {search_threads} search / {download_threads} download")
    print(f"Overwrite:     {overwrite}")
    print(f"Struct mirror: {preserve_structure}")
    print(f"Mutagen:       {'yes' if MUTAGEN_AVAILABLE else 'no (pip install mutagen)'}")
    print(f"Syncedlyrics:  {'yes' if SYNCEDLYRICS_AVAILABLE else 'no (pip install syncedlyrics)'}")
    print(f"{'='*60}")

    print("\nScanning for audio files...")
    audio_files = get_audio_files(audio_dir)

    if not audio_files:
        print("No audio files found.")
        return 0

    songs = []
    skipped = 0
    for f in audio_files:
        out_path = resolve_output_path(f, audio_dir, output_dir, preserve_structure)
        if out_path.exists() and not overwrite:
            skipped += 1
            continue
        songs.append((f, extract_metadata(f)))

    print(f"Found {len(audio_files)} audio files")
    if skipped:
        print(f"Skipping {skipped} already fetched (use --overwrite to re-fetch)")
    print(f"To fetch: {len(songs)}")

    if not songs:
        print("Nothing to do.")
        return 0

    print("\nSample metadata:")
    for i, (fp, meta) in enumerate(songs[:5], 1):
        print(f"  {i}. {meta['artist'] or '(no artist)'} — {meta['title']}")
    if len(songs) > 5:
        print(f"  ... and {len(songs) - 5} more")

    # Search phase
    print(f"\nSearching for lyrics ({search_threads} threads)...")
    found_results = []
    not_found = 0
    errors = 0
    processed = 0

    with ThreadPoolExecutor(max_workers=search_threads) as executor:
        futures = {
            executor.submit(search_song, song, prefer_synced): song
            for song in songs
        }
        for future in as_completed(futures):
            result = future.result()
            processed += 1

            if result['status'] == 'found':
                found_results.append(result)
            elif result['status'] == 'error':
                errors += 1
                not_found += 1
            else:
                not_found += 1

            _progress_bar(processed, len(songs), len(found_results), errors)

    print()
    print(f"\nFound lyrics for {len(found_results)}/{len(songs)} songs")
    if not_found:
        print(f"Not found: {not_found}")

    if not found_results:
        print("Nothing to download.")
        return 0

    try:
        confirm = input(f"\nDownload {len(found_results)} lyrics files? (y/n): ").strip().lower()
    except KeyboardInterrupt:
        print("\nCancelled")
        return 0

    if confirm != 'y':
        print("Cancelled")
        return 0

    # Save phase (content already in memory)
    print(f"\nSaving lyrics ({download_threads} threads)...")
    tally = {'success': 0, 'error': 0}
    sources = {'lrclib': 0, 'syncedlyrics': 0}

    def _save_result(search_result):
        filepath, _ = search_result['song']
        out_path = resolve_output_path(filepath, audio_dir, output_dir, preserve_structure)
        ok = save_lyrics(search_result['content'], out_path)
        return {
            'file': filepath.name,
            'status': 'success' if ok else 'error',
            'source': search_result.get('source', 'lrclib'),
        }

    with ThreadPoolExecutor(max_workers=download_threads) as executor:
        futures = {executor.submit(_save_result, r): r for r in found_results}
        for i, future in enumerate(as_completed(futures), 1):
            result = future.result()
            status = result['status']
            tally[status] = tally.get(status, 0) + 1
            if status == 'success':
                sources[result['source']] = sources.get(result['source'], 0) + 1
            print(f"  [{i}/{len(found_results)}] {result['file']}: {status}")

    print(f"\n{'='*60}")
    print(f"Done!")
    print(f"  ✓ Saved:   {tally['success']}")
    if sources.get('lrclib'):
        print(f"      lrclib:       {sources['lrclib']}")
    if sources.get('syncedlyrics'):
        print(f"      syncedlyrics: {sources['syncedlyrics']}")
    if tally.get('error'):
        print(f"  ✗ Errors:  {tally['error']}")
    print(f"\nOutput: {output_dir}")
    print(f"{'='*60}")

    return 0


if __name__ == '__main__':
    sys.exit(main())
