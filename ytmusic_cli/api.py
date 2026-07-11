import json
import locale as locale_mod
import time
from pathlib import Path
from ytmusicapi import YTMusic
from ytmusicapi.auth.oauth.credentials import OAuthCredentials

CONFIG_DIR = Path.home() / ".config" / "ytmusic-cli"
OAUTH_FILE = CONFIG_DIR / "oauth.json"

DEFAULT_CLIENT_ID = "861556708454-d6dlm3lh05idd8npek18k6be8ba3oc68.apps.googleusercontent.com"
DEFAULT_CLIENT_SECRET = "SboVhoG9s0rNafixCSGGKXAT"


def _with_locale(fn):
    try:
        saved = locale_mod.setlocale(locale_mod.LC_NUMERIC, None)
    except locale_mod.Error:
        saved = "C"
    try:
        locale_mod.setlocale(locale_mod.LC_NUMERIC, "C")
    except locale_mod.Error:
        pass
    try:
        return fn()
    finally:
        try:
            locale_mod.setlocale(locale_mod.LC_NUMERIC, saved)
        except locale_mod.Error:
            pass


class YTMusicAPI:
    def __init__(self):
        self.yt = None
        self.authenticated = False
        self._oauth_credentials = None
        if OAUTH_FILE.exists():
            self._load_auth()

    def _load_auth(self):
        self._oauth_credentials = self._load_oauth_credentials()

        def _load():
            if self._oauth_credentials:
                self.yt = YTMusic(
                    str(OAUTH_FILE), oauth_credentials=self._oauth_credentials
                )
            else:
                self.yt = YTMusic(str(OAUTH_FILE))
            self._ensure_api_key()
            self.authenticated = True

        try:
            _with_locale(_load)
            if self.authenticated:
                return
        except Exception:
            self.authenticated = False
            self.yt = None

        def _fallback():
            self.yt = YTMusic()
            self.authenticated = False

        try:
            _with_locale(_fallback)
        except Exception:
            self.authenticated = False

    def _load_oauth_credentials(self):
        if not OAUTH_FILE.exists():
            return None
        try:
            with open(OAUTH_FILE) as f:
                data = json.load(f)
            client_id = data.get("_client_id") or data.get("client_id", DEFAULT_CLIENT_ID)
            client_secret = data.get("_client_secret") or data.get("client_secret", DEFAULT_CLIENT_SECRET)
            return OAuthCredentials(client_id, client_secret)
        except Exception:
            return None

    def _ensure_api_key(self):
        from ytmusicapi.ytmusic import YTM_PARAMS_KEY

        if self.yt and YTM_PARAMS_KEY not in self.yt.params:
            self.yt.params += YTM_PARAMS_KEY

    def search(self, query, filter=None, limit=20):
        return _with_locale(lambda: self.yt.search(query=query, limit=limit, filter=filter))

    def get_search_suggestions(self, query):
        return _with_locale(lambda: self.yt.get_search_suggestions(query))

    def get_library_playlists(self, limit=50):
        return _with_locale(lambda: self.yt.get_library_playlists(limit=limit))

    def get_library_songs(self, limit=50, order=None):
        return _with_locale(lambda: self.yt.get_library_songs(limit=limit, order=order))

    def get_library_albums(self, limit=50):
        return _with_locale(lambda: self.yt.get_library_albums(limit=limit))

    def get_liked_songs(self, limit=100):
        result = _with_locale(lambda: self.yt.get_liked_songs(limit=limit))
        return result.get("tracks", [])

    def get_playlist(self, playlist_id, limit=None):
        return _with_locale(lambda: self.yt.get_playlist(playlistId=playlist_id, limit=limit))

    def get_album(self, browse_id):
        return _with_locale(lambda: self.yt.get_album(browse_id))

    def get_song(self, video_id):
        return _with_locale(lambda: self.yt.get_song(video_id))

    def get_watch_playlist(self, video_id=None, playlist_id=None, radio=False):
        return _with_locale(
            lambda: self.yt.get_watch_playlist(
                videoId=video_id, playlistId=playlist_id, radio=radio
            )
        )

    def rate_song(self, video_id, rating):
        return _with_locale(lambda: self.yt.rate_song(video_id, rating))

    def rate_playlist(self, playlist_id, rating):
        return _with_locale(lambda: self.yt.rate_playlist(playlist_id, rating))

    def get_history(self):
        return _with_locale(lambda: self.yt.get_history())

    def get_home(self, limit=10):
        return _with_locale(lambda: self.yt.get_home(limit=limit))

    def get_stream_url(self, video_id):
        def _get():
            try:
                song = self.yt.get_song(video_id)
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

        return _with_locale(_get)


class DeviceCodeFlow:
    """ytmusicapi device code OAuth flow — same as ``ytmusicapi.setup_oauth``."""

    def __init__(self, client_id=None, client_secret=None):
        self.client_id = client_id or DEFAULT_CLIENT_ID
        self.client_secret = client_secret or DEFAULT_CLIENT_SECRET
        self._credentials = OAuthCredentials(self.client_id, self.client_secret)

    def get_code(self):
        code = self._credentials.get_code()
        url = f"{code['verification_url']}?user_code={code['user_code']}"
        return {
            "device_code": code["device_code"],
            "url": url,
            "user_code": code["user_code"],
            "interval": code.get("interval", 5),
        }

    def wait_for_token(self, device_code, interval=5, timeout=300):
        import time as _time

        elapsed = 0
        while elapsed < timeout:
            try:
                raw_token = self._credentials.token_from_code(device_code)
                if "access_token" in raw_token:
                    return self._save_token(raw_token)
            except Exception as e:
                error_msg = str(e).lower()
                if "authorization_pending" in error_msg:
                    pass
                elif "slow_down" in error_msg:
                    interval += 1
                elif "expired_token" in error_msg:
                    raise Exception("Device code expired. Please try again.")
                else:
                    raise
            _time.sleep(interval)
            elapsed += interval
        raise TimeoutError("Authorization timed out")

    def _save_token(self, raw_token):
        CONFIG_DIR.mkdir(parents=True, exist_ok=True)
        refresh_token_expires_in = raw_token.get(
            "refresh_token_expires_in", raw_token.get("expires_in", 3600)
        )
        token_data = {
            "access_token": raw_token["access_token"],
            "refresh_token": raw_token["refresh_token"],
            "scope": raw_token.get("scope", "https://www.googleapis.com/auth/youtube"),
            "token_type": raw_token.get("token_type", "Bearer"),
            "expires_in": refresh_token_expires_in,
            "expires_at": int(time.time()) + refresh_token_expires_in,
            "_client_id": self.client_id,
            "_client_secret": self.client_secret,
        }
        with open(OAUTH_FILE, "w") as f:
            json.dump(token_data, f, indent=2)
        return True
