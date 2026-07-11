from textual.app import ComposeResult
from textual.screen import Screen
from textual.widgets import Input, Static, TabbedContent, TabPane, ListView, ListItem
from textual.containers import Vertical
from textual import work
from ytmusic_cli.widgets.track_table import TrackTable
from ytmusic_cli.screens.playlist_screen import PlaylistScreen


def _make_list_item(label, data):
    item = ListItem(Static(label))
    item.data = data
    return item


class SearchScreen(Screen):
    BINDINGS = [
        ("escape", "app.pop_screen", "Back"),
    ]

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Input(placeholder="Search songs, albums, artists...", id="search-input")
            with TabbedContent(id="search-tabs"):
                with TabPane("All", id="all"):
                    yield Vertical(Static("Type to search..."), id="search-all-wrap")
                with TabPane("Songs", id="songs"):
                    yield Vertical(id="search-songs-wrap")
                with TabPane("Albums", id="albums"):
                    yield Vertical(id="search-albums-wrap")
                with TabPane("Playlists", id="playlists"):
                    yield Vertical(id="search-playlists-wrap")
                with TabPane("Artists", id="artists"):
                    yield Vertical(Static(""), id="search-artists-wrap")

    def on_mount(self):
        self.query_one("#search-input", Input).focus()

    def on_input_submitted(self, event: Input.Submitted):
        if len(event.value) >= 1:
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

    def _clear_wrap(self, wrap_id):
        wrap = self.query_one(f"#{wrap_id}", Vertical)
        for child in list(wrap.children):
            child.remove()
        return wrap

    def _display_all(self, results):
        wrap = self._clear_wrap("search-all-wrap")
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
        wrap.mount(Static("\n".join(lines) if lines else "No results"))

    def _display_songs(self, songs):
        wrap = self._clear_wrap("search-songs-wrap")
        if songs:
            table = TrackTable(id="search-song-table")
            table.set_tracks(songs)
            wrap.mount(table)
            table.focus()
        else:
            wrap.mount(Static("No songs found"))

    def _display_albums_tab(self, albums):
        wrap = self._clear_wrap("search-albums-wrap")
        if albums:
            lv = ListView(id="search-album-list")
            for a in albums:
                title = a.get("title", "Unknown")
                artist = a.get("artist", "?")
                bid = a.get("browseId", "")
                lv.append(
                    _make_list_item(
                        f"{title} - {artist}",
                        {"type": "album", "id": bid},
                    )
                )
            wrap.mount(lv)
        else:
            wrap.mount(Static("No albums found"))

    def _display_playlists_tab(self, playlists):
        wrap = self._clear_wrap("search-playlists-wrap")
        if playlists:
            lv = ListView(id="search-playlist-list")
            for p in playlists:
                title = p.get("title", "Unknown")
                author = p.get("author", "?")
                pid = p.get("browseId", p.get("playlistId", ""))
                count = p.get("itemCount", "?")
                lv.append(
                    _make_list_item(
                        f"{title} ({author}) - {count} tracks",
                        {"type": "playlist", "id": pid},
                    )
                )
            wrap.mount(lv)
        else:
            wrap.mount(Static("No playlists found"))

    def _display_artists_tab(self, artists):
        wrap = self._clear_wrap("search-artists-wrap")
        lines = []
        for a in artists:
            lines.append(a.get("title", "Unknown"))
        wrap.mount(Static("\n".join(lines) if lines else "No artists found"))

    def _show_error(self, msg):
        try:
            wrap = self.query_one("#search-all-wrap", Vertical)
            for child in list(wrap.children):
                child.remove()
            wrap.mount(Static(f"[red]Error: {msg}[/red]"))
        except Exception:
            pass

    def on_list_view_selected(self, event):
        item = event.item
        data = item.data
        if not data:
            return
        item_type = data.get("type", "")
        item_id = data.get("id", "")
        if item_type == "playlist" and item_id:
            self.app.push_screen(PlaylistScreen(item_id, title="Playlist", is_album=False))
        elif item_type == "album" and item_id:
            self.app.push_screen(PlaylistScreen(item_id, title="Album", is_album=True))

    def on_data_table_row_selected(self, event):
        try:
            table = self.query_one("#search-song-table", TrackTable)
        except Exception:
            return
        if event.row_key is not None and table._tracks:
            idx = int(str(event.row_key)) - 1
            if 0 <= idx < len(table._tracks):
                self.app.play_track(table._tracks[idx])
