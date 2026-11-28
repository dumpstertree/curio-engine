//! Blocking GETs through `curl`, for the import's job thread only.

use std::io::Write;
use std::process::{Command, Stdio};

/// GET `url` into `out_path`, following redirects (media downloads answer 307 to storage).
/// Headers go in on stdin (`-H @-`) so the token never shows in the process list.
pub fn get(url: &str, token: &str, accept: &str, out_path: &str) -> Result<(), String> {
    let mut command = Command::new("curl");
    command.args(["-s", "-S", "-L", "--max-time", "60", "-H", "@-", "-o", out_path, "-w", "%{http_code}", url]);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|e| format!("curl failed to start: {e}"))?;

    let mut headers = format!("Accept: {accept}\n");
    if !token.is_empty() {
        headers.push_str(&format!("Authorization: Token {token}\n"));
    }
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(headers.as_bytes());
    }
    let output = child
        .wait_with_output()
        .map_err(|e| format!("curl failed: {e}"))?;

    let code = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() && code == "200" {
        return Ok(());
    }
    let body = std::fs::read(out_path)
        .map(|b| snip(&b, 220))
        .unwrap_or_default();
    let _ = std::fs::remove_file(out_path);
    if code.is_empty() || code == "000" {
        let complaint = snip(&output.stderr, 200);
        return Err(format!("could not reach {url}: {complaint}"));
    }
    if body.is_empty() {
        return Err(format!("request for {url} returned http {code}"));
    }
    Err(format!("request for {url} returned http {code}: {body}"))
}

/// Printable-ASCII start of a response, enough to quote Penpot's error JSON.
fn snip(bytes: &[u8], max: usize) -> String {
    bytes
        .iter()
        .filter(|b| **b >= 32 && **b < 127)
        .take(max)
        .map(|b| *b as char)
        .collect()
}
