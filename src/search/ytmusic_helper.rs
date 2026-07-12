use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde::Deserialize;

const PYTHON_SCRIPT: &str = include_str!("../../scripts/ytmusic_helper.py");

fn find_python() -> String {
    static PYTHON: OnceLock<String> = OnceLock::new();
    PYTHON
        .get_or_init(|| {
        let binary_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()));

        let mut candidates: Vec<PathBuf> = Vec::new();

        candidates.push("python3".into());

        if let Some(ref dir) = binary_dir {
            candidates.push(dir.join(".venv/bin/python3"));
            candidates.push(dir.join("venv/bin/python3"));
        }

        candidates.push(PathBuf::from(".venv/bin/python3"));
        candidates.push(PathBuf::from("venv/bin/python3"));

        for c in &candidates {
            if Command::new(c)
                .arg("-c")
                .arg("import ytmusicapi")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
            {
                return c.to_string_lossy().to_string();
            }
        }

        "python3".to_string()
    }).clone()
}

pub fn run_python(args: &[&str]) -> Result<String> {
    let python = find_python();

    let mut cmd = Command::new(python);
    cmd.arg("-c")
        .arg(PYTHON_SCRIPT)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for arg in args {
        cmd.arg(arg);
    }

    let output = cmd
        .output()
        .context("failed to run python3 — is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = if stderr.contains("No module named 'ytmusicapi'") {
            "ytmusicapi not found — run: pip install ytmusicapi".to_string()
        } else {
            stderr.trim().to_string()
        };
        anyhow::bail!("ytmusic helper error: {}", msg);
    }

    String::from_utf8(output.stdout).context("invalid utf-8 from python helper")
}

#[derive(Deserialize)]
struct PlaylistResult {
    name: String,
    tracks: Vec<super::events::Track>,
}

pub fn fetch_playlist(url_or_id: &str) -> (String, Vec<super::events::Track>) {
    match run_python(&["playlist", url_or_id]) {
        Ok(stdout) => {
            match serde_json::from_str::<PlaylistResult>(&stdout) {
                Ok(pr) => (pr.name, pr.tracks),
                Err(_) => (format!("Import ({})", &url_or_id[..url_or_id.len().min(12)]), vec![]),
            }
        }
        Err(_) => (format!("Import ({})", &url_or_id[..url_or_id.len().min(12)]), vec![]),
    }
}
