# ytmusic-cli

TUI YouTube Music client built with Python, Textual, ytmusicapi, and mpv.

## Architecture

```
app.py          # Main App: state, keybindings, screen management
api.py          # YTMusicAPI wrapper (ytmusicapi) + OAuth device flow
player.py       # MusicPlayer (python-mpv bindings)
screens/
  auth_screen.py     # OAuth device flow setup
  main_screen.py     # Three-pane: sidebar library + content + now playing
  search_screen.py   # Search with tabs
  playlist_screen.py # Playlist/album detail
widgets/
  now_playing.py     # Now playing bar (progress, time, volume)
  track_table.py     # Track list DataTable
css/app.tcss         # Textual styles
```

## Commands

```bash
# Run
python -m ytmusic_cli

# Install
pip install -e .

# Style check
ruff check .

# Type check
mypy ytmusic_cli
```

## Dependencies

- textual — TUI framework
- ytmusicapi — YouTube Music API (metadata, search, library)
- mpv (python-mpv) — Audio playback via libmpv
- httpx — Used for OAuth device flow

## System Requirements

- `libmpv1` (for python-mpv audio playback)
- `yt-dlp` (used by mpv for stream URL resolution)

## Conventions

- All API calls run in `@work(thread=True)` workers, results posted via `call_from_thread`
- Queue is managed in Python (not mpv playlist) for full control over shuffle/repeat
- State uses Textual reactives: `current_track`, `playback_state`, `queue`, `queue_index`, `volume`, `position`, `duration`, `repeat_mode`, `shuffle`
- No subprocess spawning (zero overhead) — ytmusicapi uses direct HTTP, mpv uses libmpv bindings
- All UI mutations happen on the main thread
