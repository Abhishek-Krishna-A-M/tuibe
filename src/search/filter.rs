use super::events::Track;

const GARBAGE_PATTERNS: &[&str] = &[
    "lyrics", "lyric", "slowed", "reverb", "nightcore", "8d audio",
    "8d music", "bass boosted", "cover", "reaction", "reacting to",
    "status", "edit", "fan made", "fanmade", "amv", "tutorial",
    "how to", "tribute", "vs ", " remix", "mashup", "speed up",
    "sped up", " slowed", "instrumental", "karaoke", "loop",
    "1 hour", "one hour", "10 hours", "extended",
];

fn contains_garbage_pattern(text: &str, query: &str) -> bool {
    let lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    for pattern in GARBAGE_PATTERNS {
        if !lower.contains(pattern) {
            continue;
        }
        // Skip filter if user explicitly searched for this pattern
        if query_lower.contains(pattern) {
            continue;
        }
        return true;
    }
    false
}

pub fn filter_tracks(tracks: Vec<Track>, query: &str) -> Vec<Track> {
    tracks
        .into_iter()
        .filter(|t| {
            let title_artist = format!("{} {}", t.title, t.artist);
            !contains_garbage_pattern(&title_artist, query)
        })
        .collect()
}
