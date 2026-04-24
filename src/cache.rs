use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

pub struct PreviewCache {
    dir: PathBuf,
    ttl: Duration,
    memory: HashMap<String, (Instant, String)>,
}

impl PreviewCache {
    pub fn new(ttl_secs: u64) -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "GitNapse", "GitNapse")
            .ok_or_else(|| anyhow!("Unable to resolve project cache directory"))?;
        let dir = project_dirs.cache_dir().join("preview");
        fs::create_dir_all(&dir)
            .with_context(|| format!("Cannot create preview cache directory: {}", dir.display()))?;
        Ok(Self {
            dir,
            ttl: Duration::from_secs(ttl_secs.max(1)),
            memory: HashMap::new(),
        })
    }

    pub fn get(&mut self, repo: &str, branch: &str, path: &str) -> Option<String> {
        let key = cache_key(repo, branch, path);
        if let Some((created_at, content)) = self.memory.get(&key)
            && created_at.elapsed() <= self.ttl
        {
            return Some(content.clone());
        }

        let file = self.dir.join(format!("{key}.txt"));
        if !file.exists() {
            return None;
        }
        let metadata = fs::metadata(&file).ok()?;
        let modified = metadata.modified().ok()?;
        let age = SystemTime::now().duration_since(modified).ok()?;
        if age > self.ttl {
            let _ = fs::remove_file(&file);
            return None;
        }

        let content = fs::read_to_string(&file).ok()?;
        self.memory
            .insert(key, (Instant::now(), content.clone()));
        Some(content)
    }

    pub fn put(&mut self, repo: &str, branch: &str, path: &str, content: &str) {
        let key = cache_key(repo, branch, path);
        let file = self.dir.join(format!("{key}.txt"));
        let _ = fs::write(file, content);
        self.memory
            .insert(key, (Instant::now(), content.to_string()));
    }
}

fn cache_key(repo: &str, branch: &str, path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(repo.as_bytes());
    hasher.update(b"::");
    hasher.update(branch.as_bytes());
    hasher.update(b"::");
    hasher.update(path.as_bytes());
    let digest = hasher.finalize();
    digest.iter().map(|b| format!("{b:02x}")).collect()
}
