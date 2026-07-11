from textual.app import ComposeResult
from textual.screen import Screen
from textual.widgets import Static, Button
from textual.containers import Horizontal, Vertical
from textual import work
from ytmusic_cli.widgets.track_table import TrackTable


class PlaylistScreen(Screen):
    BINDINGS = [
        ("escape", "app.pop_screen", "Back"),
        ("enter", "play_selected", "Play"),
    ]

    def __init__(self, playlist_id, title="Playlist"):
        super().__init__()
        self._playlist_id = playlist_id
        self._title = title
        self._tracks = []

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Static(self._title, id="playlist-header")
            with Horizontal(id="playlist-actions"):
                yield Button("Play All", id="play-all", variant="primary")
                yield Button("Shuffle", id="shuffle", variant="default")
            yield TrackTable(id="playlist-tracks")

    def on_mount(self):
        self._load()

    @work(thread=True)
    def _load(self):
        try:
            data = self.app.api.get_playlist(self._playlist_id)
            tracks = data.get("tracks", [])
            self._tracks = tracks
            self.app.call_from_thread(self._display, tracks, data.get("title", self._title))
        except Exception as e:
            self.app.call_from_thread(self._show_error, str(e))

    def _display(self, tracks, title):
        self.query_one("#playlist-header", Static).update(title)
        table = self.query_one("#playlist-tracks", TrackTable)
        table.set_tracks(tracks)

    def _show_error(self, msg):
        header = self.query_one("#playlist-header", Static)
        header.update(f"[red]Error: {msg}[/red]")

    def on_button_pressed(self, event: Button.Pressed):
        if event.button.id == "play-all" and self._tracks:
            self.app.play_playlist(self._tracks)
        elif event.button.id == "shuffle" and self._tracks:
            import random
            shuffled = list(self._tracks)
            random.shuffle(shuffled)
            self.app.play_playlist(shuffled)

    def play_selected(self):
        table = self.query_one("#playlist-tracks", TrackTable)
        row_key = table.cursor_row
        if row_key is not None and self._tracks:
            idx = int(row_key) - 1
            if 0 <= idx < len(self._tracks):
                self.app.play_track(self._tracks[idx])
