from textual.app import ComposeResult
from textual.screen import ModalScreen
from textual.widgets import Button, Input, Label, LoadingIndicator
from textual.containers import Container
from textual import work


class AuthScreen(ModalScreen[bool]):
    CSS = """
    AuthScreen {
        align: center middle;
    }

    #auth-container {
        width: 60;
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

    #auth-success {
        color: $success;
        height: auto;
        margin-top: 1;
    }

    #oauth-code-screen {
        align: center middle;
    }

    #oauth-url {
        text-style: bold;
        color: $accent;
    }

    #oauth-code {
        text-style: bold;
        color: $success;
        height: 3;
        content-align: center middle;
    }
    """

    def __init__(self, api):
        super().__init__()
        self._api = api
        self._flow = None

    def compose(self) -> ComposeResult:
        with Container(id="auth-container"):
            yield Label("YouTube Music - Setup", id="auth-title")
            yield Label(
                "To access your YouTube Music library, you need to authenticate.\n"
                "Create a Google Cloud OAuth client to get credentials:\n"
                "1. Go to https://console.cloud.google.com/apis/credentials\n"
                "2. Create OAuth 2.0 Client ID (Desktop app type)\n"
                "3. Copy the Client ID and Client Secret below",
                id="auth-desc",
            )
            yield Input(
                placeholder="Client ID",
                id="client-id",
                classes="auth-input",
            )
            yield Input(
                placeholder="Client Secret",
                id="client-secret",
                password=True,
                classes="auth-input",
            )
            yield Button("Authorize", id="auth-submit", variant="primary")

    def on_button_pressed(self, event: Button.Pressed):
        if event.button.id == "auth-submit":
            client_id = self.query_one("#client-id", Input).value.strip()
            client_secret = self.query_one("#client-secret", Input).value.strip()
            if not client_id or not client_secret:
                self._show_error("Please fill in both fields")
                return
            self._start_oauth(client_id, client_secret)

    @work(thread=True)
    def _start_oauth(self, client_id, client_secret):
        from ytmusic_cli.api import OAuthFlow

        self._flow = OAuthFlow(client_id, client_secret)

        try:
            url, code = self._flow.start()
        except Exception as e:
            self.app.call_from_thread(self._show_error, f"Failed to start OAuth: {e}")
            return

        self.app.call_from_thread(self._show_oauth_instructions, url, code)

        while True:
            import time
            time.sleep(self._flow.interval)
            success, error = self._flow.poll()
            if success:
                self.app.call_from_thread(self._on_auth_success)
                return
            elif error:
                self.app.call_from_thread(self._show_error, f"OAuth error: {error}")
                return

    def _show_oauth_instructions(self, url, code):
        self.query_one("#auth-container").remove()
        self.mount(
            Container(
                Label("Authorize YouTube Music", id="auth-title"),
                Label("Visit this URL in your browser:", id="oauth-desc"),
                Label(url, id="oauth-url"),
                Label("And enter this code:", id="oauth-code-label"),
                Label(code, id="oauth-code"),
                LoadingIndicator(),
                id="oauth-code-screen",
            )
        )

    def _show_error(self, msg):
        try:
            existing = self.query_one("#auth-error", Label)
            existing.update(msg)
        except Exception:
            self.mount(
                Label(msg, id="auth-error"),
                before=0,
            )

    def _on_auth_success(self):
        try:
            self._api._load_auth()
            self.dismiss(True)
        except Exception as e:
            self._show_error(f"Failed to load credentials: {e}")
