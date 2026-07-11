import locale
import os

os.environ["LC_NUMERIC"] = "C"
locale.setlocale(locale.LC_NUMERIC, "C")

from ytmusic_cli.app import YTMusicApp  # noqa: E402


def main():
    app = YTMusicApp()
    app.run()


if __name__ == "__main__":
    main()
