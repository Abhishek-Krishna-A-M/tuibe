import json
import sys

from ytmusicapi import YTMusic

YT = None


def get_yt():
    global YT
    if YT is None:
        YT = YTMusic()
    return YT


def parse_duration(dur):
    if dur is None:
        return 0.0
    if isinstance(dur, (int, float)):
        return float(dur)
    if isinstance(dur, str):
        try:
            return float(dur)
        except ValueError:
            pass
        parts = dur.split(":")
        if len(parts) == 2:
            return int(parts[0]) * 60 + int(parts[1])
        elif len(parts) == 3:
            return int(parts[0]) * 3600 + int(parts[1]) * 60 + int(parts[2])
    return 0.0


def get_duration(item):
    dur = item.get("duration_seconds")
    if dur is not None:
        return float(dur)
    dur = item.get("duration")
    if dur is not None:
        return parse_duration(dur)
    dur = item.get("lengthSeconds")
    if dur is not None:
        return float(dur)
    return 0.0


def get_artist(item):
    artists = item.get("artists") or []
    if artists:
        return artists[0].get("name", "Unknown")
    return "Unknown"


def get_thumbnail_url(item):
    thumbs = item.get("thumbnails") or item.get("thumbnail") or []
    if isinstance(thumbs, str):
        return thumbs
    if isinstance(thumbs, list) and thumbs:
        return thumbs[-1].get("url") if isinstance(thumbs[-1], dict) else str(thumbs[-1])
    return None


def normalize(item, video_id_key="videoId"):
    vid = item.get(video_id_key) or ""
    return {
        "id": vid,
        "title": item.get("title", "Unknown"),
        "artist": get_artist(item),
        "duration": get_duration(item),
        "url": f"https://music.youtube.com/watch?v={vid}",
        "thumbnail": get_thumbnail_url(item),
    }


def cmd_search(args):
    query = " ".join(args) if args else ""
    if not query:
        print(json.dumps([]))
        return

    yt = get_yt()

    results = yt.search(query, limit=50)

    songs_filtered = yt.search(query, filter="songs", limit=50)
    dur_map = {}
    for r in songs_filtered:
        vid = r.get("videoId") or ""
        if vid:
            dur_map[vid] = get_duration(r)

    tracks = []
    for r in results:
        rt = r.get("resultType", "")
        if rt not in ("song", "video"):
            continue
        vid = r.get("videoId") or ""
        if not vid:
            continue
        track = normalize(r)
        if track["duration"] == 0.0 and vid in dur_map:
            track["duration"] = dur_map[vid]
        tracks.append(track)

    print(json.dumps(tracks))


def cmd_related(args):
    video_id = args[0] if args else ""
    if not video_id:
        print(json.dumps([]))
        return

    yt = get_yt()
    playlist = yt.get_watch_playlist(videoId=video_id, limit=50, radio=True)

    tracks = []
    for r in playlist.get("tracks", []):
        vid = r.get("videoId") or ""
        if not vid or vid == video_id:
            continue
        tracks.append(normalize(r, video_id_key="videoId"))

    print(json.dumps(tracks))


def cmd_playlist(args):
    raw = args[0] if args else ""
    if not raw:
        print(json.dumps({"name": "", "tracks": []}))
        return

    playlist_id = raw

    if "?" in raw and "list=" in raw:
        from urllib.parse import parse_qs, urlparse
        parsed = urlparse(raw)
        qs = parse_qs(parsed.query)
        playlist_id = qs.get("list", [raw])[0]
    elif "/playlist/" in raw:
        playlist_id = raw.rstrip("/").split("/playlist/")[-1]
        if "?" in playlist_id:
            playlist_id = playlist_id.split("?")[0]

    yt = get_yt()
    pl = yt.get_playlist(playlist_id, limit=None)

    tracks = []
    for r in pl.get("tracks", []):
        tracks.append(normalize(r))

    result = {
        "id": playlist_id,
        "name": pl.get("title", "Imported Playlist"),
        "tracks": tracks,
    }
    print(json.dumps(result))


def main():
    if len(sys.argv) < 2:
        print(json.dumps([]), file=sys.stderr)
        sys.exit(1)

    cmd = sys.argv[1]
    args = sys.argv[2:]

    if cmd == "search":
        cmd_search(args)
    elif cmd == "related":
        cmd_related(args)
    elif cmd == "playlist":
        cmd_playlist(args)
    else:
        print(json.dumps([]), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
