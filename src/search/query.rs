use serde::{Deserialize, Serialize};

/// What kind of entity a search targets.
/// `movie:` is an alias of `Albums` — on YouTube Music, movie
/// soundtracks show up as albums.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
pub enum SearchScope {
    #[default]
    Songs,
    Artists,
    Albums,
}

impl SearchScope {
    pub fn label(self) -> &'static str {
        match self {
            SearchScope::Songs => "songs",
            SearchScope::Artists => "artists",
            SearchScope::Albums => "albums",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            SearchScope::Songs => "song",
            SearchScope::Artists => "artist",
            SearchScope::Albums => "album",
        }
    }

    /// Filter string passed to `ytmusicapi` (`songs` / `artists` / `albums`).
    pub fn api_filter(self) -> &'static str {
        self.label()
    }
}

/// Parse `artist:Anirudh`, `album:Leo`, `movie:Vikram`, `song:Master`, …
/// Returns `(scope, clean_query)`.
///
/// Rules:
/// - prefix match is case-insensitive, whitespace around `:` is allowed
///   (`"artist : arr"` works);
/// - `movie:` / `movies:` / `album:` / `albums:` all map to `Albums`;
/// - `artist:` / `artists:` map to `Artists`;
/// - `song:` / `songs:` / `track:` / `video:` map to `Songs`;
/// - bare text (or unknown prefix, or empty value) falls back to `Songs`
///   with the *original* text so we never drop the user's query.
pub fn parse_query(raw: &str) -> (SearchScope, String) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (SearchScope::Songs, String::new());
    }

    if let Some((prefix, value)) = split_prefix(trimmed) {
        let scope = match prefix {
            "artist" | "artists" => Some(SearchScope::Artists),
            "album" | "albums" | "movie" | "movies" => Some(SearchScope::Albums),
            "song" | "songs" | "track" | "tracks" | "video" | "videos" => {
                Some(SearchScope::Songs)
            }
            _ => None,
        };
        if let Some(scope) = scope {
            let clean = value.trim().to_string();
            if !clean.is_empty() {
                return (scope, clean);
            }
            // `artist:` with nothing after it — fall through to Songs
            // with the original text so the user still searches something.
        }
    }

    (SearchScope::Songs, trimmed.to_string())
}

/// Split `prefix:value` on the first `:`. Returns lowercased trimmed
/// prefix + raw value. Returns `None` when there is no `:`.
fn split_prefix(s: &str) -> Option<(&'static str, &str)> {
    let idx = s.find(':')?;
    let (raw_prefix, value) = s.split_at(idx);
    let value = &value[1..]; // skip ':'
    let normalized = raw_prefix.trim().to_lowercase();
    // Leak a tiny string so we can return &'static str without allocation
    // churn on the hot path — prefixes are a small closed set.
    let leaked: &'static str = match normalized.as_str() {
        "artist" => "artist",
        "artists" => "artists",
        "album" => "album",
        "albums" => "albums",
        "movie" => "movie",
        "movies" => "movies",
        "song" => "song",
        "songs" => "songs",
        "track" => "track",
        "tracks" => "tracks",
        "video" => "video",
        "videos" => "videos",
        _ => return None,
    };
    Some((leaked, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_query_is_songs() {
        assert_eq!(
            parse_query("Anirudh"),
            (SearchScope::Songs, "Anirudh".to_string())
        );
    }

    #[test]
    fn artist_prefix() {
        assert_eq!(
            parse_query("artist:Anirudh"),
            (SearchScope::Artists, "Anirudh".to_string())
        );
        assert_eq!(
            parse_query("ARTIST:  A.R. Rahman "),
            (SearchScope::Artists, "A.R. Rahman".to_string())
        );
        assert_eq!(
            parse_query("artists : Ilaiyaraaja"),
            (SearchScope::Artists, "Ilaiyaraaja".to_string())
        );
    }

    #[test]
    fn album_and_movie_alias() {
        assert_eq!(
            parse_query("album:Leo"),
            (SearchScope::Albums, "Leo".to_string())
        );
        assert_eq!(
            parse_query("movie:Vikram"),
            (SearchScope::Albums, "Vikram".to_string())
        );
        assert_eq!(
            parse_query("Movies: Jailer"),
            (SearchScope::Albums, "Jailer".to_string())
        );
    }

    #[test]
    fn song_prefix_and_unknown_fallback() {
        assert_eq!(
            parse_query("song:Master"),
            (SearchScope::Songs, "Master".to_string())
        );
        // Unknown prefix keeps the whole text as a song query.
        assert_eq!(
            parse_query("foo:bar"),
            (SearchScope::Songs, "foo:bar".to_string())
        );
        // Empty value falls back to the raw text.
        assert_eq!(
            parse_query("artist:"),
            (SearchScope::Songs, "artist:".to_string())
        );
        assert_eq!(parse_query("   "), (SearchScope::Songs, String::new()));
    }
}
