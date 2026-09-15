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


def normalize_artist(item):
    return {
        "id": item.get("browseId") or "",
        "name": item.get("artist") or item.get("title") or "Unknown",
        "subscribers": item.get("subscribers"),
        "thumbnail": get_thumbnail_url(item),
    }


def normalize_album(item):
    return {
        "id": item.get("browseId") or "",
        "title": item.get("title") or "Unknown",
        "artist": get_artist(item),
        "year": str(item.get("year")) if item.get("year") is not None else None,
        "thumbnail": get_thumbnail_url(item),
    }


def cmd_search(args):
    # New: `search <filter> <query...>` where filter is
    # songs|artists|albums. Old callers passed only `<query...>`;
    # keep that working by defaulting to songs.
    filt = "songs"
    query_parts = args
    if args and args[0] in ("songs", "artists", "albums"):
        filt = args[0]
        query_parts = args[1:]
    query = " ".join(query_parts) if query_parts else ""
    if not query:
        print(json.dumps([]))
        return

    yt = get_yt()

    if filt == "artists":
        results = yt.search(query, filter="artists", limit=25)
        out = []
        for r in results:
            if r.get("resultType", "") != "artist":
                continue
            if not (r.get("browseId") or ""):
                continue
            out.append(normalize_artist(r))
        print(json.dumps(out))
        return

    if filt == "albums":
        results = yt.search(query, filter="albums", limit=25)
        out = []
        for r in results:
            if r.get("resultType", "") != "album":
                continue
            if not (r.get("browseId") or ""):
                continue
            out.append(normalize_album(r))
        print(json.dumps(out))
        return

    # filt == "songs": original behaviour (songs + videos)
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


def cmd_artist(args):
    browse_id = args[0] if args else ""
    if not browse_id:
        print(json.dumps([]))
        return

    yt = get_yt()
    data = yt.get_artist(browse_id)

    songs = (data.get("songs") or {}).get("results") or []
    tracks = []
    for r in songs:
        vid = r.get("videoId") or ""
        if not vid:
            continue
        tracks.append(normalize(r))
    print(json.dumps(tracks))


def cmd_album(args):
    browse_id = args[0] if args else ""
    if not browse_id:
        print(json.dumps([]))
        return

    yt = get_yt()
    data = yt.get_album(browse_id)

    tracks = []
    for r in data.get("tracks") or []:
        vid = r.get("videoId") or ""
        if not vid:
            continue
        track = normalize(r)
        # Albums often omit the artist per-track; fall back to album artist.
        if track["artist"] in ("", "Unknown"):
            track["artist"] = get_artist(data)
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
    elif cmd == "artist":
        cmd_artist(args)
    elif cmd == "album":
        cmd_album(args)
    elif cmd == "related":
        cmd_related(args)
    elif cmd == "playlist":
        cmd_playlist(args)
    else:
        print(json.dumps([]), file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
