use std::process::{Command, Stdio};
use std::sync::OnceLock;

use anyhow::{Context, Result};

const PYTHON_SCRIPT: &str = include_str!("../../scripts/ytmusic_helper.py");

fn find_python() -> &'static str {
    static PYTHON: OnceLock<&str> = OnceLock::new();
    PYTHON.get_or_init(|| {
        let candidates = &[".venv/bin/python3", "venv/bin/python3", "python3"];
        for c in candidates {
            if Command::new(c)
                .arg("-c")
                .arg("import ytmusicapi")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
            {
                return *c;
            }
        }
        "python3"
    })
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
        anyhow::bail!("ytmusic helper error: {}", stderr.trim());
    }

    String::from_utf8(output.stdout).context("invalid utf-8 from python helper")
}
