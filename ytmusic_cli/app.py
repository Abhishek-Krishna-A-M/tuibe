from textual.app import App
from textual.reactive import reactive
from textual.binding import Binding
from ytmusic_cli.api import YTMusicAPI
from ytmusic_cli.player import MusicPlayer
from ytmusic_cli.screens.auth_screen import AuthScreen
from ytmusic_cli.screens.main_screen import MainScreen
from ytmusic_cli.screens.search_screen import SearchScreen


class YTMusicApp(App):
    CSS_PATH = "css/app.tcss"

    SCREENS = {
        "search": SearchScreen,
    }

    BINDINGS = [
        Binding("q", "quit", "Quit", priority=True),
        Binding("space", "toggle_playback", "Play/Pause"),
        Binding("n", "next_track", "Next"),
        Binding("p", "prev_track", "Prev"),
        Binding("+", "volume_up", "Vol+"),
        Binding("-", "volume_down", "Vol-"),
        Binding("s", "push_screen('search')", "Search"),
    ]

    current_track = reactive(None)
    playback_state = reactive("stopped")
    queue = reactive([])
    queue_index = reactive(-1)
    volume = reactive(100)
    position = reactive(0.0)
    duration = reactive(0.0)
    repeat_mode = reactive("off")
    shuffle = reactive(False)

    def __init__(self):
        super().__init__()
        self.api = YTMusicAPI()
        self.player = None

    def check_environment(self):
        try:
            import mpv
            p = mpv.MPV()
            p.terminate()
            del p
        except Exception:
            return False
        return True

    def on_mount(self):
        if not self.check_environment():
            self.notify(
                "libmpv not found.\n"
                "Install: sudo apt install libmpv1  or  brew install mpv",
                title="Missing Dependency",
                severity="error",
                timeout=15,
            )
            self.exit()
            return

        if self.api.authenticated:
            self._init_player()
            self.push_screen(MainScreen())
        else:
            self.push_screen(AuthScreen(self.api), callback=self._on_auth_done)

    def _on_auth_done(self, success):
        if success:
            self._init_player()
            self.push_screen(MainScreen())
        else:
            self.exit()

    def _init_player(self):
        self.player = MusicPlayer(
            on_track_end=self._on_track_end,
            on_error=self._on_player_error,
        )
        self.set_interval(0.5, self._sync_player_state)

    def _on_track_end(self):
        self.call_from_thread(self.next_track)

    def _on_player_error(self, event):
        self.call_from_thread(self.notify, "Playback error", severity="error")

    def _sync_player_state(self):
        if self.player and self.playback_state != "stopped":
            self.position = self.player.position
            self.duration = self.player.duration

    def play_track(self, track):
        self.queue = [track]
        self.queue_index = 0
        self._play_current()

    def play_playlist(self, tracks, index=0):
        self.queue = list(tracks)
        self.queue_index = index
        self._play_current()

    def add_to_queue(self, track):
        self.queue = list(self.queue) + [track]

    def _play_current(self):
        if 0 <= self.queue_index < len(self.queue):
            track = self.queue[self.queue_index]
            video_id = track.get("videoId")
            if video_id:
                self.player.play(video_id)
                self.current_track = track
                self.playback_state = "playing"
        else:
            self.playback_state = "stopped"

    def toggle_playback(self):
        if self.player is None:
            return
        if self.playback_state == "playing":
            self.player.pause()
            self.playback_state = "paused"
        elif self.playback_state == "paused":
            self.player.resume()
            self.playback_state = "playing"
        elif self.queue:
            self._play_current()

    def next_track(self):
        if not self.queue or self.queue_index < 0:
            return
        if self.shuffle:
            import random
            n = len(self.queue)
            if n > 1:
                offset = random.randint(1, n - 1)
                self.queue_index = (self.queue_index + offset) % n
            self._play_current()
        elif self.queue_index < len(self.queue) - 1:
            self.queue_index += 1
            self._play_current()
        elif self.repeat_mode == "all":
            self.queue_index = 0
            self._play_current()
        else:
            self.playback_state = "stopped"
            self.current_track = None
            if self.player:
                self.player.stop()

    def prev_track(self):
        if not self.queue or self.queue_index < 0:
            return
        if self.queue_index > 0:
            self.queue_index -= 1
            self._play_current()
        elif self.repeat_mode == "all":
            self.queue_index = len(self.queue) - 1
            self._play_current()

    def volume_up(self):
        self.volume = min(100, self.volume + 5)
        if self.player:
            self.player.volume = self.volume

    def volume_down(self):
        self.volume = max(0, self.volume - 5)
        if self.player:
            self.player.volume = self.volume

    def action_toggle_playback(self):
        self.toggle_playback()

    def action_next_track(self):
        self.next_track()

    def action_prev_track(self):
        self.prev_track()

    def action_volume_up(self):
        self.volume_up()

    def action_volume_down(self):
        self.volume_down()
