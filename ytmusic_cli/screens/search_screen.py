from textual.app import ComposeResult
from textual.screen import Screen
from textual.widgets import Input, Static, TabbedContent, TabPane
from textual.containers import Vertical
from textual import work


class SearchScreen(Screen):
    BINDINGS = [
        ("escape", "app.pop_screen", "Back"),
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
        for tab in ["all", "songs", "albums", "playlists", "artists"]:
            widget = self.query_one(f"#search-{tab}", Static)
            items = [r for r in results if r.get("resultType") == tab.rstrip("s") or tab == "all"]
            if tab == "all":
                lines = []
                for r in results[:30]:
                    result_type = r.get("resultType", "")
                    title = r.get("title", "")
                    if result_type == "song":
                        artists = r.get("artists", [])
                        artist = artists[0].get("name", "?") if artists else "?"
                        lines.append(f"[yellow]S[/yellow] {title} - {artist}")
                    elif result_type == "album":
                        a = r.get("artist", "?")
                        lines.append(f"[blue]A[/blue] {title} - {a}")
                    elif result_type == "playlist":
                        a = r.get("author", "?")
                        lines.append(f"[green]P[/green] {title} ({a})")
                    elif result_type == "artist":
                        lines.append(f"[magenta]AR[/magenta] {title}")
                    else:
                        lines.append(f"  {title}")
                widget.update("\n".join(lines) if lines else "No results")
            else:
                result_type = tab.rstrip("s")
                items = [r for r in results if r.get("resultType") == result_type]
                lines = []
                for r in items[:20]:
                    title = r.get("title", "")
                    if result_type == "song":
                        artists = r.get("artists", [])
                        artist = artists[0].get("name", "?") if artists else "?"
                        lines.append(f"{title} - {artist}")
                    elif result_type == "album":
                        a = r.get("artist", "?")
                        lines.append(f"{title} - {a}")
                    elif result_type == "playlist":
                        a = r.get("author", "?")
                        count = r.get("itemCount", "?")
                        lines.append(f"{title} ({count} tracks)")
                    elif result_type == "artist":
                        lines.append(f"{title}")
                widget.update("\n".join(lines) if lines else "No results")

    def _show_error(self, msg):
        widget = self.query_one("#search-all", Static)
        widget.update(f"[red]Error: {msg}[/red]")
