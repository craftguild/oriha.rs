use crate::error::ViewerError;
use crate::model::StoredRecord;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub(crate) fn collect_stats_files(
    inputs: &[PathBuf],
    include_open: bool,
) -> Result<Vec<PathBuf>, ViewerError> {
    let mut files = Vec::new();
    for input in inputs {
        let metadata = fs::metadata(input).map_err(|source| ViewerError::Io {
            path: input.clone(),
            source,
        })?;
        if metadata.is_dir() {
            collect_stats_files_in_dir(input, include_open, &mut files)?;
        } else if metadata.is_file() {
            files.push(input.clone());
        }
    }

    files.sort();
    files.dedup();
    if files.is_empty() {
        return Err(ViewerError::NoStatsFiles {
            inputs: inputs.to_vec(),
            include_open,
        });
    }
    Ok(files)
}

fn collect_stats_files_in_dir(
    dir: &Path,
    include_open: bool,
    output: &mut Vec<PathBuf>,
) -> Result<(), ViewerError> {
    let mut entries = fs::read_dir(dir)
        .map_err(|source| ViewerError::Io {
            path: dir.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| ViewerError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| ViewerError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_dir() {
            collect_stats_files_in_dir(&path, include_open, output)?;
        } else if file_type.is_file() && is_stats_file(&path, include_open) {
            output.push(path);
        }
    }
    Ok(())
}

fn is_stats_file(path: &Path, include_open: bool) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    name.ends_with(".jsonl") || (include_open && name.ends_with(".jsonl.open"))
}

pub(crate) fn read_stats_file(
    path: &Path,
    skip_invalid: bool,
    mut on_record: impl FnMut(&StoredRecord),
) -> Result<u64, ViewerError> {
    let file = fs::File::open(path).map_err(|source| ViewerError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut skipped = 0_u64;
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|source| ViewerError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<StoredRecord>(line) {
            Ok(record) => on_record(&record),
            Err(_source) if skip_invalid => skipped += 1,
            Err(source) => {
                return Err(ViewerError::InvalidJsonLine {
                    path: path.to_path_buf(),
                    line: index + 1,
                    source,
                });
            }
        }
    }
    Ok(skipped)
}
