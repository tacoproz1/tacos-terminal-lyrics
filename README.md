# lrc-tools

[EVERY FILE IS VIBECODED IF THAT WASNT OBVIOUS ENOUGH,SORRY]
ALSO THIS BUILD SUCKSSSS, in regards to the lyric accuracy (faster the spoken lyrics in song the more accurate the wlrc will be)
also non mainstream songs occasionally dont get pulled cause they are simply not in lrclib
there are of course tons of small other issues, which i will have fixed eventually


Terminal lyrics visualizer with word-level sync. Displays song lyrics in large
ASCII block letters in your terminal, synchronized to whatever is playing in
your media player.

```
  ██   ██ ███████ ██      ██       ██████
  ██   ██ ██      ██      ██      ██    ██
  ███████ █████   ██      ██      ██    ██
  ██   ██ ██      ██      ██      ██    ██
  ██   ██ ███████ ███████ ███████  ██████
```

## Dependencies

**Required:**
- `python >= 3.12`
- `python-pyyaml`
- `ffmpeg` (provides ffprobe)
- `playerctl`

**Optional but you should really really get it:**
- `python-mutagen` — reads embedded audio tags for better lyrics matching
- `python-syncedlyrics` — fallback lyrics source (NetEase, etc.)
- `python-librosa` — onset detection for more accurate word-level timing

## Install

### AUR (havent made it yet, prolly wont though)
```bash
paru -S lrc-tools
```

### Manual (do this)
```bash
git clone https://github.com/tacoproz1/tacos-terminal-lyrics
cd tacos-terminal-lyrics
bash setup.sh
pip install pyyaml mutagen syncedlyrics --break-system-packages
```

## Quickstart

```bash
mkdir -p ~/lyrics/raw ~/lyrics/processed

# 1. Download lyrics for your music library (if using bash and not fish run export PATH="$HOME/.local/bin:$PATH")
lrc-fetch --audio-dir ~/music --output-dir ~/lyrics/raw
# 2. Process into word-level timing
lrc-processor --lrc-dir ~/lyrics/raw --audio-dir ~/music \
              --output-dir ~/lyrics/processed --wlrc

# 3. Start the visualizer, play something in your media player
lrc-vis --lrc-dir ~/lyrics/processed --wlrc
```

Steps 1 and 2 are one-time setup per library. `lrc-vis` is the daily driver.

## How it works

**`lrc-fetch`** scans your music directory, reads embedded tags (or parses
filenames), and downloads synced LRC lyrics from [LRCLIB](https://lrclib.net)
with syncedlyrics as a fallback.

**`lrc-processor`** takes standard LRC files (phrase-level timing) and splits
long phrases at natural boundaries — commas, conjunctions, duration — and
optionally converts to word-level WLRC format using even distribution or
librosa onset detection.

**`lrc-vis`** hooks into your media player via playerctl (MPRIS), finds the
matching LRC/WLRC file for the current track, and renders lyrics as large
block letters centered in the terminal. Handles seeking, pausing, and track
changes automatically.

## LRC file matching

Output filenames must match audio filenames. `Artist - Title.mp3` →
`Artist - Title.lrc`. `lrc-fetch` handles this automatically. If you source
LRC files elsewhere, name them to match.

## Configuration

Copy `config_example.yaml` to `~/.config/lrc-tools/config.yaml` and pass it
with `--config`:

```bash
lrc-vis --config ~/.config/lrc-tools/config.yaml --lrc-dir ~/lyrics/processed --wlrc
```

Key settings:

```yaml
processor:
  max_phrase_duration: 2.5   # split phrases longer than this (seconds)
  max_words_per_phrase: 8

puller:
  search_threads: 5
  download_threads: 5
  preserve_structure: true   # mirror audio dir layout in lyrics dir

visualizer:
  default_font: block        # block or compact
  refresh_rate: 0.05
```

## Custom fonts

Fonts are defined in JSON. See `custom_fonts.json` for the format.

```bash
lrc-vis --lrc-dir ~/lyrics/processed --wlrc \
        --custom-fonts custom_fonts.json --font mini
```

## License

MIT
# tacos-terminal-lyrics
