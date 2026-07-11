from textual.app import ComposeResult
from textual.screen import Screen
from textual.widgets import Static, Tree
from textual.containers import Horizontal, Vertical
from textual import work
from ytmusic_cli.widgets.now_playing import NowPlaying
from ytmusic_cli.widgets.track_table import TrackTable


class MainScreen(Screen):
    BINDINGS = [
        ("j", "cursor_down", "Down"),
        ("k", "cursor_up", "Up"),
        ("h", "focus_sidebar", "Library"),
        ("l", "focus_content", "Content"),
        ("enter", "select_item", "Select"),
        ("tab", "focus_next", "Next Pane"),
    ]

    def compose(self) -> ComposeResult:
        with Horizontal(id="main-layout"):
            with Vertical(id="sidebar"):
                yield Static("Library", id="sidebar-title")
                yield Tree("Library", id="library-tree")
            with Vertical(id="content"):
                yield Static("Select a category", id="content-header")
                yield Vertical(id="content-body")
        yield NowPlaying(id="now-playing")

    def on_mount(self):
        tree = self.query_one("#library-tree", Tree)
        tree.root.add("Liked Songs", data={"type": "liked"})
        tree.root.add("Playlists", data={"type": "playlists"})
        tree.root.add("Albums", data={"type": "albums"})
        tree.root.add("History", data={"type": "history"})
        tree.root.expand()
        self.set_interval(0.5, self._sync_now_playing)
        self.focus_sidebar()

    def _sync_now_playing(self):
        np = self.query_one(NowPlaying)
        np.position = self.app.position
        np.duration = self.app.duration
        np.volume = self.app.volume
        np.state = self.app.playback_state
        np.track = self.app.current_track

    def on_tree_node_selected(self, event):
        node_data = event.node.data
        if node_data:
            self._handle_nav(node_data["type"])

    def _handle_nav(self, nav_type):
        header = self.query_one("#content-header", Static)
        header.update(nav_type.replace("_", " ").title())
        if nav_type == "liked":
            self._load_liked()
        elif nav_type == "playlists":
            self._load_playlists()
        elif nav_type == "albums":
            self._load_albums()
        elif nav_type == "history":
            self._load_history()

    @work(thread=True)
    def _load_liked(self):
        try:
            tracks = self.app.api.get_liked_songs(limit=50)
            self.app.call_from_thread(self._display_tracks, tracks)
        except Exception as e:
            self.app.call_from_thread(self._show_error, str(e))

    @work(thread=True)
    def _load_playlists(self):
        try:
            playlists = self.app.api.get_library_playlists(limit=50)
            self.app.call_from_thread(self._display_playlists, playlists)
        except Exception as e:
            self.app.call_from_thread(self._show_error, str(e))

    @work(thread=True)
    def _load_albums(self):
        try:
            albums = self.app.api.get_library_albums(limit=50)
            self.app.call_from_thread(self._display_albums, albums)
        except Exception as e:
            self.app.call_from_thread(self._show_error, str(e))

    @work(thread=True)
    def _load_history(self):
        try:
            history = self.app.api.get_history()
            self.app.call_from_thread(self._display_tracks, history)
        except Exception as e:
            self.app.call_from_thread(self._show_error, str(e))

    def _clear_body(self):
        body = self.query_one("#content-body", Vertical)
        for child in list(body.children):
            child.remove()

    def _display_tracks(self, tracks):
        self._clear_body()
        table = TrackTable(id="track-table")
        self.query_one("#content-body", Vertical).mount(table)
        table.set_tracks(tracks)

    def _display_playlists(self, playlists):
        self._clear_body()
        lines = []
        for p in playlists:
            count = p.get("count", "?")
            lines.append(f"{p.get('title', 'Unknown')}  ({count} tracks)")
        text = "\n".join(lines) if lines else "No playlists found"
        self.query_one("#content-body", Vertical).mount(Static(text))

    def _display_albums(self, albums):
        self._clear_body()
        lines = []
        for a in albums:
            artists = a.get("artists", [])
            artist = artists[0].get("name", "Unknown") if artists else "Unknown"
            lines.append(f"{a.get('title', 'Unknown')} - {artist} ({a.get('year', '')})")
        text = "\n".join(lines) if lines else "No albums found"
        self.query_one("#content-body", Vertical).mount(Static(text))

    def _show_error(self, msg):
        self._clear_body()
        self.query_one("#content-body", Vertical).mount(Static(f"[red]Error: {msg}[/red]"))

    def focus_sidebar(self):
        self.query_one("#library-tree", Tree).focus()

    def focus_content(self):
        focused = self.focused
        if focused == self.query_one("#library-tree", Tree):
            table = self.query_one("#track-table", TrackTable)
            table.focus()

    def cursor_down(self):
        focused = self.focused
        if focused and hasattr(focused, "action_cursor_down"):
            focused.action_cursor_down()

    def cursor_up(self):
        focused = self.focused
        if focused and hasattr(focused, "action_cursor_up"):
            focused.action_cursor_up()

    def select_item(self):
        focused = self.focused
        if focused and hasattr(focused, "action_select"):
            focused.action_select()

    def action_focus_sidebar(self):
        self.focus_sidebar()

    def action_focus_content(self):
        self.focus_content()
