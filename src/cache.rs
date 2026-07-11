use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AudioCache {
    cache_dir: PathBuf,
    max_bytes: u64,
    index: HashMap<String, u64>,
}

impl AudioCache {
    pub fn new(max_mb: u64) -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("tuibe");
        let index = Self::load_index(&cache_dir);
        let mut cache = Self {
            cache_dir,
            max_bytes: max_mb * 1024 * 1024,
            index,
        };
        cache.evict_if_needed();
        cache
    }

    fn index_path(&self) -> PathBuf {
        self.cache_dir.join("index.json")
    }

    fn load_index(dir: &Path) -> HashMap<String, u64> {
        let path = dir.join("index.json");
        if path.exists()
            && let Ok(content) = fs::read_to_string(&path)
            && let Ok(idx) = serde_json::from_str(&content)
        {
            return idx;
        }
        HashMap::new()
    }

    fn save_index(&self) {
        fs::create_dir_all(&self.cache_dir).ok();
        if let Ok(content) = serde_json::to_string(&self.index) {
            let _ = fs::write(self.index_path(), content);
        }
    }

    pub fn cache_path_for(&self, track_id: &str) -> PathBuf {
        // Sanitize track_id for filesystem use
        let safe: String = track_id
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        self.cache_dir.join(format!("{}.m4a", safe))
    }

    pub fn is_cached(&self, track_id: &str) -> bool {
        self.cache_path_for(track_id).exists()
    }

    pub fn open_cached(&self, track_id: &str) -> Option<CachedFile> {
        let path = self.cache_path_for(track_id);
        if path.exists() {
            let file = fs::File::open(&path).ok()?;
            let len = file.metadata().ok()?.len();
            Some(CachedFile { inner: file, len })
        } else {
            None
        }
    }

    pub fn touch(&mut self, track_id: &str) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.index.insert(track_id.to_string(), now);
        self.save_index();
    }

    pub fn put(&mut self, track_id: &str, data: &[u8]) {
        fs::create_dir_all(&self.cache_dir).ok();
        let path = self.cache_path_for(track_id);
        if fs::write(&path, data).is_ok() {
            self.touch(track_id);
            self.evict_if_needed();
        }
    }

    fn current_size(&self) -> u64 {
        let mut total = 0;
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "m4a") {
                    if let Ok(meta) = entry.metadata() {
                        total += meta.len();
                    }
                }
            }
        }
        total
    }

    fn evict_if_needed(&mut self) {
        loop {
            let size = self.current_size();
            if size <= self.max_bytes {
                break;
            }
            let oldest_id = self
                .index
                .iter()
                .min_by_key(|&(_, time)| *time)
                .map(|(id, _)| id.clone());

            if let Some(id) = oldest_id {
                let path = self.cache_path_for(&id);
                let _ = fs::remove_file(&path);
                self.index.remove(&id);
            } else {
                break;
            }
        }
        self.save_index();
    }
}

pub struct CachedFile {
    inner: fs::File,
    len: u64,
}

impl Read for CachedFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for CachedFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}
