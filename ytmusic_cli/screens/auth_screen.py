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

    #auth-code-box {
        width: 100%;
        height: auto;
        margin-top: 1;
        padding: 1;
        border: solid $accent;
    }

    #auth-code-box Label {
        text-style: bold;
    }

    #auth-code-url {
        text-style: bold;
        color: $accent;
        margin-top: 0;
    }

    #auth-code-text {
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
                "4. Enter the path below\n\n"
                "Or leave empty to use built-in credentials.",
                id="auth-desc",
            )
            yield Input(
                placeholder="Path to credentials JSON (or press Enter for default)",
                id="creds-path",
            )
            yield Button("Authorize", id="auth-submit", variant="primary")

    def on_button_pressed(self, event: Button.Pressed):
        if event.button.id == "auth-submit":
            path = self.query_one("#creds-path", Input).value.strip()
            self._start_oauth(path)

    def _show_error(self, msg):
        try:
            self.query_one("#auth-error", Label).update(msg)
        except Exception:
            self.query_one("#auth-container", Container).mount(
                Label(msg, id="auth-error")
            )

    def _show_status(self, msg):
        try:
            self.query_one("#auth-status", Label).update(msg)
        except Exception:
            self.query_one("#auth-container", Container).mount(
                Label(msg, id="auth-status")
            )

    def _show_code(self, url, user_code):
        try:
            self.query_one("#auth-code-url", Label).update(url)
            self.query_one("#auth-code-text", Label).update(f"Enter code: {user_code}")
        except Exception:
            self.query_one("#auth-container", Container).mount(
                Container(
                    Label("Open this URL and enter the code:", id="auth-code-label"),
                    Label(url, id="auth-code-url"),
                    Label(f"Code: {user_code}", id="auth-code-text"),
                    id="auth-code-box",
                )
            )

    @work(thread=True)
    def _start_oauth(self, path):
        from ytmusic_cli.api import DeviceCodeFlow

        client_id = None
        client_secret = None

        if path:
            import json

            try:
                with open(path) as f:
                    data = json.load(f)
                if "installed" in data:
                    data = data["installed"]
                elif "web" in data:
                    data = data["web"]
                client_id = data.get("client_id")
                client_secret = data.get("client_secret")
                if not client_id:
                    raise Exception("No client_id found in credentials file")
            except Exception as e:
                self.app.call_from_thread(
                    self._show_error, f"Failed to read credentials: {e}"
                )
                return

        flow = DeviceCodeFlow(client_id, client_secret)

        try:
            self.app.call_from_thread(self._show_status, "Requesting device code...")
            code_data = flow.get_code()
            self.app.call_from_thread(
                self._show_code, code_data["url"], code_data["user_code"]
            )
            self.app.call_from_thread(
                self._show_status,
                f"Waiting for authorization... (polls every {code_data['interval']}s)",
            )
            flow.wait_for_token(
                code_data["device_code"], interval=code_data["interval"]
            )
        except Exception as e:
            self.app.call_from_thread(self._show_error, f"OAuth failed: {e}")
            return

        self.app.call_from_thread(self._on_auth_success)

    def _on_auth_success(self):
        try:
            self._api._load_auth()
            self.dismiss(True)
        except Exception as e:
            self._show_error(f"Failed to load credentials: {e}")
