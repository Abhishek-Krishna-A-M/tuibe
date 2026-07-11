from textual.widgets import DataTable


class TrackTable(DataTable):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.add_columns("#", "Title", "Artist", "Album", "Duration")
        self.cursor_type = "row"
        self.zebra_stripes = True
        self._tracks = []

    def set_tracks(self, tracks):
        self._tracks = list(tracks)
        self.clear()
        for i, track in enumerate(tracks, 1):
            artists = track.get("artists", [])
            artist = artists[0].get("name", "") if artists else ""
            album = track.get("album", {})
            album_name = album.get("name", "") if isinstance(album, dict) else ""
            duration = track.get("duration", "")
            self.add_row(
                str(i),
                track.get("title", ""),
                artist,
                album_name,
                duration,
            )
