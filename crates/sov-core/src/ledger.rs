//! The ledger: one JSON object per line, hash-chained (SPEC R7).

use std::io::Write;
use std::path::Path;

use crate::Result;
use crate::hash;
use crate::model::*;

impl Run {
    pub fn compute_hash(&self) -> Result<String> {
        hash::hash_without(self, "hash")
    }

    pub fn seal(&mut self) -> Result<()> {
        self.hash = self.compute_hash()?;
        Ok(())
    }
}

pub fn read_runs(path: &Path) -> Result<Vec<Run>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(path)?;
    let mut runs = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let run: Run =
            serde_json::from_str(line).map_err(|e| format!("{}:{}: {e}", path.display(), i + 1))?;
        runs.push(run);
    }
    Ok(runs)
}

pub fn last_hash(path: &Path) -> Result<Option<String>> {
    Ok(read_runs(path)?.last().map(|r| r.hash.clone()))
}

pub fn append(path: &Path, run: &Run) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", serde_json::to_string(run)?)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ChainReport {
    pub runs: usize,
    pub ok: bool,
    pub problem: Option<String>,
    pub last: Option<String>,
}

pub fn verify_chain(path: &Path) -> Result<ChainReport> {
    let runs = read_runs(path)?;
    let mut previous: Option<String> = None;
    for (i, run) in runs.iter().enumerate() {
        let recomputed = run.compute_hash()?;
        if recomputed != run.hash {
            return Ok(ChainReport {
                runs: runs.len(),
                ok: false,
                problem: Some(format!("line {}: hash does not match content", i + 1)),
                last: previous,
            });
        }
        if run.prev != previous {
            return Ok(ChainReport {
                runs: runs.len(),
                ok: false,
                problem: Some(format!(
                    "line {}: prev does not link to the line before it",
                    i + 1
                )),
                last: previous,
            });
        }
        previous = Some(run.hash.clone());
    }
    Ok(ChainReport {
        runs: runs.len(),
        ok: true,
        problem: None,
        last: previous,
    })
}
