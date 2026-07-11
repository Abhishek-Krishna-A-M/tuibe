use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

const MAX_STREAM_BYTES: u64 = 200 * 1024 * 1024;

pub struct StreamWriter {
    file: File,
    bytes_written: Arc<AtomicU64>,
    finished: Arc<AtomicBool>,
    cvar: Arc<Condvar>,
}

impl Write for StreamWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let current = self.bytes_written.load(Ordering::Relaxed);
        if current >= MAX_STREAM_BYTES {
            return Ok(0);
        }
        let remaining = (MAX_STREAM_BYTES - current) as usize;
        let to_write = buf.len().min(remaining);
        let written = self.file.write(&buf[..to_write])?;
        self.bytes_written.fetch_add(written as u64, Ordering::Relaxed);
        self.cvar.notify_all();
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl StreamWriter {
    pub fn finish(&self) {
        self.finished.store(true, Ordering::Release);
        self.cvar.notify_all();
    }

    pub fn set_error(&self, msg: String) {
        eprintln!("Stream error: {}", msg);
        self.finished.store(true, Ordering::Release);
        self.cvar.notify_all();
    }
}

pub struct StreamHandle {
    file: File,
    bytes_written: Arc<AtomicU64>,
    finished: Arc<AtomicBool>,
    cvar: Arc<Condvar>,
    lock: Arc<Mutex<()>>,
    path: String,
    delete_on_drop: bool,
}

impl Drop for StreamHandle {
    fn drop(&mut self) {
        if self.delete_on_drop {
            let _ = fs::remove_file(&self.path);
        }
    }
}

impl Read for StreamHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let pos = self.file.stream_position()?;
            let available = self.bytes_written.load(Ordering::Acquire);

            if pos < available {
                return self.file.read(buf);
            }

            if self.finished.load(Ordering::Acquire) {
                return self.file.read(buf);
            }

            let guard = self.lock.lock().unwrap();
            let _ = self.cvar.wait_timeout(guard, std::time::Duration::from_millis(100));
        }
    }
}

impl Seek for StreamHandle {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let available = self.bytes_written.load(Ordering::Acquire);

        match pos {
            SeekFrom::Start(offset) => {
                let clamped = offset.min(available);
                self.file.seek(SeekFrom::Start(clamped))
            }
            SeekFrom::Current(offset) => {
                let current = self.file.stream_position()?;
                let new = current as i64 + offset;
                if new < 0 {
                    self.file.seek(SeekFrom::Start(0))
                } else {
                    self.file.seek(SeekFrom::Start((new as u64).min(available)))
                }
            }
            SeekFrom::End(offset) => {
                let end = available as i64 + offset;
                if end < 0 {
                    self.file.seek(SeekFrom::Start(0))
                } else {
                    self.file.seek(SeekFrom::Start(end as u64))
                }
            }
        }
    }
}

fn make_handle(
    path: String,
    delete_on_drop: bool,
) -> (StreamWriter, StreamHandle) {
    let write_file = File::create(&path).expect("failed to create stream file");
    let read_file = File::open(&path).expect("failed to open stream file for reading");

    let bytes_written = Arc::new(AtomicU64::new(0));
    let finished = Arc::new(AtomicBool::new(false));
    let cvar = Arc::new(Condvar::new());
    let lock = Arc::new(Mutex::new(()));

    let writer = StreamWriter {
        file: write_file,
        bytes_written: bytes_written.clone(),
        finished: finished.clone(),
        cvar: cvar.clone(),
    };

    let handle = StreamHandle {
        file: read_file,
        bytes_written,
        finished,
        cvar,
        lock,
        path: path.clone(),
        delete_on_drop,
    };

    (writer, handle)
}

pub fn spawn_stream(url: &str) -> (StreamHandle, thread::JoinHandle<()>) {
    let tmp_dir = std::env::temp_dir();
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = tmp_dir.join(format!("tuibe_stream_{}.tmp", id));
    let path_str = path.to_string_lossy().to_string();

    let (writer, handle) = make_handle(path_str, true);
    let url = url.to_string();
    let jh = thread::spawn(move || run_ytdlp(&url, writer));

    (handle, jh)
}

pub fn spawn_cached_stream(url: &str, cache_path: &Path) -> (StreamHandle, thread::JoinHandle<()>) {
    let path_str = cache_path.to_string_lossy().to_string();
    let (writer, handle) = make_handle(path_str, false);
    let url = url.to_string();
    let jh = thread::spawn(move || run_ytdlp(&url, writer));

    (handle, jh)
}

fn run_ytdlp(url: &str, mut writer: StreamWriter) {
    let result = Command::new("yt-dlp")
        .args([
            "-f",
            "bestaudio[ext=m4a]/bestaudio",
            "-o",
            "-",
            "--no-playlist",
            "--no-warnings",
            "--no-cache-dir",
            url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    match result {
        Ok(mut child) => {
            if let Some(ref mut stdout) = child.stdout {
                let mut buf = [0u8; 65536];
                loop {
                    match stdout.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if writer.write(&buf[..n]).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            }
            let _ = child.wait();
            writer.finish();
        }
        Err(e) => {
            writer.set_error(format!("Failed to spawn yt-dlp: {}", e));
        }
    }
}
