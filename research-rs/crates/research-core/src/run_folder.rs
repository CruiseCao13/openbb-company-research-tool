use crate::io::ensure_dir;
use crate::types::RunContext;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RunFolder {
    pub root: PathBuf,
    pub report: PathBuf,
    pub raw: PathBuf,
    pub metadata: PathBuf,
    pub ai: PathBuf,
    pub audit: PathBuf,
    pub self_review: PathBuf,
    pub data: PathBuf,
    pub charts: PathBuf,
    pub pack: PathBuf,
}

impl RunFolder {
    pub fn new(ctx: &RunContext) -> Self {
        let root = PathBuf::from(&ctx.root)
            .join(&ctx.ticker)
            .join("runs")
            .join(&ctx.run_id);
        Self {
            report: root.join("report"),
            raw: root.join("raw"),
            metadata: root.join("metadata"),
            ai: root.join("ai"),
            audit: root.join("audit"),
            self_review: root.join("self_review"),
            data: root.join("data"),
            charts: root.join("charts"),
            pack: root.join("pack"),
            root,
        }
    }

    pub fn create(&self) -> Result<()> {
        for dir in [
            &self.root,
            &self.report,
            &self.raw,
            &self.metadata,
            &self.ai,
            &self.ai.join("prompts"),
            &self.ai.join("responses"),
            &self.audit,
            &self.self_review,
            &self.data,
            &self.charts,
            &self.pack,
        ] {
            ensure_dir(dir)?;
        }
        Ok(())
    }
}
