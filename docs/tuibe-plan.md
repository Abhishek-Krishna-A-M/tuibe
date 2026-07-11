# tuibe — Full Implementation Plan

## What We're Building

A high-performance Rust TUI YouTube Music client. Search any song, play instantly via
yt-dlp + rodio. No account needed. Keyboard-first with full mouse support. Clean,
transparent terminal aesthetic.

## Project Structure

```
ytmusic-cli/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .gitignore
├── docs/
│   └── tuibe-plan.md
├── src/
│   ├── main.rs                 # entry point, terminal setup, event loop
│   ├── app.rs                  # App state, action dispatch, screen management
│   ├── search.rs               # yt-dlp search subprocess, parse JSON results
│   ├── player.rs               # dedicated audio thread, mpsc channels, rodio playback
│   ├── stream.rs               # StreamHandle: temp file + Condvar, Read+Seek
│   ├── favorites.rs            # local favorites/playlist persistence (JSON)
│   ├── config.rs               # app config (keybinds, theme)
│   ├── theme.rs                # colors, styles, transparency support
│   └── ui/
│       ├── mod.rs              # render orchestrator
│       ├── search.rs           # search input + results table layout
│       └── chunks.rs           # layout::Chunk helpers
├── python/                     # archived Python version
│   ├── ytmusic_cli/
│   ├── pyproject.toml
│   └── ...
```

## Architecture

```
┌──────────────┐     mpsc::channel      ┌──────────────┐
│              │  PlayerCommand::Play    │              │
│   UI Thread  │ ──────────────────────▶ │  Audio Thread │
│   (ratatui)  │                         │   (rodio)    │
│              │  PlayerEvent::Progress  │              │
│              │ ◀────────────────────── │              │
└──────┬───────┘                         └──────┬───────┘
       │                                        │
       │  std::process::Command                 │  StreamHandle
       │  yt-dlp -f bestaudio -o -             │  (temp file Read+Seek)
       │  ──▶ stdout ──▶ temp file ─────────────┘
       │
       │  yt-dlp --dump-json
       │  "ytsearch10:song name"
       │  ──▶ parse JSON ──▶ search results
```

## Dependencies

```toml
[package]
name = "tuibe"
version = "0.1.0"
edition = "2024"

[dependencies]
ratatui = "0.30"
crossterm = "0.29"
rodio = "0.22"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "6"
anyhow = "1"
toml = "0.8"
rand = "0.9"
```

Zero async runtime. Zero tokio. Pure std threads + mpsc channels.

## Key Components

### 1. Search (`search.rs`)

- Spawns `yt-dlp --dump-json "ytsearch10:{query}"` synchronously on a worker thread
- Parses JSON output into `Vec<Track>` structs (title, artist, duration, url, id)
- Shows loading state in UI while search runs (SearchState::Searching)
- Also supports direct URL paste (any yt-dlp supported URL)

### 2. Stream (`stream.rs`)

**Bounded temp file storage (no OOM on long streams):**
- Writes yt-dlp stdout to a temp file at `$TMPDIR/tuibe_stream_{nanos}.tmp`
- 200MB cap per stream via `AtomicU64` byte counter
- Temp file auto-deleted on `StreamHandle::drop()`

**Robust Seek:**
- `SeekFrom::Start` — clamps to available bytes
- `SeekFrom::Current` — clamps to 0..available
- `SeekFrom::End` — clamps to 0..available
- No panics on out-of-bounds seeks

**Format pinning:**
- yt-dlp args: `-f bestaudio[ext=m4a]/bestaudio` — forces m4a/AAC when available
- Falls back to bestaudio if m4a unavailable

**Progressive playback:**
- `StreamHandle` blocks via `Condvar` when read position exceeds downloaded bytes
- Playback starts before download completes

### 3. Player (`player.rs`)

- Dedicated `std::thread` for audio (no tokio)
- Communicates via `mpsc::Sender<PlayerCommand>` / `mpsc::Receiver<PlayerEvent>`
- Commands: `Play { url, generation }`, `Stop`, `Pause`, `Resume`, `Seek(Duration)`, `SetVolume(f32)`, `Quit`
- Events: `Started { duration }`, `Progress { position, duration }`, `Finished`, `Error(String)`
- Uses `DeviceSinkBuilder::open_default_sink()` + `RodioPlayer::connect_new(mixer)`
- **Generation counter**: monotonically increasing `u64` — stale play requests are discarded
- Progress tracking via dedicated thread with 500ms tick

### 4. App State (`app.rs`)

- `App` struct holds everything: search state, queue, current track, playback state, volume
- `Action` enum: every user intent maps to an action
- `dispatch(action)` processes actions synchronously
- `drain_player_events()` non-blocking poll of player event channel
- Loading states: `Idle`, `Searching`, `Loaded(Vec<Track>)`, `Error(String)`

### 5. Input (`input.rs`)

- `crossterm::event::poll(50ms)` loop with `KeyEventKind::Press` filtering
- Mouse events: `MouseEventKind::Down(Left)`, `ScrollDown/Up`
- Search mode: when `search_focused == true`, all keys go to input (Esc to exit)
- Normal mode: vim-like keybindings

**Keybindings:**
| Key | Action |
|-----|--------|
| `/` | Focus search input |
| `j`/`↓` | Move down |
| `k`/`↑` | Move up |
| `Enter` | Play selected |
| `Space` | Pause/Resume |
| `s` | Stop |
| `n` | Next track |
| `p` | Previous track |
| `→` | Seek forward 10s |
| `←` | Seek backward 10s |
| `+`/`-` | Volume up/down |
| `f` | Toggle favorite |
| `l` | Show favorites |
| `q` | Quit |
| `Esc` | Back to search |
| `Ctrl+C` | Quit |

### 6. Favorites (`favorites.rs`)

- `Favorites` struct: `Vec<Track>` persisted as JSON at `~/.config/tuibe/favorites.json`
- `add(track)`, `remove(id)`, `is_favorite(id)`, `save()`, `load()`
- No account needed — everything local

### 7. UI (`ui/`)

- Three-zone layout: search results (top), now playing bar (bottom), status (middle)
- Search input field with focus indicator
- Now playing: track title, artist, progress bar with time, volume indicator
- Transparent aesthetic: no `.bg()` calls — terminal background shows through
- Subtle borders with `Color::DarkGray`, no opaque backgrounds

## Terminal Transparency

The "transparent" look means:
- No opaque background colors on any widget
- Use `Color::Reset` or no `.bg()` calls — terminal background shows through
- Subtle borders with `BorderStyle::Rounded` or `Plain`
- Muted accent colors that blend with any terminal theme

## Implementation Notes

### OOM Protection
The stream buffer uses a bounded temp file (200MB cap) instead of unbounded `Vec<u8>`.
This prevents OOM on long compilations, livestreams, or 10-hour mixes. The temp file is
auto-deleted when the `StreamHandle` is dropped.

### Seek Stability
Seek operations are clamped to `[0, bytes_written]` to prevent panics from symphonia/rodio
when seeking beyond available data. Format is pinned to `bestaudio[ext=m4a]/bestaudio` to
minimize codec-specific seek edge cases.

### Search Latency
`yt-dlp --dump-json` can take 2-4 seconds for YouTube Music queries. The search runs on
the main thread (synchronous) but the UI immediately shows `SearchState::Searching` with
a loading indicator. The event loop continues polling at 50ms, keeping the UI responsive.

### Generation Counter
Each `Play` command carries a monotonically increasing `u64` generation. The player thread
discards any `Play` with a generation older than what it's currently processing. This
prevents stale downloads from interfering with newer ones when the user skips rapidly.
