use crate::figma::FigmaEvent;
use crate::penpot::run_import;

use std::sync::mpsc::Receiver;

/// The "Import from Penpot" dialog's inputs and in-flight job, owned by `AssetState`.
pub struct PenpotState {
    pub dialog_open: bool,
    /// Where the `.comp`s and images land: the dir whose context menu opened the dialog.
    pub target_dir: String,
    /// Workspace URL from the browser; its host is also the API host, so self-hosted works.
    pub url: String,
    pub token: String,
    /// `(target dir, events)` while the job thread runs; `AssetState::poll_penpot` clears it.
    pub job: Option<(String, Receiver<FigmaEvent>)>,
    pub progress: Vec<String>,
    pub summary: Option<String>,
    pub error: Option<String>,
}

impl PenpotState {
    pub fn new() -> Self {
        Self {
            dialog_open: false,
            target_dir: String::new(),
            url: String::new(),
            token: String::new(),
            job: None,
            progress: Vec::new(),
            summary: None,
            error: None,
        }
    }

    /// Starts the job thread; a no-op while one runs or with an input empty.
    pub fn start(&mut self) {
        if self.job.is_some() || self.url.trim().is_empty() || self.token.trim().is_empty() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let token = self.token.trim().to_string();
        let url = self.url.trim().to_string();
        let dir = self.target_dir.clone();
        std::thread::spawn(move || run_import(tx, token, url, dir));
        self.job = Some((self.target_dir.clone(), rx));
        self.progress = Vec::new();
        self.summary = None;
        self.error = None;
    }
}
