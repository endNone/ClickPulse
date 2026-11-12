//! Bounded application diagnostics, shared with the periodic activity log.
use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

static LOG_WRITE_LOCK: Mutex<()> = Mutex::new(());

/// Keep desktop failures available after a normal GUI launch without stderr.
/// An absent path deliberately keeps excluded/test sessions off the user's log.
pub fn error(path: Option<&Path>, message: &str) {
    // External errors must remain a bounded, single-line record.
    let mut chars = message.chars();
    let mut message: String = chars
        .by_ref()
        .take(4096)
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect();
    if chars.next().is_some() {
        message.push_str("… [truncated]");
    }
    let line = format!(
        "{} ERROR {message}\n",
        chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    );
    eprint!("{line}");
    if let Some(path) = path
        && let Err(error) = append_status_log(path, &line, STATUS_LOG_MAX_BYTES)
    {
        eprintln!("Could not write application diagnostics: {error:#}");
    }
}

pub(crate) const STATUS_LOG_MAX_BYTES: u64 = 10 * 1024 * 1024;
pub(crate) const STATUS_LOG_BACKUPS: usize = 3;

pub(crate) fn status_log_archive(path: &Path, index: usize) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(format!(".{index}"));
    name.into()
}

fn bound_status_log(path: &Path, limit: u64) -> Result<u64> {
    use std::io::{Read, Seek, SeekFrom, Write};
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.into()),
    };
    anyhow::ensure!(metadata.is_file(), "The log path is not a regular file");
    if metadata.len() <= limit {
        return Ok(metadata.len());
    }
    // Keep the most recent diagnostics from older, unbounded log versions.
    // Copy forward with a small buffer; history remains entirely in SQLite.
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)?;
    let start = metadata.len() - limit;
    let mut copied = 0;
    let mut buffer = [0_u8; 64 * 1024];
    while copied < limit {
        let length = (limit - copied).min(buffer.len() as u64) as usize;
        file.seek(SeekFrom::Start(start + copied))?;
        file.read_exact(&mut buffer[..length])?;
        file.seek(SeekFrom::Start(copied))?;
        file.write_all(&buffer[..length])?;
        copied += length as u64;
    }
    file.set_len(limit)?;
    Ok(limit)
}

pub(crate) fn append_status_log(path: &Path, line: &str, limit: u64) -> Result<()> {
    use std::io::Write;
    // Rotation and append are one operation across the storage and UI threads.
    let _guard = LOG_WRITE_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    anyhow::ensure!(
        line.len() as u64 <= limit,
        "A log entry exceeds the size limit"
    );
    std::fs::create_dir_all(path.parent().context("Invalid log path")?)?;
    let current_size = bound_status_log(path, limit)?;
    for index in 1..=STATUS_LOG_BACKUPS {
        bound_status_log(&status_log_archive(path, index), limit)?;
    }
    if current_size + line.len() as u64 > limit {
        let oldest = status_log_archive(path, STATUS_LOG_BACKUPS);
        match std::fs::remove_file(oldest) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        for index in (1..STATUS_LOG_BACKUPS).rev() {
            match std::fs::rename(
                status_log_archive(path, index),
                status_log_archive(path, index + 1),
            ) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        std::fs::rename(path, status_log_archive(path, 1))?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    file.write_all(line.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concurrent_writers_rotate_complete_records_within_the_size_bound() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("clickpulse.log");
        std::thread::scope(|scope| {
            for writer in 0..4 {
                let path = &path;
                scope.spawn(move || {
                    for index in 0..32 {
                        append_status_log(path, &format!("w{writer}:{index:04}\n"), 128).unwrap();
                    }
                });
            }
        });
        let mut records = std::collections::HashSet::new();
        for file in std::fs::read_dir(temp.path()).unwrap() {
            let file = file.unwrap();
            assert_eq!(file.metadata().unwrap().len(), 128);
            for line in std::fs::read_to_string(file.path()).unwrap().lines() {
                let (writer, index) = line.split_once(':').unwrap();
                assert!(matches!(writer, "w0" | "w1" | "w2" | "w3"));
                assert!(index.parse::<usize>().unwrap() < 32);
                assert!(records.insert(line.to_owned()));
            }
        }
        assert_eq!(records.len(), 64);
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 4);
    }

    #[test]
    fn external_error_details_are_bounded_single_line_utf8() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("logs/clickpulse.log");
        error(
            Some(&path),
            &format!("offline\r\n\x1b[31m{}", "界".repeat(5000)),
        );
        let contents = std::fs::read_to_string(path).unwrap();
        assert_eq!(contents.lines().count(), 1);
        assert!(contents.contains("ERROR offline   [31m"));
        assert!(contents.ends_with("… [truncated]\n"));
        assert!(contents.len() < 16 * 1024);
    }
}
