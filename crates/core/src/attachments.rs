use crate::now_utc_rfc3339;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SetKey {
    pub session_id: String,
    pub ex_code: String,
    pub set_num: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentEventKind {
    Attach,
    Detach,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAttachmentEvent {
    pub event: AttachmentEventKind,
    pub set_key: SetKey,
    /// Media file name (session-local) or relative path
    pub media_file: String,
    pub created_at: String, // RFC3339 UTC
    /// Optional: schema versioning for forwards-compat
    #[serde(default)]
    pub version: Option<u32>,
}

impl MediaAttachmentEvent {
    pub fn new(event: AttachmentEventKind, set_key: SetKey, media_file: impl Into<String>) -> Self {
        Self {
            event,
            set_key,
            media_file: media_file.into(),
            created_at: now_utc_rfc3339(),
            version: Some(1),
        }
    }
}

#[derive(Error, Debug)]
pub enum AttachmentIoError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Append-only JSONL writer for media attachments
pub struct MediaAttachmentWriter {
    path: PathBuf,
}

impl MediaAttachmentWriter {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }

    fn ensure_parent_dir(&self) -> Result<(), AttachmentIoError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    fn append_event(&self, ev: &MediaAttachmentEvent) -> Result<(), AttachmentIoError> {
        self.ensure_parent_dir()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let line = serde_json::to_string(ev)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        file.flush()?;
        Ok(())
    }

    pub fn attach(
        &self,
        set_key: SetKey,
        media_file: impl Into<String>,
    ) -> Result<(), AttachmentIoError> {
        let ev = MediaAttachmentEvent::new(AttachmentEventKind::Attach, set_key, media_file);
        self.append_event(&ev)
    }

    pub fn detach(
        &self,
        set_key: SetKey,
        media_file: impl Into<String>,
    ) -> Result<(), AttachmentIoError> {
        let ev = MediaAttachmentEvent::new(AttachmentEventKind::Detach, set_key, media_file);
        self.append_event(&ev)
    }

    /// Attach each media file to each set (cartesian)
    pub fn attach_many(
        &self,
        set_keys: &[SetKey],
        media_files: &[String],
    ) -> Result<(), AttachmentIoError> {
        for sk in set_keys {
            for mf in media_files {
                self.attach(sk.clone(), mf.clone())?;
            }
        }
        Ok(())
    }

    /// Detach each media file from each set (cartesian)
    pub fn detach_many(
        &self,
        set_keys: &[SetKey],
        media_files: &[String],
    ) -> Result<(), AttachmentIoError> {
        for sk in set_keys {
            for mf in media_files {
                self.detach(sk.clone(), mf.clone())?;
            }
        }
        Ok(())
    }
}

pub struct MediaAttachmentReader;

impl MediaAttachmentReader {
    pub fn read_all<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<MediaAttachmentEvent>, AttachmentIoError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(vec![]);
        }
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let ev: MediaAttachmentEvent = serde_json::from_str(&line)?;
            events.push(ev);
        }
        Ok(events)
    }

    /// Compute current mapping SetKey -> media files by applying events in order
    pub fn compute_current(events: &[MediaAttachmentEvent]) -> HashMap<SetKey, HashSet<String>> {
        let mut map: HashMap<SetKey, HashSet<String>> = HashMap::new();
        for ev in events {
            match ev.event {
                AttachmentEventKind::Attach => {
                    map.entry(ev.set_key.clone())
                        .or_default()
                        .insert(ev.media_file.clone());
                }
                AttachmentEventKind::Detach => {
                    if let Some(set) = map.get_mut(&ev.set_key) {
                        set.remove(&ev.media_file);
                        if set.is_empty() {
                            map.remove(&ev.set_key);
                        }
                    }
                }
            }
        }
        map
    }

    pub fn compute_current_from_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<HashMap<SetKey, HashSet<String>>, AttachmentIoError> {
        let events = Self::read_all(path)?;
        Ok(Self::compute_current(&events))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    fn sk(session: &str, ex: &str, set_num: u32) -> SetKey {
        SetKey {
            session_id: session.to_string(),
            ex_code: ex.to_string(),
            set_num,
        }
    }

    #[test]
    fn write_and_read_events() {
        let base = std::env::temp_dir().join(format!("att_test_{}", Uuid::new_v4()));
        fs::create_dir_all(&base).unwrap();
        let path = base.join("media_attachments.jsonl");
        let writer = MediaAttachmentWriter::new(&path);

        writer
            .attach(sk("S1", "BP.BB.FLAT", 1), "set1.mp4")
            .unwrap();
        writer
            .attach(sk("S1", "BP.BB.FLAT", 1), "alt_angle.mp4")
            .unwrap();
        writer.attach(sk("S1", "SQ.BB.HB", 2), "sq2.mp4").unwrap();
        writer
            .detach(sk("S1", "BP.BB.FLAT", 1), "alt_angle.mp4")
            .unwrap();

        let events = MediaAttachmentReader::read_all(&path).unwrap();
        assert_eq!(events.len(), 4);

        let current = MediaAttachmentReader::compute_current(&events);
        assert_eq!(current.len(), 2);
        let bp_files = current.get(&sk("S1", "BP.BB.FLAT", 1)).unwrap();
        assert!(bp_files.contains("set1.mp4"));
        assert!(!bp_files.contains("alt_angle.mp4"));
        let sq_files = current.get(&sk("S1", "SQ.BB.HB", 2)).unwrap();
        assert!(sq_files.contains("sq2.mp4"));
        // best-effort cleanup
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(&base);
    }
}
