//! Complete, bounded song downloads. Partial files are never used for playback.
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use crate::config::Config;
use crate::subsonic::models::Child;

#[derive(Clone, Default)]
pub struct SongCache { lock: Arc<tokio::sync::Mutex<()>> }

impl SongCache {
    fn root() -> Option<PathBuf> { crate::config::paths::cache_dir().map(|p| p.join("songs")) }
    fn namespace(config: &Config) -> Option<PathBuf> {
        let key = format!("{}\0{}", config.base_url, config.username);
        Some(Self::root()?.join(format!("{:x}", md5::compute(key))))
    }
    fn path(config: &Config, id: &str) -> Option<PathBuf> {
        Some(Self::namespace(config)?.join(format!("{:x}.audio", md5::compute(id))))
    }
    pub fn lookup(config: &Config, id: &str) -> Option<PathBuf> {
        if config.cache_size_gb == 0 { return None; }
        Self::path(config, id).filter(|p| p.is_file()).map(|p| {
            if let Ok(file) = std::fs::OpenOptions::new().write(true).open(&p) {
                let _ = file.set_times(std::fs::FileTimes::new().set_modified(std::time::SystemTime::now()));
            }
            p
        })
    }
    pub fn songs(config: &Config) -> Vec<Child> {
        if config.cache_size_gb == 0 { return vec![]; }
        let Some(dir) = Self::namespace(config) else { return vec![] };
        let mut songs: Vec<Child> = std::fs::read_dir(dir).into_iter().flatten().flatten().filter_map(|entry| {
            let p = entry.path();
            if p.extension()?.to_str()? != "json" || !p.with_extension("audio").is_file() { return None; }
            serde_json::from_slice(&std::fs::read(p).ok()?).ok()
        }).collect();
        songs.sort_by(|a, b| (&a.artist, &a.album, a.disc_number, a.track, &a.title).cmp(&(&b.artist, &b.album, b.disc_number, b.track, &b.title)));
        songs
    }
    fn entries(dir: &Path, result: &mut Vec<(PathBuf, u64, std::time::SystemTime)>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() { Self::entries(&path, result); }
                else if path.extension().is_some_and(|x| x == "audio") {
                    if let Ok(m) = entry.metadata() { result.push((path, m.len(), m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH))); }
                }
            }
        }
    }
    pub async fn trim(&self, gb: u8) {
        let _guard = self.lock.lock().await;
        Self::trim_locked(u64::from(gb) * 1_000_000_000, 0);
    }
    /// Delete cached songs and metadata, reporting any files that could not be removed.
    pub async fn clear(&self) -> Result<(), String> {
        let _guard = self.lock.lock().await;
        let Some(root) = Self::root() else { return Err("Cache directory unavailable".into()) };
        Self::clear_at(&root)
    }
    fn clear_at(root: &Path) -> Result<(), String> {
        if !root.exists() { return Ok(()); }
        let mut failures = Vec::new();
        fn visit(dir: &Path, failures: &mut Vec<String>) {
            match std::fs::read_dir(dir) {
                Ok(entries) => for entry in entries {
                    match entry {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.is_dir() { visit(&path, failures); }
                            else if path.extension().is_some_and(|ext| ext == "audio" || ext == "json") {
                                if let Err(e) = std::fs::remove_file(&path) { failures.push(format!("{}: {}", path.display(), e)); }
                            }
                        }
                        Err(e) => failures.push(e.to_string()),
                    }
                },
                Err(e) => failures.push(format!("{}: {}", dir.display(), e)),
            }
        }
        visit(root, &mut failures);
        if failures.is_empty() { Ok(()) } else { Err(failures.join("; ")) }
    }
    fn trim_locked(limit: u64, reserve: u64) {
        let Some(root) = Self::root() else { return };
        Self::trim_at(&root, limit, reserve);
    }
    fn trim_at(root: &Path, limit: u64, reserve: u64) {
        let mut entries = Vec::new(); Self::entries(&root, &mut entries);
        entries.sort_by_key(|e| e.2);
        let mut total: u64 = entries.iter().map(|e| e.1).sum();
        for (path, size, _) in entries {
            if total.saturating_add(reserve) <= limit { break; }
            if std::fs::remove_file(&path).is_ok() {
                let _ = std::fs::remove_file(path.with_extension("json"));
                total = total.saturating_sub(size);
            }
        }
    }
    pub async fn download(&self, config: Config, song: Child, url: String) -> Result<(), String> {
        let _guard = self.lock.lock().await;
        let limit = u64::from(config.cache_size_gb) * 1_000_000_000;
        if limit == 0 { return Err("Cache is disabled".into()); }
        if Self::lookup(&config, &song.id).is_some() { return Ok(()); }
        let path = Self::path(&config, &song.id).ok_or("Cache directory unavailable")?;
        let parent = path.parent().ok_or("Cache directory unavailable")?;
        tokio::fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
        let mut response = reqwest::Client::new().get(url).timeout(std::time::Duration::from_secs(600)).send().await.map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?;
        if response.headers().get(reqwest::header::CONTENT_TYPE).and_then(|v| v.to_str().ok()).is_some_and(|s| s.contains("json") || s.contains("xml") || s.contains("text/html")) {
            return Err("Server returned an error instead of audio".into());
        }
        if response.content_length().is_some_and(|n| n > limit) { return Err("Song exceeds cache limit".into()); }
        let temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
        let mut file = tokio::fs::File::from_std(temp.reopen().map_err(|e| e.to_string())?);
        let mut size = 0u64;
        while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
            size += chunk.len() as u64;
            if size > limit { return Err("Song exceeds cache limit".into()); }
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        }
        if size == 0 { return Err("Empty audio response".into()); }
        file.flush().await.map_err(|e| e.to_string())?;
        drop(file);
        Self::trim_locked(limit, size);
        let metadata = serde_json::to_vec(&song).map_err(|e| e.to_string())?;
        std::fs::write(path.with_extension("json"), metadata).map_err(|e| e.to_string())?;
        temp.persist(&path).map_err(|e| e.error.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evicts_complete_audio_and_metadata_to_enforce_limit() {
        let dir = tempfile::tempdir().unwrap();
        let first = dir.path().join("first.audio");
        let second = dir.path().join("second.audio");
        std::fs::write(&first, [1u8; 8]).unwrap();
        std::fs::write(first.with_extension("json"), "{}").unwrap();
        std::fs::write(&second, [1u8; 8]).unwrap();
        SongCache::trim_at(dir.path(), 8, 0);
        let remaining: Vec<_> = [first, second].into_iter().filter(|p| p.exists()).collect();
        assert_eq!(remaining.len(), 1);
        SongCache::trim_at(dir.path(), 0, 0);
        assert!(!remaining[0].exists());
    }

    #[test]
    fn clear_removes_audio_and_metadata_but_not_other_files() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("account");
        std::fs::create_dir(&nested).unwrap();
        for name in ["a.audio", "a.json", "other.txt"] { std::fs::write(nested.join(name), b"data").unwrap(); }
        SongCache::clear_at(dir.path()).unwrap();
        assert!(!nested.join("a.audio").exists());
        assert!(!nested.join("a.json").exists());
        assert!(nested.join("other.txt").exists());
    }

    #[test]
    fn server_and_user_get_separate_namespaces() {
        let mut a = Config::new();
        a.base_url = "https://example.com/".into();
        a.username = "alice".into();
        let mut b = a.clone();
        b.username = "bob".into();
        assert_ne!(SongCache::namespace(&a), SongCache::namespace(&b));
        b.username = a.username.clone();
        b.base_url = "https://other.example/".into();
        assert_ne!(SongCache::namespace(&a), SongCache::namespace(&b));
    }
}
