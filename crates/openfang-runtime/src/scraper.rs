//! Web scraping via a Python Scrapling bridge.
//!
//! Manages a single shared subprocess that runs `scraper_bridge.py`, communicating
//! via the same JSON-line stdin/stdout protocol used by the browser bridge.
//!
//! Unlike the browser (which keeps per-agent sessions for page state), scraping is
//! stateless — each fetch is independent — so one shared subprocess serves all agents.
//!
//! # Security
//! - SSRF check runs in Rust *before* sending URLs to Python
//! - Bridge subprocess launched with cleared environment (safe passthrough only)
//! - All returned content wrapped with `wrap_external_content()` markers
//! - Falls back gracefully when Scrapling is not installed (plain HTTP)

use openfang_types::config::ScraperConfig;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Stdio};
use std::sync::OnceLock;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Embedded Python bridge script (compiled into the binary).
const BRIDGE_SCRIPT: &str = include_str!("scraper_bridge.py");

// ── Protocol types ──────────────────────────────────────────────────────────

/// Command sent from Rust to the Python scraper bridge.
#[derive(Debug, Serialize)]
#[serde(tag = "action")]
pub enum ScraperCommand {
    /// HTTP fetch with anti-bot headers via Scrapling StealthyFetcher.
    FetchUrl { url: String },
    /// Full browser fetch via Scrapling DynamicFetcher (requires Playwright).
    FetchDynamic { url: String, wait_for: Option<String> },
    /// Health check — returns pong.
    Ping,
}

/// Response received from the Python bridge.
#[derive(Debug, Deserialize)]
pub struct ScraperResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

// ── Internal subprocess ─────────────────────────────────────────────────────

struct ScraperProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl ScraperProcess {
    fn send(&mut self, cmd: &ScraperCommand) -> Result<ScraperResponse, String> {
        let json = serde_json::to_string(cmd).map_err(|e| format!("Serialize error: {e}"))?;
        self.stdin
            .write_all(json.as_bytes())
            .map_err(|e| format!("Failed to write to scraper stdin: {e}"))?;
        self.stdin
            .write_all(b"\n")
            .map_err(|e| format!("Failed to write newline: {e}"))?;
        self.stdin
            .flush()
            .map_err(|e| format!("Failed to flush scraper stdin: {e}"))?;

        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read scraper stdout: {e}"))?;

        if line.trim().is_empty() {
            return Err("Scraper bridge process closed unexpectedly".to_string());
        }

        serde_json::from_str(line.trim())
            .map_err(|e| format!("Failed to parse scraper response '{line}': {e}"))
    }
}

impl Drop for ScraperProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ── Manager ─────────────────────────────────────────────────────────────────

/// Manages the shared Scrapling subprocess.
///
/// One subprocess is kept alive and shared across all agents. Since scraping is
/// stateless, there is no need for per-agent sessions.
pub struct ScraperManager {
    process: Mutex<Option<ScraperProcess>>,
    config: ScraperConfig,
    bridge_path: OnceLock<PathBuf>,
}

impl ScraperManager {
    pub fn new(config: ScraperConfig) -> Self {
        Self {
            process: Mutex::new(None),
            config,
            bridge_path: OnceLock::new(),
        }
    }

    /// Write the embedded bridge script to a temp file (once per process lifetime).
    fn ensure_bridge_script(&self) -> Result<&PathBuf, String> {
        if let Some(path) = self.bridge_path.get() {
            return Ok(path);
        }
        let dir = std::env::temp_dir().join("openfang");
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create temp dir: {e}"))?;
        let path = dir.join("scraper_bridge.py");
        std::fs::write(&path, BRIDGE_SCRIPT)
            .map_err(|e| format!("Failed to write scraper bridge script: {e}"))?;
        debug!(path = %path.display(), "Wrote scraper bridge script");
        let _ = self.bridge_path.set(path);
        Ok(self.bridge_path.get().unwrap())
    }

    /// Spawn the Python subprocess with minimal safe environment.
    fn spawn_process(&self) -> Result<ScraperProcess, String> {
        let bridge_path = self.ensure_bridge_script()?;

        let mut cmd = std::process::Command::new(&self.config.python_path);
        cmd.arg(bridge_path.to_string_lossy().as_ref());
        cmd.arg("--timeout").arg(self.config.timeout_secs.to_string());
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());

        // SECURITY: Isolate environment
        cmd.env_clear();
        #[cfg(windows)]
        {
            for var in &["SYSTEMROOT", "PATH", "TEMP", "TMP", "USERPROFILE", "APPDATA", "LOCALAPPDATA"] {
                if let Ok(v) = std::env::var(var) {
                    cmd.env(var, v);
                }
            }
            cmd.env("PYTHONIOENCODING", "utf-8");
        }
        #[cfg(not(windows))]
        {
            for var in &["PATH", "HOME", "TMPDIR", "XDG_CACHE_HOME"] {
                if let Ok(v) = std::env::var(var) {
                    cmd.env(var, v);
                }
            }
            cmd.env("PYTHONIOENCODING", "utf-8");
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn scraper bridge: {e}. Ensure Python is installed."))?;

        let stdin = child.stdin.take().ok_or("Failed to capture scraper stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to capture scraper stdout")?;
        let mut reader = BufReader::new(stdout);

        // Wait for "ready" signal
        let mut ready_line = String::new();
        reader
            .read_line(&mut ready_line)
            .map_err(|e| format!("Scraper bridge failed to start: {e}"))?;

        if ready_line.trim().is_empty() {
            let _ = child.kill();
            return Err("Scraper bridge exited without sending ready signal.".to_string());
        }

        let ready: ScraperResponse = serde_json::from_str(ready_line.trim())
            .map_err(|e| format!("Scraper bridge startup failed: {e}. Output: {ready_line}"))?;

        if !ready.success {
            let err = ready.error.unwrap_or_else(|| "Unknown error".to_string());
            let _ = child.kill();
            return Err(format!("Scraper bridge failed to start: {err}"));
        }

        info!("Scraper bridge started");
        Ok(ScraperProcess { child, stdin, stdout: reader })
    }

    /// Send a command to the scraper bridge, (re)spawning if needed.
    async fn send_command(&self, cmd: &ScraperCommand) -> Result<ScraperResponse, String> {
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut guard = self.process.lock().await;

                // Spawn if not running
                if guard.is_none() {
                    *guard = Some(self.spawn_process()?);
                }

                let proc = guard.as_mut().unwrap();
                let result = proc.send(cmd);

                // If the process died, clear it so next call respawns
                if result.is_err() {
                    warn!("Scraper bridge died, will respawn on next call");
                    *guard = None;
                }

                result
            })
        })
    }
}

// ── Tool handler functions ──────────────────────────────────────────────────

/// scrape_url — Fetch a URL with anti-bot stealth headers.
/// Uses Scrapling StealthyFetcher. Falls back to plain HTTP if Scrapling not installed.
pub async fn tool_scrape_url(
    input: &serde_json::Value,
    mgr: &ScraperManager,
) -> Result<String, String> {
    let url = input["url"].as_str().ok_or("Missing 'url' parameter")?;

    // SECURITY: SSRF check before delegating to Python
    crate::web_fetch::check_ssrf(url)?;

    let resp = mgr.send_command(&ScraperCommand::FetchUrl { url: url.to_string() }).await?;

    if !resp.success {
        return Err(resp.error.unwrap_or_else(|| "scrape_url failed".to_string()));
    }

    let data = resp.data.unwrap_or_default();
    let content = data["content"].as_str().unwrap_or("(no content)");
    let mode = data["mode"].as_str().unwrap_or("unknown");

    debug!(url, mode, "scrape_url complete");

    let wrapped = crate::web_content::wrap_external_content(url, content);
    Ok(format!("Scraped [{mode}]: {url}\n\n{wrapped}"))
}

/// scrape_dynamic — Fetch a JS-rendered page via a full headless browser.
/// Uses Scrapling DynamicFetcher (requires Playwright + Chromium).
pub async fn tool_scrape_dynamic(
    input: &serde_json::Value,
    mgr: &ScraperManager,
) -> Result<String, String> {
    let url = input["url"].as_str().ok_or("Missing 'url' parameter")?;
    let wait_for = input["wait_for"].as_str().map(|s| s.to_string());

    // SECURITY: SSRF check before delegating to Python
    crate::web_fetch::check_ssrf(url)?;

    let resp = mgr
        .send_command(&ScraperCommand::FetchDynamic {
            url: url.to_string(),
            wait_for,
        })
        .await?;

    if !resp.success {
        return Err(resp.error.unwrap_or_else(|| "scrape_dynamic failed".to_string()));
    }

    let data = resp.data.unwrap_or_default();
    let content = data["content"].as_str().unwrap_or("(no content)");

    debug!(url, "scrape_dynamic complete");

    let wrapped = crate::web_content::wrap_external_content(url, content);
    Ok(format!("Scraped [dynamic/JS]: {url}\n\n{wrapped}"))
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scraper_command_serialize_fetch_url() {
        let cmd = ScraperCommand::FetchUrl { url: "https://example.com".to_string() };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"action\":\"FetchUrl\""));
        assert!(json.contains("\"url\":\"https://example.com\""));
    }

    #[test]
    fn test_scraper_command_serialize_fetch_dynamic() {
        let cmd = ScraperCommand::FetchDynamic {
            url: "https://example.com".to_string(),
            wait_for: Some(".product-title".to_string()),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"action\":\"FetchDynamic\""));
        assert!(json.contains("\"url\":\"https://example.com\""));
        assert!(json.contains("\"wait_for\":\""));
    }

    #[test]
    fn test_scraper_command_serialize_dynamic_no_wait() {
        let cmd = ScraperCommand::FetchDynamic {
            url: "https://example.com".to_string(),
            wait_for: None,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"action\":\"FetchDynamic\""));
        assert!(json.contains("\"wait_for\":null"));
    }

    #[test]
    fn test_scraper_command_serialize_ping() {
        let cmd = ScraperCommand::Ping;
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("\"action\":\"Ping\""));
    }

    #[test]
    fn test_scraper_response_deserialize_success() {
        let json = r#"{"success": true, "data": {"url": "https://example.com", "content": "Hello", "mode": "stealth"}}"#;
        let resp: ScraperResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        let data = resp.data.unwrap();
        assert_eq!(data["mode"], "stealth");
        assert_eq!(data["content"], "Hello");
    }

    #[test]
    fn test_scraper_response_deserialize_error() {
        let json = r#"{"success": false, "error": "Cloudflare blocked"}"#;
        let resp: ScraperResponse = serde_json::from_str(json).unwrap();
        assert!(!resp.success);
        assert_eq!(resp.error.unwrap(), "Cloudflare blocked");
    }

    #[test]
    fn test_scraper_manager_new() {
        let config = ScraperConfig::default();
        let mgr = ScraperManager::new(config);
        // Just verify it constructs without panic
        drop(mgr);
    }

    #[test]
    fn test_ssrf_blocks_localhost_in_scraper() {
        // scrape_url delegates SSRF to check_ssrf — verify the check is the same
        assert!(crate::web_fetch::check_ssrf("http://localhost/data").is_err());
        assert!(crate::web_fetch::check_ssrf("http://169.254.169.254/metadata").is_err());
        assert!(crate::web_fetch::check_ssrf("http://10.0.0.1/internal").is_err());
    }
}
