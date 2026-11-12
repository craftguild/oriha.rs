use chrono::{DateTime, Utc};
use oriha_stats_schema::ValidatedUsageAnalyticsPayload;
use serde::Serialize;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredUsageAnalyticsRecord {
    pub received_at: DateTime<Utc>,
    pub storage_id: Uuid,
    pub digest: String,
    pub payload: ValidatedUsageAnalyticsPayload,
}

impl StoredUsageAnalyticsRecord {
    pub fn new(digest: String, payload: ValidatedUsageAnalyticsPayload) -> Self {
        Self {
            received_at: Utc::now(),
            storage_id: Uuid::new_v4(),
            digest,
            payload,
        }
    }
}

#[derive(Clone)]
pub struct StatsWriterHandle {
    sender: mpsc::Sender<WriteRequest>,
}

impl StatsWriterHandle {
    pub fn start(data_dir: PathBuf, rotate_seconds: u64, rotate_bytes: u64) -> Self {
        let (sender, mut receiver) = mpsc::channel::<WriteRequest>(1024);
        tokio::spawn(async move {
            let stale_recovery = seal_stale_open_chunks(&data_dir);
            let mut writer = JsonlChunkWriter::new(data_dir, rotate_seconds, rotate_bytes);
            while let Some(request) = receiver.recv().await {
                match request {
                    WriteRequest::Append { record, response } => {
                        let result = match &stale_recovery {
                            Ok(()) => writer.append(record.as_ref()).await,
                            Err(err) => Err(WriterError::Io(std::io::Error::new(
                                err.kind(),
                                err.to_string(),
                            ))),
                        };
                        let _ = response.send(result);
                    }
                    WriteRequest::Shutdown { response } => {
                        let result = writer.rotate().await;
                        let _ = response.send(result);
                        return;
                    }
                }
            }
            let _ = writer.rotate().await;
        });

        Self { sender }
    }

    pub async fn write(&self, record: StoredUsageAnalyticsRecord) -> Result<(), WriterError> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(WriteRequest::Append {
                record: Box::new(record),
                response,
            })
            .await
            .map_err(|_| WriterError::Closed)?;
        receiver.await.map_err(|_| WriterError::Closed)?
    }

    pub async fn shutdown(&self) -> Result<(), WriterError> {
        let (response, receiver) = oneshot::channel();
        self.sender
            .send(WriteRequest::Shutdown { response })
            .await
            .map_err(|_| WriterError::Closed)?;
        receiver.await.map_err(|_| WriterError::Closed)?
    }
}

enum WriteRequest {
    Append {
        record: Box<StoredUsageAnalyticsRecord>,
        response: oneshot::Sender<Result<(), WriterError>>,
    },
    Shutdown {
        response: oneshot::Sender<Result<(), WriterError>>,
    },
}

pub struct JsonlChunkWriter {
    data_dir: PathBuf,
    rotate_seconds: u64,
    rotate_bytes: u64,
    host: String,
    pid: u32,
    sequence: u64,
    active: Option<ActiveChunk>,
}

struct ActiveChunk {
    file: File,
    open_path: PathBuf,
    partition_dir: PathBuf,
    opened_at: DateTime<Utc>,
    bytes_written: u64,
}

impl JsonlChunkWriter {
    pub fn new(data_dir: PathBuf, rotate_seconds: u64, rotate_bytes: u64) -> Self {
        Self {
            data_dir,
            rotate_seconds,
            rotate_bytes,
            host: sanitized_host(),
            pid: std::process::id(),
            sequence: 0,
            active: None,
        }
    }

    pub async fn append(&mut self, record: &StoredUsageAnalyticsRecord) -> Result<(), WriterError> {
        let mut line = serde_json::to_vec(record).map_err(WriterError::Serialize)?;
        line.push(b'\n');
        let now = Utc::now();
        self.ensure_active(now, line.len() as u64).await?;

        let active = self.active.as_mut().ok_or(WriterError::Closed)?;
        active
            .file
            .write_all(&line)
            .await
            .map_err(WriterError::Io)?;
        active.file.flush().await.map_err(WriterError::Io)?;
        active.bytes_written += line.len() as u64;
        Ok(())
    }

    pub async fn rotate(&mut self) -> Result<(), WriterError> {
        if let Some(mut active) = self.active.take() {
            active.file.flush().await.map_err(WriterError::Io)?;
            active.file.sync_all().await.map_err(WriterError::Io)?;
            drop(active.file);

            let final_path = sealed_path_for_open_path(&active.open_path);
            fs::rename(&active.open_path, final_path)
                .await
                .map_err(WriterError::Io)?;
        }
        Ok(())
    }

    async fn ensure_active(
        &mut self,
        now: DateTime<Utc>,
        next_bytes: u64,
    ) -> Result<(), WriterError> {
        let partition_dir = partition_dir(&self.data_dir, now);
        let should_rotate = match &self.active {
            None => false,
            Some(active) => {
                active.partition_dir != partition_dir
                    || now
                        .signed_duration_since(active.opened_at)
                        .num_seconds()
                        .max(0) as u64
                        >= self.rotate_seconds
                    || active.bytes_written.saturating_add(next_bytes) > self.rotate_bytes
            }
        };

        if should_rotate {
            self.rotate().await?;
        }

        if self.active.is_none() {
            self.open_active(partition_dir, now).await?;
        }

        Ok(())
    }

    async fn open_active(
        &mut self,
        partition_dir: PathBuf,
        now: DateTime<Utc>,
    ) -> Result<(), WriterError> {
        fs::create_dir_all(&partition_dir)
            .await
            .map_err(WriterError::Io)?;
        let (open_path, file) = self.create_next_part_file(&partition_dir).await?;
        let bytes_written = file.metadata().await.map_err(WriterError::Io)?.len();
        self.active = Some(ActiveChunk {
            file,
            open_path,
            partition_dir,
            opened_at: now,
            bytes_written,
        });
        Ok(())
    }

    async fn create_next_part_file(
        &mut self,
        partition_dir: &Path,
    ) -> Result<(PathBuf, File), WriterError> {
        loop {
            let open_path = partition_dir.join(format!(
                "part-{}-{}-{:06}.jsonl.open",
                self.host, self.pid, self.sequence
            ));
            self.sequence = self.sequence.saturating_add(1);

            if sealed_path_for_open_path(&open_path).exists() {
                continue;
            }

            match OpenOptions::new()
                .create_new(true)
                .append(true)
                .open(&open_path)
                .await
            {
                Ok(file) => return Ok((open_path, file)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(WriterError::Io(error)),
            }
        }
    }
}

pub async fn cleanup_old_raw_partitions(
    data_dir: &Path,
    retention_days: u64,
) -> Result<(), WriterError> {
    let raw_dir = data_dir.join("raw");
    let Ok(mut entries) = fs::read_dir(&raw_dir).await else {
        return Ok(());
    };
    let cutoff = Utc::now()
        .date_naive()
        .checked_sub_days(chrono::Days::new(retention_days))
        .ok_or(WriterError::InvalidRetention(retention_days))?;

    while let Some(entry) = entries.next_entry().await.map_err(WriterError::Io)? {
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        let Some(date) = name.strip_prefix("dt=") else {
            continue;
        };
        let Ok(date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
            continue;
        };
        if date < cutoff {
            fs::remove_dir_all(entry.path())
                .await
                .map_err(WriterError::Io)?;
        }
    }

    Ok(())
}

pub fn seal_stale_open_chunks(data_dir: &Path) -> Result<(), std::io::Error> {
    let raw_dir = data_dir.join("raw");
    if !raw_dir.exists() {
        return Ok(());
    }
    seal_stale_open_chunks_in_dir(&raw_dir)
}

fn seal_stale_open_chunks_in_dir(dir: &Path) -> Result<(), std::io::Error> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            seal_stale_open_chunks_in_dir(&path)?;
            continue;
        }
        let is_open_chunk = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".jsonl.open"));
        if !is_open_chunk {
            continue;
        }

        let mut final_path = sealed_path_for_open_path(&path);
        if final_path.exists() {
            final_path = recovered_path_for_open_path(&path);
        }
        std::fs::rename(path, final_path)?;
    }
    Ok(())
}

fn partition_dir(data_dir: &Path, now: DateTime<Utc>) -> PathBuf {
    data_dir
        .join("raw")
        .join(format!("dt={}", now.format("%Y-%m-%d")))
        .join(format!("hour={}", now.format("%H")))
}

fn sealed_path_for_open_path(open_path: &Path) -> PathBuf {
    let file_name = open_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("part.jsonl.open");
    if let Some(stripped) = file_name.strip_suffix(".open") {
        open_path.with_file_name(stripped)
    } else {
        open_path.with_extension("jsonl")
    }
}

fn recovered_path_for_open_path(open_path: &Path) -> PathBuf {
    let file_name = open_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("part.jsonl.open")
        .trim_end_matches(".open");
    open_path.with_file_name(format!("{file_name}.recovered-{}.jsonl", Uuid::new_v4()))
}

fn sanitized_host() -> String {
    let value = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "host".to_string());
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        "host".to_string()
    } else {
        sanitized
    }
}

#[derive(Debug)]
pub enum WriterError {
    Closed,
    Io(std::io::Error),
    Serialize(serde_json::Error),
    InvalidRetention(u64),
}

impl fmt::Display for WriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => write!(f, "stats writer is closed"),
            Self::Io(err) => write!(f, "stats writer I/O error: {err}"),
            Self::Serialize(err) => write!(f, "stats record serialization failed: {err}"),
            Self::InvalidRetention(days) => write!(f, "invalid retention days: {days}"),
        }
    }
}

impl Error for WriterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Serialize(err) => Some(err),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oriha_core::RenderStats;
    use oriha_stats_schema::ValidatedUsageAnalyticsPayload;

    fn record() -> StoredUsageAnalyticsRecord {
        StoredUsageAnalyticsRecord::new(
            "digest".to_string(),
            ValidatedUsageAnalyticsPayload {
                event: "markdown_render_complete".to_string(),
                schema_version: 1,
                platform: "macOS".to_string(),
                app_version: "1.0".to_string(),
                build_version: "1".to_string(),
                locale: "ja".to_string(),
                subscription_plan: "free".to_string(),
                event_id: None,
                stats: RenderStats::default(),
            },
        )
    }

    #[tokio::test]
    async fn writer_rotates_open_file_to_sealed_jsonl() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut writer = JsonlChunkWriter::new(temp.path().to_path_buf(), 900, 20);

        writer.append(&record()).await.expect("first append");
        writer
            .append(&record())
            .await
            .expect("second append rotates");
        writer.rotate().await.expect("final rotate");

        let mut sealed = Vec::new();
        collect_files(temp.path(), &mut sealed);
        assert!(sealed.iter().any(|path| path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".jsonl"))));
        assert!(!sealed.iter().any(|path| path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".open"))));
    }

    #[tokio::test]
    async fn writer_does_not_overwrite_existing_sealed_part() {
        let temp = tempfile::tempdir().expect("tempdir");
        let mut first = JsonlChunkWriter::new(temp.path().to_path_buf(), 900, 20);
        first.append(&record()).await.expect("first append");
        first.rotate().await.expect("first rotate");

        let sealed_before = collect_file_names(temp.path(), ".jsonl");
        assert_eq!(sealed_before.len(), 1);

        let mut second = JsonlChunkWriter::new(temp.path().to_path_buf(), 900, 20);
        second.append(&record()).await.expect("second append");
        second.rotate().await.expect("second rotate");

        let sealed_after = collect_file_names(temp.path(), ".jsonl");
        assert_eq!(sealed_after.len(), 2);
        assert!(sealed_before.iter().all(|name| sealed_after.contains(name)));
    }

    #[test]
    fn stale_open_chunks_are_sealed_on_startup_recovery() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir = temp
            .path()
            .join("raw")
            .join("dt=2026-05-18")
            .join("hour=10");
        std::fs::create_dir_all(&dir).expect("create partition");
        let open = dir.join("part-host-1-000000.jsonl.open");
        std::fs::write(&open, "{}\n").expect("write open file");

        seal_stale_open_chunks(temp.path()).expect("seal stale open chunks");

        assert!(!open.exists());
        assert!(dir.join("part-host-1-000000.jsonl").exists());
    }

    #[tokio::test]
    async fn writer_shutdown_seals_active_chunk() {
        let temp = tempfile::tempdir().expect("tempdir");
        let handle = StatsWriterHandle::start(temp.path().to_path_buf(), 900, 67_108_864);

        handle.write(record()).await.expect("write record");
        handle.shutdown().await.expect("shutdown writer");

        let sealed = collect_file_names(temp.path(), ".jsonl");
        assert_eq!(sealed.len(), 1);
        assert!(collect_file_names(temp.path(), ".jsonl.open").is_empty());
    }

    fn collect_files(path: &Path, output: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(path).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, output);
            } else {
                output.push(path);
            }
        }
    }

    fn collect_file_names(path: &Path, suffix: &str) -> Vec<String> {
        let mut files = Vec::new();
        collect_files(path, &mut files);
        let mut names = files
            .iter()
            .filter_map(|path| path.file_name().and_then(|name| name.to_str()))
            .filter(|name| name.ends_with(suffix))
            .map(str::to_string)
            .collect::<Vec<_>>();
        names.sort();
        names
    }
}
