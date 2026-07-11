from textual.app import ComposeResult
from textual.screen import Screen
from textual.widgets import Input, Static, TabbedContent, TabPane, ListView, ListItem
from textual.containers import Vertical
from textual import work
from ytmusic_cli.widgets.track_table import TrackTable
from ytmusic_cli.screens.playlist_screen import PlaylistScreen


class SearchScreen(Screen):
    BINDINGS = [
        ("escape", "app.pop_screen", "Back"),
        ("enter", "select_focused", "Select"),
    ]

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Input(placeholder="Search songs, albums, artists...", id="search-input")
            with TabbedContent(id="search-tabs"):
                with TabPane("All", id="all"):
                    yield Static("", id="search-all")
                with TabPane("Songs", id="songs"):
                    yield Static("", id="search-songs")
                with TabPane("Albums", id="albums"):
                    yield Static("", id="search-albums")
                with TabPane("Playlists", id="playlists"):
                    yield Static("", id="search-playlists")
                with TabPane("Artists", id="artists"):
                    yield Static("", id="search-artists")

    def on_mount(self):
        self.query_one("#search-input", Input).focus()

    def on_input_changed(self, event: Input.Changed):
        if len(event.value) >= 2:
            self._do_search(event.value)

    @work(exclusive=True, thread=True)
    def _do_search(self, query):
        try:
            results = self.app.api.search(query, limit=20)
            self.app.call_from_thread(self._display_results, results)
        except Exception as e:
            self.app.call_from_thread(self._show_error, str(e))

    def _display_results(self, results):
        songs = [r for r in results if r.get("resultType") == "song"]
        albums = [r for r in results if r.get("resultType") == "album"]
        playlists = [r for r in results if r.get("resultType") == "playlist"]
        artists = [r for r in results if r.get("resultType") == "artist"]

        self._display_all(results)
        self._display_songs(songs)
        self._display_albums_tab(albums)
        self._display_playlists_tab(playlists)
        self._display_artists_tab(artists)

    def _display_all(self, results):
        widget = self.query_one("#search-all", Static)
        lines = []
        for r in results[:30]:
            rt = r.get("resultType", "")
            title = r.get("title", "")
            if rt == "song":
                artists = r.get("artists", [])
                a = artists[0].get("name", "?") if artists else "?"
                lines.append(f"[yellow]S[/yellow] {title} - {a}")
            elif rt == "album":
                a = r.get("artist", "?")
                lines.append(f"[blue]A[/blue] {title} - {a}")
            elif rt == "playlist":
                a = r.get("author", "?")
                lines.append(f"[green]P[/green] {title} ({a})")
            elif rt == "artist":
                lines.append(f"[magenta]AR[/magenta] {title}")
            else:
                lines.append(f"  {title}")
        widget.update("\n".join(lines) if lines else "No results")

    def _display_songs(self, songs):
        widget = self.query_one("#search-songs", Static)
        if songs:
            table = TrackTable(id="search-song-table")
            table.set_tracks(songs)
            widget.update("")
            widget.remove()
            songs_pane = self.query_one("#songs", TabPane)
            songs_pane.mount(table)
        else:
            widget.update("No songs found")

    def _display_albums_tab(self, albums):
        widget = self.query_one("#search-albums", Static)
        if albums:
            lv = ListView(id="search-album-list")
            for a in albums:
                title = a.get("title", "Unknown")
                artist = a.get("artist", "?")
                bid = a.get("browseId", "")
                lv.append(ListItem(Static(f"{title} - {artist}"), data={"type": "album", "id": bid}))
            widget.remove()
            albums_pane = self.query_one("#albums", TabPane)
            albums_pane.mount(lv)
        else:
            widget.update("No albums found")

    def _display_playlists_tab(self, playlists):
        widget = self.query_one("#search-playlists", Static)
        if playlists:
            lv = ListView(id="search-playlist-list")
            for p in playlists:
                title = p.get("title", "Unknown")
                author = p.get("author", "?")
                pid = p.get("browseId", p.get("playlistId", ""))
                count = p.get("itemCount", "?")
                lv.append(ListItem(Static(f"{title} ({author}) - {count} tracks"),
                                   data={"type": "playlist", "id": pid}))
            widget.remove()
            playlists_pane = self.query_one("#playlists", TabPane)
            playlists_pane.mount(lv)
        else:
            widget.update("No playlists found")

    def _display_artists_tab(self, artists):
        widget = self.query_one("#search-artists", Static)
        lines = []
        for a in artists:
            lines.append(a.get("title", "Unknown"))
        widget.update("\n".join(lines) if lines else "No artists found")

    def _show_error(self, msg):
        self.query_one("#search-all", Static).update(f"[red]Error: {msg}[/red]")

    def on_list_view_selected(self, event):
        item = event.item
        data = item.data
        if data:
            if data["type"] == "playlist" and data["id"]:
                self.app.push_screen(PlaylistScreen(data["id"], title="Playlist"))
            elif data["type"] == "album" and data["id"]:
                self.app.push_screen(PlaylistScreen(data["id"], title="Album"))

    def on_data_table_row_selected(self, event):
        try:
            table = self.query_one("#search-song-table", TrackTable)
        except Exception:
            return
        if event.row_key and table._tracks:
            idx = int(str(event.row_key)) - 1
            if 0 <= idx < len(table._tracks):
                self.app.play_track(table._tracks[idx])

    def select_focused(self):
        focused = self.focused
        if focused and hasattr(focused, "action_select"):
            focused.action_select()
