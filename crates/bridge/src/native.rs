//! Atomic local-file transport; never edits either application's project files.
use super::*;
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
pub fn root() -> Result<PathBuf> {
    if let Some(p) = std::env::var_os("COLORCRAFT_BRIDGE_DIR").filter(|p| !p.is_empty()) {
        return Ok(PathBuf::from(p));
    }
    Ok(PathBuf::from(
        std::env::var_os("LOCALAPPDATA")
            .ok_or("Local ColorCraft bridge is unavailable on this platform")?,
    )
    .join("ColorCraft/Bridge/v1"))
}
pub fn library_root() -> Result<PathBuf> {
    if let Some(p) = std::env::var_os("COLORCRAFT_LIBRARY_DIR").filter(|p| !p.is_empty()) {
        return Ok(PathBuf::from(p));
    }
    Ok(PathBuf::from(
        std::env::var_os("APPDATA")
            .ok_or("Local ColorCraft library is unavailable on this platform")?,
    )
    .join("ArtCraft/Palettes"))
}
pub fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let f = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    if !f.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("Not a regular file".into());
    }
    let mut bytes = vec![];
    f.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err("ColorCraft data exceeds the size limit".into());
    }
    Ok(bytes)
}
pub fn atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_JSON {
        return Err("ColorCraft data exceeds the size limit".into());
    }
    let parent = path.parent().ok_or("Missing parent directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = parent.join(format!("{}.tmp", uid()));
    let result = (|| {
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp)
            .map_err(|e| e.to_string())?;
        f.write_all(&bytes).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
        fs::rename(&temp, path).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _cleanup = fs::remove_file(&temp);
    }
    result
}
pub fn library(root: &Path) -> Result<Library> {
    let lib: Library = serde_json::from_slice(&read(&root.join("library.json"), 8 * 1024 * 1024)?)
        .map_err(|e| e.to_string())?;
    if lib.schema_version != 1 || lib.palettes.len() > 10000 {
        return Err("Unsupported ColorCraft library version or size".into());
    }
    let mut ids = BTreeSet::new();
    if lib.palettes.iter().any(|e| {
        e.palette_id.is_empty()
            || e.palette_id.len() > 1024
            || e.name.len() > 16384
            || !ids.insert(&e.palette_id)
    }) {
        return Err("Invalid or duplicate palette library identity".into());
    }
    Ok(lib)
}
pub fn snapshot(root: &Path, entry: &LibraryEntry) -> Result<Snapshot> {
    let rel = Path::new(&entry.snapshot);
    if rel.is_absolute() || rel.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err("Unsafe palette snapshot path".into());
    }
    let base = root.canonicalize().map_err(|e| e.to_string())?;
    let path = root.join(rel).canonicalize().map_err(|e| e.to_string())?;
    if !path.starts_with(&base) {
        return Err("Palette snapshot escapes the library".into());
    }
    let s: Snapshot = serde_json::from_slice(&read(&path, MAX_JSON)?).map_err(|e| e.to_string())?;
    s.validate()?;
    if s.palette_id != entry.palette_id || s.revision != entry.revision {
        return Err("Palette snapshot identity does not match the index".into());
    }
    Ok(s)
}
pub fn submit(root: &Path, request: &Request) -> Result<()> {
    request.validate()?;
    let bytes = serde_json::to_vec(request).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_REQUEST {
        return Err("ColorCraft request exceeds 32 MiB".into());
    }
    let target = root
        .join("requests")
        .join(format!("{}.json", request.request_id));
    let completed = root
        .join("completed")
        .join(format!("{}.json", request.request_id));
    if target.exists() || completed.exists() {
        if read(
            if completed.exists() {
                &completed
            } else {
                &target
            },
            MAX_REQUEST,
        )? == bytes
        {
            return Ok(());
        }
        return Err("Request ID was already used with different data".into());
    }
    atomic(&target, request)
}
pub fn response(root: &Path, id: &str) -> Result<Option<Response>> {
    uuid::Uuid::parse_str(id).map_err(|e| e.to_string())?;
    let path = root.join("responses").join(format!("{id}.json"));
    if !path.exists() {
        return Ok(None);
    }
    let r: Response = serde_json::from_slice(&read(&path, MAX_JSON)?).map_err(|e| e.to_string())?;
    if r.schema_version != VERSION || r.request_id != id {
        return Err("Invalid ColorCraft response identity".into());
    }
    if let Some(snapshot) = &r.snapshot {
        snapshot.validate()?;
    }
    Ok(Some(r))
}
pub struct Receiver {
    pub root: PathBuf,
    _lock: File,
}
impl Receiver {
    pub fn acquire(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(root.join("requests")).map_err(|e| e.to_string())?;
        fs::create_dir_all(root.join("responses")).map_err(|e| e.to_string())?;
        fs::create_dir_all(root.join("completed")).map_err(|e| e.to_string())?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join("receiver.lock"))
            .map_err(|e| e.to_string())?;
        lock.try_lock_exclusive()
            .map_err(|_| "ColorCraft bridge already has a receiver".to_string())?;
        Ok(Self { root, _lock: lock })
    }
    pub fn pending(&self) -> Result<Vec<Request>> {
        let mut out = vec![];
        for e in fs::read_dir(self.root.join("requests"))
            .map_err(|e| e.to_string())?
            .take(10000)
        {
            let e = e.map_err(|e| e.to_string())?;
            let path = e.path();
            let Some(id) = path.file_stem().and_then(|n| n.to_str()) else {
                continue;
            };
            if path.extension().is_none_or(|v| v != "json") || uuid::Uuid::parse_str(id).is_err() {
                continue;
            }
            if self
                .root
                .join("responses")
                .join(format!("{id}.json"))
                .exists()
            {
                // Recover an interrupted archive after an already-durable response.
                fs::rename(
                    &path,
                    self.root.join("completed").join(format!("{id}.json")),
                )
                .map_err(|e| e.to_string())?;
                continue;
            }
            let parsed = read(&path, MAX_REQUEST).and_then(|bytes| {
                serde_json::from_slice::<Request>(&bytes).map_err(|e| e.to_string())
            });
            match parsed {
                Ok(req) if req.request_id == id => match req.validate() {
                    Ok(()) => out.push(req),
                    Err(e) => self.finish(&Response::error(&req, "invalidRequest", e))?,
                },
                _ => {
                    let mut rejected = Request::new("inspect", "invalid");
                    rejected.request_id = id.to_owned();
                    self.finish(&Response::error(
                        &rejected,
                        "invalidRequest",
                        "Malformed or excessive bridge request".into(),
                    ))?;
                }
            }
            if out.len() == 16 {
                break;
            }
        }
        Ok(out)
    }
    pub fn finish(&self, response: &Response) -> Result<()> {
        uuid::Uuid::parse_str(&response.request_id).map_err(|e| e.to_string())?;
        atomic(
            &self
                .root
                .join("responses")
                .join(format!("{}.json", response.request_id)),
            response,
        )?;
        let source = self
            .root
            .join("requests")
            .join(format!("{}.json", response.request_id));
        if source.exists() {
            fs::rename(
                source,
                self.root
                    .join("completed")
                    .join(format!("{}.json", response.request_id)),
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    fn temp() -> PathBuf {
        std::env::temp_dir().join(format!("colorcraft-wire-{}", uid()))
    }
    #[test]
    fn receiver_is_exclusive_and_requests_are_idempotent() {
        let root = temp();
        let receiver = Receiver::acquire(root.clone()).unwrap();
        assert!(Receiver::acquire(root.clone()).is_err());
        let request = Request::new("inspect", "palette");
        submit(&root, &request).unwrap();
        submit(&root, &request).unwrap();
        assert_eq!(receiver.pending().unwrap().len(), 1);
        let mut changed = request.clone();
        changed.palette_id = "other".into();
        assert!(submit(&root, &changed).is_err());
        receiver.finish(&Response::new(&request, "ready")).unwrap();
        assert!(receiver.pending().unwrap().is_empty());
        assert!(response(&root, &request.request_id).unwrap().unwrap().ok);
        drop(receiver);
        assert!(Receiver::acquire(root).is_ok());
    }
    #[test]
    fn malformed_request_does_not_block_following_requests() {
        let root = temp();
        let receiver = Receiver::acquire(root.clone()).unwrap();
        let id = uid();
        fs::write(root.join("requests").join(format!("{id}.json")), b"garbage").unwrap();
        let good = Request::new("inspect", "palette");
        submit(&root, &good).unwrap();
        assert_eq!(receiver.pending().unwrap().len(), 1);
        assert_eq!(
            response(&root, &id).unwrap().unwrap().status,
            "invalidRequest"
        );
    }
    #[test]
    fn snapshot_traversal_is_rejected_before_reading() {
        let root = temp();
        fs::create_dir_all(&root).unwrap();
        let e = LibraryEntry {
            palette_id: "p".into(),
            revision: 1,
            name: "p".into(),
            snapshot: "../outside.json".into(),
            source_path: String::new(),
            colors: 0,
            preview: vec![],
        };
        assert!(snapshot(&root, &e).is_err());
    }
}
