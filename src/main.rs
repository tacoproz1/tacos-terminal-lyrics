mod lrc;
mod lyrics;
mod player;
mod render;
mod term;
mod theme;
mod ytmgo;

use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use lrc::PlaybackStatus;
use lyrics::LyricsFinder;
use render::{fit_big, render_big, word_cols, word_times, Font};
use term::{clear_screen, hide_cursor, poll_key, show_cursor, RawMode};
use theme::Theme;

/// Set by the SIGUSR1 handler: reload the theme file on the next tick.
static RELOAD_THEME: AtomicBool = AtomicBool::new(false);

extern "C" {
    fn signal(signum: i32, handler: usize) -> usize;
}

const SIGUSR1: i32 = 10;

extern "C" fn on_sigusr1(_sig: i32) {
    RELOAD_THEME.store(true, Ordering::Relaxed);
}

/// Install the SIGUSR1 -> theme-reload hook (libc `signal` is fine here:
/// the handler only flips an atomic).
fn install_reload_handler() {
    unsafe {
        signal(SIGUSR1, on_sigusr1 as *const () as usize);
    }
}

struct Args {
    font: Font,
    karaoke: bool,
    refresh_ms: u64,
    show_header: bool,
    theme_path: Option<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        font: Font::Block,
        karaoke: true,
        refresh_ms: 50,
        show_header: true,
        theme_path: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--compact" | "-c" => args.font = Font::Compact,
            "--block" | "-b" => args.font = Font::Block,
            "--no-karaoke" => args.karaoke = false,
            "--no-header" => args.show_header = false,
            "--refresh" => {
                let v = it.next().ok_or("--refresh needs a value (ms)")?;
                args.refresh_ms = v.parse().map_err(|_| "bad --refresh value")?;
                if args.refresh_ms < 10 {
                    return Err("--refresh minimum is 10ms".into());
                }
            }
            "--theme" => {
                let v = it.next().ok_or("--theme needs a file path")?;
                args.theme_path = Some(PathBuf::from(v));
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-V" => {
                println!("tacos-lyrics {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(args)
}

fn print_help() {
    println!(
        "tacos-lyrics - terminal lyrics, zero setup

USAGE:
    tacos-lyrics [FLAGS]

It just works:
  1. finds the active MPRIS player (browser, mpv, ytmgo, spotify, vlc...)
  2. asks lrclib.net for synced lyrics (cached in ~/.cache/tacos-lyrics)
  3. renders the current line in big letters, following your terminal's
     colorscheme by default

COLORS:
  By default the lyrics use terminal palette colors, so they follow
  your colorscheme live (no config needed).
  A theme file overrides them; it is looked up at
  ~/.config/tacos-lyrics/theme.txt or passed with --theme FILE.
  Format (one per line, '#' comments):
      sung   = #cba6f7      hex color for the words already sung
      unsung = palette:8     palette slot for the not-yet-sung words
      paused = dim           whole-line color while paused
      header = default       'artist - title' line color
  Values: #rrggbb | #rgb | palette:N | default | dim
  Send SIGUSR1 (kill -USR1 <pid>) to reload the theme file live -
  that's how noctalia/matugen templates recolor the running visualizer.

FLAGS:
  -c, --compact    3-row compact font instead of the 5-row block font
  -b, --block      5-row block font (default)
      --no-karaoke color the whole line instead of word-by-word reveal
      --refresh N  redraw every N milliseconds (default 50)
      --theme FILE use this theme file instead of the default path
      --no-header  hide the \"artist - title\" status line
  -h, --help       this help
  -V, --version    version

KEYS:
  q / Ctrl-C       quit"
    );
}

/// What to draw this tick (deduplicated against the previous frame).
#[derive(Clone, PartialEq)]
struct Frame {
    line: String,     // fitted big text
    sung_cols: usize, // karaoke: columns already sung
    paused: bool,
}

fn main() {
    if let Err(e) = run() {
        let _ = show_cursor();
        let _ = clear_screen();
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    let mut theme = Theme::load(args.theme_path.as_deref());
    let finder = LyricsFinder::new();
    install_reload_handler();

    let _raw = RawMode::enable().map_err(|e| format!("not a terminal: {e}"))?;
    hide_cursor().ok();
    clear_screen().ok();

    // Background watcher: re-scan the MPRIS bus every 500ms.
    let (tx, rx) = mpsc::channel::<Option<(String, player::TrackInfo)>>();
    thread::spawn(move || loop {
        let msg = player::find_active_player().map(|ap| (ap.name, ap.track));
        if tx.send(msg).is_err() {
            return;
        }
        thread::sleep(Duration::from_millis(500));
    });

    let mut current: Option<(String, player::TrackInfo)> = None;
    let mut lyrics: Option<lrc::Lyrics> = None;
    let mut last_frame: Option<Frame> = None;

    loop {
        // SIGUSR1: reload the theme file (noctalia/matugen template output)
        if RELOAD_THEME.swap(false, Ordering::Relaxed) {
            theme = Theme::load(args.theme_path.as_deref());
            last_frame = None; // force redraw with the new colors
        }

        // Player/track change detection (drain watcher, keep latest).
        let mut update = None;
        while let Ok(msg) = rx.try_recv() {
            update = Some(msg);
        }
        if let Some(msg) = update {
            match msg {
                Some(new) if current.as_ref() != Some(&new) => {
                    current = Some(new);
                    lyrics = None; // refetch per track
                }
                None if current.is_some() => {
                    current = None;
                    lyrics = None;
                }
                _ => {}
            }
        }

        // Quit keys: q or Ctrl-C (raw mode catches the ^C byte itself).
        if matches!(poll_key(), Some(k) if k == b'q' || k == 3) {
            return finish();
        }

        // Fallback: no MPRIS player -> check the ytmgo backend (mpv IPC +
        // its SQLite queue). ytmgo's mpv doesn't expose MPRIS.
        if current.is_none() && ytmgo::is_running() {
            if let Some(track) = ytmgo::current_track() {
                let new = ("ytmgo".to_string(), track);
                if current.as_ref() != Some(&new) {
                    current = Some(new);
                    lyrics = None;
                }
            }
        }
        if current.is_none() && !ytmgo::is_running() {
            // clear a stale ytmgo binding if the socket went away
            if let Some((name, _)) = current.as_ref() {
                if name == "ytmgo" {
                    current = None;
                    lyrics = None;
                }
            }
        }

        let Some((bus_name, track)) = current.clone() else {
            draw_simple(
                &theme,
                "WAITING",
                "waiting for a player…",
                &mut last_frame,
                &args,
            )?;
            thread::sleep(Duration::from_millis(200));
            continue;
        };

        if lyrics.is_none() {
            // ytmgo keeps its own synced-lyrics cache; use it first,
            // fall back to lrclib for everything else.
            lyrics = if bus_name == "ytmgo" {
                ytmgo::cached_lyrics()
                    .map(|lrc| lrc::parse_lrc(&lrc))
                    .filter(|l| !l.lines.is_empty())
                    .or_else(|| finder.get(&track))
            } else {
                finder.get(&track)
            };
        }

        let Some(lyr) = lyrics.as_ref() else {
            draw_simple(
                &theme,
                "NO LYRICS",
                &format!("{} — not on lrclib", track.title),
                &mut last_frame,
                &args,
            )?;
            thread::sleep(Duration::from_millis(300));
            continue;
        };

        // Position + pause state from the right backend.
        let (pos, paused) = if bus_name == "ytmgo" {
            (ytmgo::position_secs(), ytmgo::is_paused().unwrap_or(false))
        } else {
            (
                player::position_of(&bus_name),
                matches!(
                    player::status_of(&bus_name),
                    None | Some(PlaybackStatus::Paused) | Some(PlaybackStatus::Stopped)
                ),
            )
        };

        // Poll real position every tick: pause/seek just work, no drift.
        if let Some(t) = pos {
            let frame = build_frame(lyr, t, paused, &args);
            if last_frame.as_ref() != Some(&frame) {
                draw_lyric_frame(&theme, &frame, &track, &mut last_frame, &args)?;
            }
        }
        thread::sleep(Duration::from_millis(args.refresh_ms));
    }
}

fn finish() -> Result<(), String> {
    let _ = show_cursor();
    let _ = clear_screen();
    Ok(())
}

/// Compute the frame to draw at playback position `t` (seconds).
fn build_frame(lyr: &lrc::Lyrics, t: f64, paused: bool, args: &Args) -> Frame {
    let (cols, _) = term::size();
    let idx = lyr.line_at(t).unwrap_or(0);
    let raw = lyr
        .lines
        .get(idx)
        .map(|l| l.text.trim())
        .unwrap_or("")
        .to_string();
    let line = if raw.is_empty() {
        "...".to_string()
    } else {
        raw
    };
    let line = fit_big(&line, args.font, cols.saturating_sub(2));

    // Karaoke: how many columns of the big render are already sung?
    let sung_cols = if args.karaoke && !paused {
        let start = lyr.lines.get(idx).map(|l| l.time).unwrap_or(t);
        let end = lyr
            .lines
            .get(idx + 1)
            .map(|l| l.time)
            .unwrap_or_else(|| start + 5.0);
        // Words sung so far -> column cut in the big render
        let cols_of_words = word_cols(&line, args.font);
        let times: Vec<f64> = word_times(&line, start, end)
            .into_iter()
            .map(|(wt, _)| wt)
            .collect();
        let mut sung_words = 0;
        for (i, wt) in times.iter().enumerate() {
            if t >= *wt {
                sung_words = i + 1;
            }
        }
        cols_of_words
            .get(sung_words.saturating_sub(1))
            .map(|&(_, end_col)| end_col)
            .unwrap_or(0)
            .min(render_big(&line, args.font)[0].chars().count())
    } else if args.karaoke && paused {
        usize::MAX // paused: whole line in the paused color, no sweep
    } else {
        0
    };

    Frame {
        line,
        sung_cols,
        paused,
    }
}

/// Draw a non-lyric screen (waiting / no lyrics) as big centered text.
fn draw_simple(
    theme: &Theme,
    big: &str,
    sub: &str,
    last: &mut Option<Frame>,
    args: &Args,
) -> Result<(), String> {
    let frame = Frame {
        line: big.to_string(),
        sung_cols: 0,
        paused: false,
    };
    let track = player::TrackInfo {
        title: sub.to_string(),
        artist: String::new(),
        album: None,
        length: None,
    };
    draw_lyric_frame(theme, &frame, &track, last, args)
}

/// Render one frame: header line + big letters centered, karaoke coloring.
fn draw_lyric_frame(
    theme: &Theme,
    frame: &Frame,
    track: &player::TrackInfo,
    last: &mut Option<Frame>,
    args: &Args,
) -> Result<(), String> {
    let (cols, rows) = term::size();
    let mut out = String::with_capacity(cols * rows * 2);
    out.push_str("\x1b[H");

    if args.show_header {
        let name = if track.artist.is_empty() {
            track.title.clone()
        } else {
            format!("{} - {}", track.artist, track.title)
        };
        let header = format!(" {name} ");
        let hpad = cols.saturating_sub(header.chars().count());
        out.push_str(&theme.header.sgr());
        out.push_str(&"─".repeat(hpad));
        out.push_str(&header);
        out.push_str("\x1b[0m\x1b[K\r\n");
    }

    let big_rows = render_big(&frame.line, args.font);
    let top = rows.saturating_sub(big_rows.len() + 1).div_ceil(2) + usize::from(args.show_header);

    for _ in 0..top {
        out.push_str("\x1b[K\r\n");
    }

    for row in &big_rows {
        let width = row.chars().count();
        let pad = cols.saturating_sub(width) / 2;
        out.push_str("\x1b[K");
        out.push_str(&" ".repeat(pad));
        if frame.paused {
            out.push_str(&theme.paused.sgr());
            out.push_str(row);
            out.push_str("\x1b[0m");
        } else if args.karaoke && frame.sung_cols > 0 {
            // sung prefix in the sung color, rest in the unsung color
            let cut = frame.sung_cols.min(width);
            let pre: String = row.chars().take(cut).collect();
            let post: String = row.chars().skip(cut).collect();
            out.push_str(&theme.sung.sgr());
            out.push_str(&pre);
            out.push_str(&theme.unsung.sgr());
            out.push_str(&post);
            out.push_str("\x1b[0m");
        } else {
            out.push_str(&theme.sung.sgr());
            out.push_str(row);
            out.push_str("\x1b[0m");
        }
        out.push_str("\r\n");
    }

    // clear the rest of the screen
    out.push_str("\x1b[J");

    let mut stdout = io::stdout();
    stdout
        .write_all(out.as_bytes())
        .map_err(|e| e.to_string())?;
    stdout.flush().map_err(|e| e.to_string())?;
    *last = Some(frame.clone());
    Ok(())
}
