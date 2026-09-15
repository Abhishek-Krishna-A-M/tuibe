# tuibe

High-performance TUI YouTube Music client. Search, play, favorite — no account needed.

Built with Rust, [ratatui](https://ratatui.rs), [rodio](https://github.com/RustAudio/rodio), [ytmusicapi](https://github.com/sigma67/ytmusicapi), and [cava](https://github.com/karlstav/cava).

## Features

- **Search YouTube Music** — type `/`, enter a query (powered by ytmusicapi).
  Scoped prefixes: `artist:` • `album:` • `movie:` • `song:`
  (bare text = song search; `movie:` is an alias of `album:`)
- **Radio autoplay** — automatically queues related tracks from search results; in playlists, only triggers when toggled with `t`
- **Streaming playback** — audio via rodio/yt-dlp, cached to `~/.cache/tuibe/`
- **Keyboard-first** — vim-like navigation (j/k), full control without mouse
- **Mouse support** — click to play, scroll to navigate
- **PipeWire volume sync** — app volume controls system volume via `wpctl`
- **Playlists** — create, delete, import and sync from YouTube Music, save/restore local playlists; like/favorite tracks
- **Queue management** — enqueue without playing (`a`), dedicated queue view (`Tab`), shuffle (`s`), repeat cycle (`r`: Off → Queue → One)
- **Visualizer** — CAVA-powered full-width spectrum with pinkish gradient bars, white peak dots
- **Nerd Font icons** — volume ``, shuffle ``, repeat `` / ` 1`
- **Transparent aesthetic** — no opaque backgrounds, lets your terminal theme shine through
- **Fast startup** — zero async runtime, pure std threads + mpsc channels
- **Audio caching** — LRU cache at `~/.cache/tuibe/`, 500MB default cap

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) ≥ 1.85
- [Python](https://www.python.org/) ≥ 3.10 and [ytmusicapi](https://github.com/sigma67/ytmusicapi)
- [cava](https://github.com/karlstav/cava) (for the visualizer)
- [Nerd Font](https://www.nerdfonts.com/) (for icon glyphs)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) (stream URL resolution)
- ALSA development libraries (Linux):

```bash
# Python dependency
pip install ytmusicapi

# Or use a venv (auto-detected by the app)
python3 -m venv .venv
.venv/bin/pip install ytmusicapi
```

## Install

```bash
git clone https://github.com/your-user/tuibe.git
cd tuibe
cargo build --release

./target/release/tuibe
```

## Usage

```bash
tuibe
```

### Search by song / artist / movie / album

Press `/`, then type with a prefix:

| Query | What you get |
|-------|--------------|
| `artist:` | Artist list — `Enter` loads top songs, `a` enqueues them |
| `album:` | Album list (title • artist • year) — `Enter` loads its tracks |
| `movie:` | Same as `album:` (movies are albums on YT Music) |
| `song:` / bare text | Song list (default when no prefix) |

Prefix matching is case-insensitive and tolerates spaces.
An on-screen `[songs]` / `[artist]` / `[album]` badge always shows the active scope.
Every text field (search, new playlist, import) shows a visible `▎` cursor with
horizontal scrolling and full Unicode support.

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Play selected track (on artist/album: load its tracks & play) |
| `a` | Enqueue selected (add next after current; on artist/album: enqueue its tracks) |
| `Space` | Pause / Resume |
| `S` | Stop |
| `n` | Next track |
| `p` | Previous track |
| `→` | Seek forward 10s |
| `←` | Seek backward 10s |
| `+` / `=` | Volume up |
| `-` | Volume down |
| `/` | Focus search |
| `Esc` | Cancel search / back |
| `v` | Toggle visualizer |
| `f` | Toggle favorite (like/unlike current track) |
| `s` | Toggle shuffle |
| `r` | Cycle repeat: Off → Queue loop → Repeat one → Off |
| `t` | Toggle autoplay (radio/related tracks) |
| `Tab` | Toggle queue view (full-width) |
| `P` | Show playlists |
| `A` | Add selected track to playlist |
| `I` | Import playlist from YouTube Music URL |
| `S` | Sync imported playlists (re-fetch from YouTube Music) |
| `q` | Quit |
| `Ctrl+C` | Quit |

**Text editing** (search / new playlist / import — `▎` cursor visible everywhere):

| Key | Action |
|-----|--------|
| `←` / `→` (`Ctrl+B` / `Ctrl+F`) | Move cursor |
| `Home` / `End` (`Ctrl+A` / `Ctrl+E`) | Start / end of line |
| `Backspace` / `Delete` | Delete char before / under cursor |
| `Ctrl+W` | Delete word before cursor |
| `Ctrl+U` | Clear line |
| `Enter` | Submit |
| `Esc` | Cancel |

**CAVA visualizer controls**:
| Key | Action |
|-----|--------|
| `[` | Sensitivity down |
| `]` | Sensitivity up |
| `{` | Bars down |
| `}` | Bars up |

**Queue view**: `Enter` play selected, `d` remove, `Tab` back
**Playlist browser**: `N` new, `I` import, `S` sync, `d` delete, `Enter` open
**Playlist detail**: `Enter` play (full queue with selected track first), `d` remove from playlist, `Esc` back — queue panel on right
**Artist/Album results**: `Enter` expand to tracks & play, `a` expand & enqueue, `j`/`k` navigate

Global playback controls (Space/S/n/p/s/r/f/v/t/Tab) work in all screens.
Autoplay (`t`) auto-fills related tracks only from search — in playlists, enable manually with `t`.

### Mouse

- **Left click** — play selected track
- **Scroll** — navigate list

### Visualizer

Powered by [cava](https://github.com/karlstav/cava) — runs as a subprocess and pipes raw bar data into the TUI. Bars are full-width with a pinkish color gradient, white `▔` peak dots, and automatically stretch to fill the terminal width. Toggle with `v`.

## Configuration

Config file: `~/.config/tuibe/config.toml`

```toml
volume = 0.7
accent_color = "#7c3aed"

[keybindings]
search = "/"
play_pause = "Space"
next = "n"
previous = "p"
volume_up = "+"
volume_down = "-"
favorite = "f"
quit = "q"
```

## Architecture

```
UI Thread (ratatui)       Audio Thread (rodio)      Python (ytmusicapi)
       │                          │                        │
       │  PlayerCommand::Play     │                        │
       │ ────────────────────────▶│                        │
       │                          │ yt-dlp → rodio decoder  │
       │  PlayerEvent::Progress   │   ▲                    │
       │ ◀────────────────────────│   │                    │
       │                          │   │                    │
       │  read cava bars ◀────────────── CAVA subprocess    │
       │                          │                        │
    python3 -c helper search  ◀───┘                        │
    ──▶ search results (songs | artists | albums)              │
    python3 -c helper artist/album ◀────────────────────────── │
    ──▶ artist top songs / album tracks                       │
   python3 -c helper related ◀───────────────────────────── │
   ──▶ radio tracks                                         │
   python3 -c helper playlist ◀──────────────────────────── │
   ──▶ playlist tracks                                      │
```

- **No tokio** — std threads + mpsc channels
- **No OAuth** — works without any account
- **ytmusicapi** — used for scoped search (`songs`/`artists`/`albums`), artist/album expansion, radio, and playlist import via embedded Python script
- **rodio** — audio playback via yt-dlp streaming (no subprocess)
- **Bounded storage** — 200MB temp file cap per stream, LRU cache at 500MB
- **Generation counter** — stale play requests discarded on rapid skip
- **CAVA visualizer** — raw 16-bit bar data read from subprocess stdout, peak dots managed in-app

## License

MIT
