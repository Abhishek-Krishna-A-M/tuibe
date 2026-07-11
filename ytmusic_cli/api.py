import json
from pathlib import Path
from ytmusicapi import YTMusic

CONFIG_DIR = Path.home() / ".config" / "ytmusic-cli"
OAUTH_FILE = CONFIG_DIR / "oauth.json"


def _with_locale(fn):
    import locale
    saved = locale.setlocale(locale.LC_NUMERIC, None)
    try:
        return fn()
    finally:
        locale.setlocale(locale.LC_NUMERIC, saved)


class YTMusicAPI:
    def __init__(self):
        self.yt = None
        self.authenticated = False
        if OAUTH_FILE.exists():
            self._load_auth()

    def _load_auth(self):
        def _load():
            self.yt = YTMusic(str(OAUTH_FILE))
            self.authenticated = True
        try:
            _with_locale(_load)
            return
        except Exception:
            pass
        def _fallback():
            self.yt = YTMusic()
            self.authenticated = False
        _with_locale(_fallback)

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
        return _with_locale(lambda: self.yt.get_watch_playlist(videoId=video_id, playlistId=playlist_id, radio=radio))

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
        return _with_locale(_get)


class BrowserOAuthFlow:
    AUTH_URL = "https://accounts.google.com/o/oauth2/auth"
    TOKEN_URL = "https://oauth2.googleapis.com/token"
    SCOPE = "https://www.googleapis.com/auth/youtube"

    def __init__(self, client_id, client_secret):
        self.client_id = client_id
        self.client_secret = client_secret
        self._port = None
        self._auth_code = []
        self._server = None
        self._server_thread = None

    def start(self):
        from http.server import HTTPServer, BaseHTTPRequestHandler
        from urllib.parse import urlencode
        import socket

        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.bind(("localhost", 0))
        self._port = sock.getsockname()[1]
        sock.close()

        redirect_uri = f"http://localhost:{self._port}/"
        params = urlencode({
            "client_id": self.client_id,
            "redirect_uri": redirect_uri,
            "response_type": "code",
            "scope": self.SCOPE,
            "access_type": "offline",
            "prompt": "consent",
        })
        auth_url = f"{self.AUTH_URL}?{params}"

        class Handler(BaseHTTPRequestHandler):
            def do_GET(self):
                from urllib.parse import urlparse, parse_qs
                p = parse_qs(urlparse(self.path).query)
                code_list = p.get("code", [])
                if code_list:
                    self.server.auth_code.append(code_list[0])
                    self.send_response(200)
                    self.send_header("Content-type", "text/html")
                    self.end_headers()
                    self.wfile.write(b"<html><body><h1>Authorized!</h1><p>Close this window.</p></body></html>")
                else:
                    self.send_response(400)
                    self.end_headers()
                    self.wfile.write(b"<html><body><h1>Error</h1></body></html>")

            def log_message(self, fmt, *args):
                pass

        self._server = HTTPServer(("localhost", self._port), Handler)
        self._server.auth_code = self._auth_code

        import threading
        self._server_thread = threading.Thread(target=self._server.serve_forever)
        self._server_thread.daemon = True
        self._server_thread.start()

        import webbrowser
        try:
            webbrowser.open(auth_url)
        except Exception:
            pass

        return auth_url

    def wait_for_code(self, timeout=300):
        import time
        for _ in range(timeout * 2):
            if self._auth_code:
                return self._auth_code[0]
            time.sleep(0.5)
        raise TimeoutError("Authorization timed out")

    def exchange(self, code):
        import httpx
        resp = httpx.post(
            self.TOKEN_URL,
            data={
                "code": code,
                "client_id": self.client_id,
                "client_secret": self.client_secret,
                "redirect_uri": f"http://localhost:{self._port}/",
                "grant_type": "authorization_code",
            },
        )
        data = resp.json()
        if "access_token" in data:
            self._save_tokens(data)
            return True
        raise Exception(data.get("error_description", data.get("error", "unknown error")))

    def _save_tokens(self, data):
        import time
        CONFIG_DIR.mkdir(parents=True, exist_ok=True)
        data["expires_at"] = time.time() + data.get("expires_in", 3600)
        data["_client_id"] = self.client_id
        data["_client_secret"] = self.client_secret
        with open(OAUTH_FILE, "w") as f:
            json.dump(data, f, indent=2)

    def close(self):
        if self._server:
            self._server.shutdown()
