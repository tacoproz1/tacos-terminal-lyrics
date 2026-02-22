"""
LRC Processor CLI - Split and convert LRC files to word-level timing
"""
import argparse
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(
        description='Process LRC files: split long phrases and optionally convert to word-level WLRC'
    )
    parser.add_argument('--lrc-dir', type=Path, required=True,
                        help='Directory containing input .lrc files')
    parser.add_argument('--audio-dir', type=Path,
                        help='Directory containing audio files (used for duration and onset detection)')
    parser.add_argument('--output-dir', type=Path, required=True,
                        help='Directory to write processed files')
    parser.add_argument('--wlrc', action='store_true',
                        help='Convert output to word-level WLRC format')
    parser.add_argument('--overwrite', action='store_true',
                        help='Overwrite existing output files')
    parser.add_argument('--no-require-audio', action='store_true',
                        help='Process files even if no matching audio is found (uses estimated duration)')
    parser.add_argument('--max-phrase-duration', type=float, default=None,
                        help='Max phrase duration in seconds before splitting (default: 2.5)')
    parser.add_argument('--min-phrase-duration', type=float, default=None,
                        help='Min phrase duration in seconds, shorter are skipped (default: 0.3)')
    parser.add_argument('--max-words', type=int, default=None,
                        help='Max words per phrase (default: 8)')
    parser.add_argument('--no-split-commas', action='store_true',
                        help='Do not split phrases at commas')
    parser.add_argument('--onset-detection', action='store_true',
                        help='Use librosa onset detection for word timing (requires librosa)')
    parser.add_argument('--quiet', action='store_true',
                        help='Suppress per-file output')
    parser.add_argument('--config', type=Path,
                        help='Path to config.yaml')

    args = parser.parse_args()

    lrc_dir = args.lrc_dir
    output_dir = args.output_dir

    if not lrc_dir.exists():
        print(f"Error: LRC directory '{lrc_dir}' does not exist")
        return 1

    if args.audio_dir and not args.audio_dir.exists():
        print(f"Error: audio directory '{args.audio_dir}' does not exist")
        return 1

    output_dir.mkdir(parents=True, exist_ok=True)

    # Load config, then let CLI args override
    try:
        from .config import Config
        cfg = Config(args.config).processor
    except ImportError:
        cfg = None

    max_phrase_duration = args.max_phrase_duration or (cfg.max_phrase_duration if cfg else 2.5)
    min_phrase_duration = args.min_phrase_duration or (cfg.min_phrase_duration if cfg else 0.3)
    max_words_per_phrase = args.max_words or (cfg.max_words_per_phrase if cfg else 8)
    split_on_commas = not args.no_split_commas
    use_onset_detection = args.onset_detection or (cfg.use_onset_detection if cfg else False)
    require_audio = not args.no_require_audio
    verbose = not args.quiet

    try:
        from .processor_main import process_lrc_file
    except ImportError as e:
        print(f"Error: could not import processor module — {e}")
        return 1

    lrc_files = list(lrc_dir.glob('*.lrc'))

    if not lrc_files:
        print(f"No .lrc files found in '{lrc_dir}'")
        return 0

    print(f"\n{'='*60}")
    print(f"LRC PROCESSOR")
    print(f"{'='*60}")
    print(f"Input dir:     {lrc_dir}")
    print(f"Output dir:    {output_dir}")
    print(f"Audio dir:     {args.audio_dir or '(none)'}")
    print(f"Output format: {'WLRC (word-level)' if args.wlrc else 'LRC (phrase-level)'}")
    print(f"Max duration:  {max_phrase_duration}s")
    print(f"Max words:     {max_words_per_phrase}")
    print(f"Split commas:  {split_on_commas}")
    print(f"Onset detect:  {use_onset_detection}")
    print(f"Require audio: {require_audio}")
    print(f"Overwrite:     {args.overwrite}")
    print(f"Files found:   {len(lrc_files)}")
    print(f"{'='*60}\n")

    success = 0
    skipped = 0
    errors = 0

    for lrc_path in lrc_files:
        result = process_lrc_file(
            lrc_path=lrc_path,
            audio_dir=args.audio_dir,
            output_dir=output_dir,
            max_phrase_duration=max_phrase_duration,
            min_phrase_duration=min_phrase_duration,
            max_words_per_phrase=max_words_per_phrase,
            split_on_commas=split_on_commas,
            require_audio=require_audio,
            overwrite=args.overwrite,
            output_wlrc=args.wlrc,
            use_onset_detection=use_onset_detection,
            verbose=verbose,
        )
        if result is True:
            success += 1
        elif result is False:
            # False can mean skipped (exists) or error — processor prints the reason
            skipped += 1

    print(f"\n{'='*60}")
    print(f"Done!")
    print(f"  ✓ Processed: {success}")
    print(f"  - Skipped:   {skipped}")
    print(f"\nOutput: {output_dir}")
    print(f"{'='*60}")

    return 0


if __name__ == '__main__':
    sys.exit(main())
