"""
LRC Processor - Main processing logic
"""
from pathlib import Path
from typing import List, Dict, Optional


def count_syllables(word: str) -> int:
    """Count syllables in a word (simple heuristic-based approach)"""
    word = word.lower().strip('.,!?;:\'"')
    if not word:
        return 1

    if word.endswith('e'):
        word = word[:-1]

    vowels = 'aeiouy'
    syllable_count = 0
    previous_was_vowel = False

    for char in word:
        is_vowel = char in vowels
        if is_vowel and not previous_was_vowel:
            syllable_count += 1
        previous_was_vowel = is_vowel

    if word.endswith('le') and len(word) > 2 and word[-3] not in vowels:
        syllable_count += 1

    return max(1, syllable_count)


def detect_onset_positions(audio_path: Path, start_time: float, end_time: float, num_words: int) -> List[float]:
    """
    Detect actual onset positions within the phrase.
    Returns relative timestamps (0.0 to 1.0) for each word position.
    """
    try:
        import librosa
        import numpy as np

        phrase_duration = end_time - start_time
        y, sr = librosa.load(str(audio_path), sr=22050, offset=start_time, duration=phrase_duration)

        onset_frames = librosa.onset.onset_detect(
            y=y, sr=sr, backtrack=True, units='time', delta=0.07
        )

        if len(onset_frames) == 0 or len(onset_frames) < num_words:
            return [i / num_words for i in range(num_words)]

        min_gap = 0.08
        filtered_onsets = [onset_frames[0]]
        for onset in onset_frames[1:]:
            if onset - filtered_onsets[-1] >= min_gap:
                filtered_onsets.append(onset)

        if len(filtered_onsets) < num_words:
            return [i / num_words for i in range(num_words)]

        onset_positions = filtered_onsets[:num_words]
        normalized_onsets = [max(0.0, min(1.0, pos / phrase_duration)) for pos in onset_positions]

        even_distribution = [i / num_words for i in range(num_words)]
        blend_factor = 0.5

        return [
            even_distribution[i] * (1 - blend_factor) + normalized_onsets[i] * blend_factor
            for i in range(num_words)
        ]

    except ImportError:
        raise SystemExit("[Error] librosa is required for onset detection. Install it with: pip install librosa --break-system-packages")
    except Exception as e:
        print(f"[Warning] Onset detection failed: {e}")
        return [i / num_words for i in range(num_words)]


def phrases_to_words(phrases: List[Dict], audio_path: Optional[Path] = None) -> List[Dict]:
    """Convert phrase-level to word-level timing using onset-based positioning"""
    words = []

    for i, phrase in enumerate(phrases):
        phrase_start = phrase['timestamp']
        phrase_words = phrase['text'].split()

        if i + 1 < len(phrases):
            phrase_end = phrases[i + 1]['timestamp']
        else:
            total_syllables = sum(count_syllables(w) for w in phrase_words)
            phrase_end = phrase_start + (total_syllables * 0.25)

        phrase_duration = phrase_end - phrase_start

        if audio_path and audio_path.exists():
            onset_positions = detect_onset_positions(audio_path, phrase_start, phrase_end, len(phrase_words))
        else:
            onset_positions = [j / len(phrase_words) for j in range(len(phrase_words))]

        for j, word in enumerate(phrase_words):
            words.append({
                'timestamp': phrase_start + (onset_positions[j] * phrase_duration),
                'text': word
            })

    return words


def process_long_phrases(
    lines: List[Dict],
    total_duration: float,
    max_phrase_duration: float = 2.5,
    min_phrase_duration: float = 0.3,
    max_words_per_phrase: int = 8
) -> List[Dict]:
    """Process LRC lines, splitting phrases that are too long or have commas"""
    from .processor_splitter import split_phrase_intelligently

    result = []

    for i, line in enumerate(lines):
        if i + 1 < len(lines):
            next_timestamp = lines[i + 1]['timestamp']
        else:
            next_timestamp = total_duration

        actual_duration = next_timestamp - line['timestamp']

        if actual_duration < min_phrase_duration:
            continue

        has_commas = ',' in line['text']
        should_split = (has_commas or
                       (actual_duration > max_phrase_duration and
                        len(line['text'].split()) > max_words_per_phrase))

        if should_split:
            result.extend(split_phrase_intelligently(
                line['text'],
                actual_duration,
                line['timestamp'],
                max_phrase_duration,
                max_words_per_phrase,
                True
            ))
        else:
            result.append(line)

    return result


def process_lrc_file(
    lrc_path: Path,
    audio_dir: Path,
    output_dir: Path,
    max_phrase_duration: float = 2.5,
    min_phrase_duration: float = 0.3,
    max_words_per_phrase: int = 8,
    split_on_commas: bool = True,
    require_audio: bool = True,
    overwrite: bool = False,
    output_wlrc: bool = False,
    use_onset_detection: bool = False,
    verbose: bool = True
) -> bool:
    """Process a single LRC file"""
    from .parser import parse_lrc, write_lrc
    from .audio import find_audio_for_lrc, get_audio_duration

    if verbose:
        print(f"\n{'='*60}")
        print(f"Processing: {lrc_path.name}")
        print(f"{'='*60}")

    output_filename = lrc_path.stem + '.wlrc' if output_wlrc else lrc_path.name
    output_path = output_dir / output_filename

    if output_path.exists() and not overwrite:
        if verbose:
            print(f"[Skip] Output already exists (use --overwrite)")
        return False

    lines = parse_lrc(lrc_path)
    if not lines:
        if verbose:
            print(f"[Error] No valid lines in LRC")
        return False

    if verbose:
        print(f"[Input] {len(lines)} lines")

    audio_path = find_audio_for_lrc(lrc_path, audio_dir) if audio_dir else None

    if audio_path:
        if verbose:
            print(f"[Audio] Found: {audio_path.name}")
        duration = get_audio_duration(audio_path)
        if duration:
            if verbose:
                print(f"[Audio] Duration: {duration:.2f}s")
        else:
            duration = lines[-1]['timestamp'] + 5.0
            if verbose:
                print(f"[Estimate] Using: {duration:.2f}s")
    elif require_audio:
        if verbose:
            print(f"[Error] No audio file found")
        return False
    else:
        duration = lines[-1]['timestamp'] + 5.0
        if verbose:
            print(f"[Estimate] Using: {duration:.2f}s")

    processed = process_long_phrases(
        lines, duration, max_phrase_duration, min_phrase_duration, max_words_per_phrase
    )
    if verbose:
        print(f"[Phrases] {len(lines)} → {len(processed)} lines")

    if output_wlrc:
        processed = phrases_to_words(processed, audio_path)
        if verbose:
            print(f"[Words] Converted to {len(processed)} word-level lines")

    header_comments = [
        "Processed by lrc-smart-processor.py",
        f"Max phrase duration: {max_phrase_duration}s",
        f"Total lines: {len(processed)}"
    ]

    output_path.parent.mkdir(parents=True, exist_ok=True)
    write_lrc(output_path, processed, header_comments=header_comments)
    if verbose:
        print(f"[Success] ✓ Wrote to {output_path.name}")

    return True
