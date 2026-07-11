from textual.app import ComposeResult
from textual.screen import ModalScreen
from textual.widgets import Button, Input, Label
from textual.containers import Container
from textual import work


class AuthScreen(ModalScreen[bool]):
    CSS = """
    AuthScreen {
        align: center middle;
    }

    #auth-container {
        width: 72;
        height: auto;
        padding: 1 2;
        border: thick $primary;
        background: $surface;
    }

    #auth-title {
        text-style: bold;
        content-align: center middle;
        height: 3;
    }

    #auth-desc {
        height: auto;
        margin-bottom: 1;
    }

    .auth-input {
        margin-bottom: 1;
    }

    #auth-submit {
        width: 100%;
        margin-top: 1;
    }

    #auth-error {
        color: $error;
        height: auto;
        margin-top: 1;
    }

    #auth-status {
        height: auto;
        margin-top: 1;
    }

    #auth-url-box {
        width: 100%;
        height: auto;
        margin-top: 1;
        padding: 1;
        border: solid $accent;
    }

    #auth-url-box Label {
        text-style: bold;
    }

    #auth-url-text {
        text-style: bold;
        color: $accent;
        margin-top: 1;
    }
    """

    def __init__(self, api):
        super().__init__()
        self._api = api

    def compose(self) -> ComposeResult:
        with Container(id="auth-container"):
            yield Label("YouTube Music - Setup", id="auth-title")
            yield Label(
                "Authenticate with your Google account.\n"
                "1. Go to https://console.cloud.google.com/apis/credentials\n"
                "2. Create OAuth 2.0 Client ID (Desktop app type)\n"
                "3. Download the JSON file\n"
                "4. Enter the path below",
                id="auth-desc",
            )
            yield Input(
                placeholder="Path to credentials JSON (e.g. ./auth.json)",
                id="creds-path",
                classes="auth-input",
            )
            yield Button("Authorize", id="auth-submit", variant="primary")

    def on_button_pressed(self, event: Button.Pressed):
        if event.button.id == "auth-submit":
            path = self.query_one("#creds-path", Input).value.strip()
            if not path:
                self._show_error("Please enter the path to your credentials file")
                return
            self._start_oauth(path)

    def _show_error(self, msg):
        try:
            existing = self.query_one("#auth-error", Label)
            existing.update(msg)
        except Exception:
            self.mount(Label(msg, id="auth-error"), before=0)

    def _show_status(self, msg):
        try:
            existing = self.query_one("#auth-status", Label)
            existing.update(msg)
        except Exception:
            self.mount(Label(msg, id="auth-status"))

    def _show_auth_url(self, url):
        try:
            self.query_one("#auth-url-text", Label).update(url)
        except Exception:
            self.mount(
                Container(
                    Label("If browser didn't open, visit:"),
                    Label(url, id="auth-url-text"),
                    id="auth-url-box",
                )
            )

    @work(thread=True)
    def _start_oauth(self, path):
        import json

        try:
            with open(path) as f:
                data = json.load(f)
            if "installed" in data:
                data = data["installed"]
            elif "web" in data:
                data = data["web"]
            client_id = data.get("client_id", "")
            client_secret = data.get("client_secret", "")
            if not client_id:
                raise Exception("No client_id found in credentials file")
        except Exception as e:
            self.app.call_from_thread(self._show_error, f"Failed to read credentials: {e}")
            return

        from ytmusic_cli.api import BrowserOAuthFlow

        flow = BrowserOAuthFlow(client_id, client_secret)

        try:
            self.app.call_from_thread(self._show_status, "Starting browser for authorization...")
            auth_url = flow.start()
            self.app.call_from_thread(self._show_auth_url, auth_url)
            self.app.call_from_thread(self._show_status, "Waiting for authorization in browser...")
            code = flow.wait_for_code()
            self.app.call_from_thread(self._show_status, "Exchanging code for tokens...")
            flow.exchange(code)
        except Exception as e:
            self.app.call_from_thread(self._show_error, f"OAuth failed: {e}")
            return
        finally:
            flow.close()

        self.app.call_from_thread(self._on_auth_success)

    def _on_auth_success(self):
        try:
            self._api._load_auth()
            self.dismiss(True)
        except Exception as e:
            self._show_error(f"Failed to load credentials: {e}")
