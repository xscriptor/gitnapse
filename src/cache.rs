use crate::error::CacheError;
use directories::ProjectDirs;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

pub struct PreviewCache {
    dir: PathBuf,
    ttl: Duration,
    memory: HashMap<String, (Instant, Vec<u8>)>,
    etag: HashMap<String, String>,
}

impl PreviewCache {
    /// Creates a new `PreviewCache` with the given TTL (in seconds).
    ///
    /// The cache directory is created under the platform-appropriate cache path.
    /// A minimum TTL of 1 second is enforced.
    ///
    /// # Errors
    /// Returns an error if the cache directory cannot be created.
    pub fn new(ttl_secs: u64) -> Result<Self, CacheError> {
        let project_dirs = ProjectDirs::from("com", "GitNapse", "GitNapse")
            .ok_or(CacheError::NoCacheDir)?;
        let dir = project_dirs.cache_dir().join("preview");
        fs::create_dir_all(&dir)
            .map_err(|e| CacheError::Other(format!("Cannot create preview cache directory: {}: {e}", dir.display())))?;
        Ok(Self {
            dir,
            ttl: Duration::from_secs(ttl_secs.max(1)),
            memory: HashMap::new(),
            etag: HashMap::new(),
        })
    }

    /// Retrieves cached content for the given repository, branch, and file path.
    ///
    /// The in-memory cache is checked first; if the entry is still within the TTL,
    /// it is returned. Otherwise, the on-disk cache is consulted. Entries that have
    /// expired are evicted.
    pub fn get(&mut self, repo: &str, branch: &str, path: &str) -> Option<Vec<u8>> {
        let key = cache_key(repo, branch, path);
        if let Some((created_at, content)) = self.memory.get(&key)
            && created_at.elapsed() <= self.ttl
        {
            return Some(content.clone());
        }

        let file = self.dir.join(format!("{key}.cache"));
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

        let content = fs::read(&file).ok()?;
        self.memory.insert(key, (Instant::now(), content.clone()));
        Some(content)
    }

    /// Stores content in the cache for the given repository, branch, and file path.
    ///
    /// Both the in-memory cache and the on-disk cache are updated. Any existing
    /// entry for the same key is overwritten. An optional ETag can be provided
    /// for conditional requests.
    pub fn put(&mut self, repo: &str, branch: &str, path: &str, content: &[u8], etag: Option<&str>) {
        let key = cache_key(repo, branch, path);
        let file = self.dir.join(format!("{key}.cache"));
        let _ = fs::write(&file, content);
        self.memory
            .insert(key.clone(), (Instant::now(), content.to_vec()));
        if let Some(etag_value) = etag {
            self.etag.insert(key, etag_value.to_string());
        }
    }

    /// Returns the stored ETag for a given cache entry, if one exists.
    #[allow(dead_code)]
    pub fn get_etag(&self, repo: &str, branch: &str, path: &str) -> Option<&str> {
        let key = cache_key(repo, branch, path);
        self.etag.get(&key).map(|s| s.as_str())
    }
}

fn cache_key(repo: &str, branch: &str, path: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    repo.hash(&mut hasher);
    branch.hash(&mut hasher);
    path.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
