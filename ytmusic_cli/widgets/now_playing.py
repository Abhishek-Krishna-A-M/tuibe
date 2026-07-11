from textual.widgets import Static, Label, ProgressBar
from textual.reactive import reactive
from textual.app import ComposeResult
from textual.containers import Horizontal


class NowPlaying(Static):
    position = reactive(0.0)
    duration = reactive(0.0)
    volume = reactive(100)
    state = reactive("stopped")
    track = reactive(None)

    def compose(self) -> ComposeResult:
        with Horizontal(id="now-playing-bar"):
            yield Label("Nothing Playing", id="np-info")
            yield ProgressBar(total=100, id="np-progress", show_eta=False)
            yield Label("0:00 / 0:00", id="np-time")
            yield Label("Vol: 100%", id="np-vol")

    def on_mount(self):
        self._info = self.query_one("#np-info", Label)
        self._progress = self.query_one("#np-progress", ProgressBar)
        self._time = self.query_one("#np-time", Label)
        self._vol = self.query_one("#np-vol", Label)

    def watch_position(self, pos):
        if self.duration > 0:
            self._progress.progress = (pos / self.duration) * 100
        self._update_time()

    def watch_duration(self, _):
        self._update_time()

    def watch_volume(self, vol):
        self._vol.update(f"Vol: {vol}%")

    def watch_state(self, state):
        if self.track and state != "stopped":
            title = self.track.get("title", "Unknown")
            artists = self.track.get("artists", [])
            artist = artists[0].get("name", "Unknown") if artists else "Unknown"
            icon = "▶" if state == "playing" else "⏸"
            self._info.update(f"{icon} {title} - {artist}")
        else:
            self._info.update("Nothing Playing")

    def _update_time(self):
        cur = self._fmt_time(self.position)
        total = self._fmt_time(self.duration)
        self._time.update(f"{cur} / {total}")

    @staticmethod
    def _fmt_time(seconds):
        if seconds <= 0:
            return "0:00"
        m = int(seconds // 60)
        s = int(seconds % 60)
        return f"{m}:{s:02d}"
