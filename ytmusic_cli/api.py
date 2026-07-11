import json
from pathlib import Path
from ytmusicapi import YTMusic

CONFIG_DIR = Path.home() / ".config" / "ytmusic-cli"
OAUTH_FILE = CONFIG_DIR / "oauth.json"


class YTMusicAPI:
    def __init__(self):
        self.yt = None
        self.authenticated = False
        if OAUTH_FILE.exists():
            self._load_auth()

    def _load_auth(self):
        try:
            self.yt = YTMusic(str(OAUTH_FILE))
            self.authenticated = True
        except Exception:
            self.yt = YTMusic()
            self.authenticated = False

    def search(self, query, filter=None, limit=20):
        kwargs = {"query": query, "limit": limit}
        if filter:
            kwargs["filter"] = filter
        return self.yt.search(**kwargs)

    def get_search_suggestions(self, query):
        return self.yt.get_search_suggestions(query)

    def get_library_playlists(self, limit=50):
        return self.yt.get_library_playlists(limit=limit)

    def get_library_songs(self, limit=50, order=None):
        kwargs = {"limit": limit}
        if order:
            kwargs["order"] = order
        return self.yt.get_library_songs(**kwargs)

    def get_library_albums(self, limit=50):
        return self.yt.get_library_albums(limit=limit)

    def get_liked_songs(self, limit=100):
        result = self.yt.get_liked_songs(limit=limit)
        return result.get("tracks", [])

    def get_playlist(self, playlist_id, limit=None):
        kwargs = {"playlistId": playlist_id}
        if limit is not None:
            kwargs["limit"] = limit
        return self.yt.get_playlist(**kwargs)

    def get_album(self, browse_id):
        return self.yt.get_album(browse_id)

    def get_song(self, video_id):
        return self.yt.get_song(video_id)

    def get_watch_playlist(self, video_id=None, playlist_id=None, radio=False):
        kwargs = {}
        if video_id:
            kwargs["videoId"] = video_id
        if playlist_id:
            kwargs["playlistId"] = playlist_id
        if radio:
            kwargs["radio"] = True
        return self.yt.get_watch_playlist(**kwargs)

    def rate_song(self, video_id, rating):
        return self.yt.rate_song(video_id, rating)

    def rate_playlist(self, playlist_id, rating):
        return self.yt.rate_playlist(playlist_id, rating)

    def get_history(self):
        return self.yt.get_history()

    def get_home(self, limit=10):
        return self.yt.get_home(limit=limit)

    def get_stream_url(self, video_id):
        sig_ts = self.yt.get_signatureTimestamp()
        try:
            song = self.yt.get_song(video_id, signatureTimestamp=sig_ts)
            formats = song.get("streamingData", {}).get("adaptiveFormats", [])
            audio = [f for f in formats if f.get("mimeType", "").startswith("audio/")]
            if audio:
                audio.sort(key=lambda f: int(f.get("bitrate", 0)), reverse=True)
                url = audio[0].get("url")
                if url:
                    return url
        except Exception:
            pass
        return None


class OAuthFlow:
    DEVICE_CODE_URL = "https://www.youtube.com/o/oauth2/device/code"
    TOKEN_URL = "https://oauth2.googleapis.com/token"
    SCOPE = "https://www.googleapis.com/auth/youtube"

    def __init__(self, client_id, client_secret):
        self.client_id = client_id
        import httpx
        self.http = httpx.Client()
        self._device_code = None
        self._user_code = None
        self._verification_url = None
        self._interval = 5

    def start(self):
        resp = self.http.post(
            self.DEVICE_CODE_URL,
            data={
                "client_id": self.client_id,
                "scope": self.SCOPE,
            },
        )
        data = resp.json()
        self._device_code = data["device_code"]
        self._user_code = data["user_code"]
        self._verification_url = data["verification_url"]
        self._interval = data.get("interval", 5)
        return self._verification_url, self._user_code

    def poll(self):
        resp = self.http.post(
            self.TOKEN_URL,
            data={
                "client_id": self.client_id,
                "code": self._device_code,
                "grant_type": "urn:ietf:params:oauth:grant-type:device_code",
            },
        )
        data = resp.json()
        if "access_token" in data:
            self._save_tokens(data)
            return True, None
        elif data.get("error") == "authorization_pending":
            return False, None
        elif data.get("error") == "slow_down":
            self._interval += 5
            return False, None
        else:
            return False, data.get("error", "unknown error")

    def _save_tokens(self, data):
        CONFIG_DIR.mkdir(parents=True, exist_ok=True)
        with open(OAUTH_FILE, "w") as f:
            json.dump(data, f, indent=2)

    @property
    def interval(self):
        return self._interval

    def close(self):
        self.http.close()
