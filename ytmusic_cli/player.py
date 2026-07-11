import mpv


class MusicPlayer:
    def __init__(self, on_track_end=None, on_error=None):
        self.player = mpv.MPV(
            ytdl=True,
            ytdl_format="bestaudio[ext=m4a]/bestaudio",
            volume=100,
            cache_secs=30,
            terminal=False,
        )
        self._position = 0.0
        self._duration = 0.0

        @self.player.property_observer("time-pos")
        def _on_time_pos(_name, value):
            self._position = value if value is not None else 0.0

        @self.player.property_observer("duration")
        def _on_duration(_name, value):
            self._duration = value if value is not None else 0.0

        if on_track_end:

            @self.player.property_observer("eof-reached")
            def _on_eof(_name, value):
                if value:
                    on_track_end()

        if on_error:
            self.player.register_event_handler("file-error", lambda e: on_error(e))

    def play(self, url_or_video_id):
        if not url_or_video_id.startswith("http"):
            url_or_video_id = f"https://music.youtube.com/watch?v={url_or_video_id}"
        self.player.play(url_or_video_id)

    def pause(self):
        self.player.pause = True

    def resume(self):
        self.player.pause = False

    def toggle_pause(self):
        self.player.cycle("pause")

    def stop(self):
        self.player.stop()

    def seek(self, position_seconds):
        self.player.seek(position_seconds, reference="absolute")

    def seek_relative(self, delta_seconds):
        self.player.seek(delta_seconds)

    @property
    def volume(self):
        return self.player.volume

    @volume.setter
    def volume(self, value):
        self.player.volume = max(0, min(100, int(value)))

    @property
    def position(self):
        return self._position

    @property
    def duration(self):
        return self._duration

    @property
    def is_playing(self):
        return not self.player.pause

    @property
    def is_idle(self):
        return self.player.core_idle

    @property
    def metadata(self):
        return self.player.metadata or {}

    def quit(self):
        try:
            self.player.terminate()
        except Exception:
            pass
