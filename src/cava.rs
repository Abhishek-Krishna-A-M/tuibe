use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::path::PathBuf;

use dirs::cache_dir;

fn config_dir() -> PathBuf {
    let mut p = cache_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    p.push("tuibe");
    let _ = std::fs::create_dir_all(&p);
    p
}

pub struct CavaProcess {
    child: Child,
    handle: Option<JoinHandle<()>>,
    tx: Sender<Vec<f32>>,
}

impl CavaProcess {
    pub fn start(bars: u32, sensitivity: u32) -> (Self, Receiver<Vec<f32>>) {
        let (tx, rx) = mpsc::channel();
        let mut child = spawn_cava(bars, sensitivity);
        let handle = spawn_reader(child.stdout.take().unwrap(), tx.clone(), bars);
        (
            Self {
                child,
                handle: Some(handle),
                tx,
            },
            rx,
        )
    }

    pub fn restart(&mut self, bars: u32, sensitivity: u32) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        self.child = spawn_cava(bars, sensitivity);
        self.handle = Some(spawn_reader(
            self.child.stdout.take().unwrap(),
            self.tx.clone(),
            bars,
        ));
    }

    pub fn join(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn write_config(bars: u32, sensitivity: u32) -> PathBuf {
    let path = config_dir().join("cava.config");
    let config = format!(
        r#"[general]
bars = {bars}
framerate = 60
autosens = 0
sensitivity = {sens}

[input]
method = pulse
source = auto

[output]
method = raw
raw_target = /dev/stdout
bit_format = 16
channels = mono
mono_option = average

[smoothing]
noise_reduction = 60
"#,
        bars = bars,
        sens = sensitivity
    );
    let _ = std::fs::write(&path, &config);
    path
}

fn spawn_cava(bars: u32, sensitivity: u32) -> Child {
    let config_path = write_config(bars, sensitivity);
    Command::new("cava")
        .arg("-p")
        .arg(&config_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn cava — is it installed?")
}

fn spawn_reader(
    stdout: std::process::ChildStdout,
    tx: Sender<Vec<f32>>,
    bars: u32,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let frame_size = bars as usize;
        let mut reader = stdout;
        let mut buf = vec![0u8; frame_size * 2];

        loop {
            let mut read = 0;
            while read < buf.len() {
                match reader.read(&mut buf[read..]) {
                    Ok(0) => return,
                    Ok(n) => read += n,
                    Err(_) => return,
                }
            }

            let vals: Vec<f32> = buf
                .chunks_exact(2)
                .map(|c| u16::from_ne_bytes([c[0], c[1]]) as f32 / 65535.0)
                .collect();

            if tx.send(vals).is_err() {
                return;
            }
        }
    })
}
