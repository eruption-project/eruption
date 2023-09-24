/*  SPDX-License-Identifier: GPL-3.0-or-later  */

/*
    This file is part of Eruption.

    Eruption is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Eruption is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use nix::fcntl::{flock, open, FlockArg, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::{ftruncate, getpid, write};
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::constants;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum UtilError {
    #[error("File not found: {description}")]
    FileNotFound { description: String },

    #[error("Read failed: {description}")]
    FileReadError {
        #[source]
        source: io::Error,
        description: String,
    },

    #[error("Not a file")]
    NotAFile {},

    #[error("Write failed: {description}")]
    FileWriteError {
        #[source]
        source: io::Error,
        description: String,
    },
}

/// Write out the current process' PID to the .pid file at `/run/eruption/eruption.pid`
pub fn write_pid_file() -> Result<()> {
    let pid = getpid().as_raw();
    let text = format!("{}", pid);

    let fd = open(
        &PathBuf::from(constants::PID_FILE),
        OFlag::O_CREAT | OFlag::O_WRONLY,
        Mode::from_bits(0o666).unwrap(),
    )?;

    flock(fd, FlockArg::LockExclusiveNonblock)?;
    ftruncate(fd, 0)?;

    write(fd, text.as_bytes())?;

    Ok(())
}

/// Returns the associated manifest path in `PathBuf` for the script `script_path`.
pub fn get_manifest_for(script_file: &Path) -> PathBuf {
    let mut manifest_path = script_file.to_path_buf();
    manifest_path.set_extension("lua.manifest");

    manifest_path
}

pub fn file_exists<P: AsRef<Path>>(p: P) -> bool {
    p.as_ref().exists()
}

pub fn demand_file_is_accessible<P: AsRef<Path>>(p: P) -> Result<()> {
    // Does the path exist?
    let path = match fs::canonicalize(p) {
        Ok(path) => path,
        Err(e) => {
            return Err(UtilError::FileReadError {
                source: e,
                description: "Could not find file".to_owned(),
            }
            .into())
        }
    };

    // Is the metadata readable?
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(e) => {
            return Err(UtilError::FileReadError {
                source: e,
                description: "Could not read metadata".to_owned(),
            }
            .into())
        }
    };

    // Is the path a regular file?  (Symlinks will have been canonicalized to regular files.)
    if !metadata.is_file() {
        return Err(UtilError::NotAFile {}.into());
    }

    // Is the file readable?
    match fs::File::open(&path) {
        Err(e) => {
            return Err(UtilError::FileReadError {
                source: e,
                description: "Could not open file".to_owned(),
            }
            .into())
        }
        _ => {}
    };

    Ok(())
}

/// write `data` to file `filename`
pub fn write_file<P: AsRef<Path>>(path: &P, data: &String) -> Result<()> {
    let path = path.as_ref();

    log::info!("Writing to file: {}", &path.display());

    fs::write(path, data).map_err(|e| UtilError::FileWriteError {
        description: format!("{}", e),
        source: e,
    })?;

    Ok(())
}

pub fn get_script_dirs() -> Vec<PathBuf> {
    let mut result = vec![];

    let config = crate::CONFIG.lock();

    let script_dirs = config
        .as_ref()
        .map(|c| {
            c.get::<Vec<String>>("global.script_dirs")
                .unwrap_or_else(|_| vec![])
        })
        .unwrap_or_default();

    let mut script_dirs = script_dirs
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>();

    result.append(&mut script_dirs);

    // if we could not determine a valid set of paths, use a hard coded fallback instead
    if result.is_empty() {
        log::warn!("Using default fallback script directory");

        let path = PathBuf::from(constants::DEFAULT_SCRIPT_DIR);
        result.push(path);
    }

    result
}

pub fn match_script_path<P: AsRef<Path>>(script_file: &P) -> Result<PathBuf> {
    let script_file = script_file.as_ref();

    for dir in get_script_dirs().iter() {
        let script_path = dir.join(script_file);

        if let Ok(metadata) = fs::metadata(&script_path) {
            if metadata.is_file() {
                return Ok(script_path);
            }
        }
    }

    Err(UtilError::FileNotFound {
        description: format!(
            "Could not find file in search path(s): {}",
            &script_file.display()
        ),
    }
    .into())
}

/// Provide a simple means to rate-limit log output
pub mod ratelimited {
    use lazy_static::lazy_static;
    use parking_lot::RwLock;
    use std::{
        collections::{hash_map::Entry, HashMap},
        sync::Arc,
        time::{Duration, Instant},
    };

    const LIMIT_MSGS_PER_MIN: u64 = 2;

    struct Metadata {
        last: Instant,
        count: usize,
    }

    lazy_static! {
        static ref LAST_LOG_MAP: Arc<RwLock<HashMap<String, Metadata>>> =
            Arc::new(RwLock::new(HashMap::new()));
    }

    #[allow(unused)]
    #[macro_export]
    macro_rules! trace {
        ($p:expr) => {{
            let (within_limit, count) = ratelimited::is_within_rate_limit($p);

            if within_limit {
                log::trace!($p);
                if count > 1 {
                    log::trace!("The previous message was rate-limited [{count}]");
                }
            } else {
                // rate-limit reached
            }
        }};

        ($p:expr, $($pn:expr),+) => {{
            let (within_limit, count) = ratelimited::is_within_rate_limit($p);

            if within_limit {
                log::trace!($p, $($pn),+);
                if count > 1 {
                    log::trace!("The previous message was rate-limited [{count}]");
                }
            } else {
                // rate-limit reached
            }
        }};
    }

    #[allow(unused)]
    #[macro_export]
    macro_rules! debug {
        ($p:expr) => {{
            let (within_limit, count) = ratelimited::is_within_rate_limit($p);

            if within_limit {
                log::debug!($p);
                if count > 1 {
                    log::debug!("The previous message was rate-limited [{count}]");
                }
            } else {
                // rate-limit reached
            }
        }};

        ($p:expr, $($pn:expr),+) => {{
            let (within_limit, count) = ratelimited::is_within_rate_limit($p);

            if within_limit {
                log::debug!($p, $($pn),+);
                if count > 1 {
                    log::debug!("The previous message was rate-limited [{count}]");
                }
            } else {
                // rate-limit reached
            }
        }};
    }

    #[allow(unused)]
    #[macro_export]
    macro_rules! info {
        ($p:expr) => {{
            let (within_limit, count) = ratelimited::is_within_rate_limit($p);

            if within_limit {
                log::info!($p);
                if count > 1 {
                    log::info!("The previous message was rate-limited [{count}]");
                }
            } else {
                // rate-limit reached
            }
        }};

        ($p:expr, $($pn:expr),+) => {{
            let (within_limit, count) = ratelimited::is_within_rate_limit($p);

            if within_limit {
                log::info!($p, $($pn),+);
                if count > 1 {
                    log::info!("The previous message was rate-limited [{count}]");
                }
            } else {
                // rate-limit reached
            }
        }};
    }

    #[allow(unused)]
    #[macro_export]
    macro_rules! warning {
        ($p:expr) => {{
            let (within_limit, count) = ratelimited::is_within_rate_limit($p);

            if within_limit {
                log::warn!($p);
                if count > 1 {
                    log::warn!("The previous message was rate-limited [{count}]");
                }
            } else {
                // rate-limit reached
            }
        }};

        ($p:expr, $($pn:expr),+) => {{
            let (within_limit, count) = ratelimited::is_within_rate_limit($p);

            if within_limit {
                log::warn!($p, $($pn),+);
                if count > 1 {
                    log::warn!("The previous message was rate-limited [{count}]");
                }
            } else {
                // rate-limit reached
            }
        }};
    }

    #[allow(unused)]
    #[macro_export]
    macro_rules! error {
        ($p:expr) => {{
            let (within_limit, count) = ratelimited::is_within_rate_limit($p);

            if within_limit {
                log::error!($p);
                if count > 1 {
                    log::error!("The previous message was rate-limited [{count}]");
                }
            } else {
                // rate-limit reached
            }
        }};

        ($p:expr, $($pn:expr),+) => {{
            let (within_limit, count) = ratelimited::is_within_rate_limit($p);

            if within_limit {
                log::error!($p, $($pn),+);
                if count > 1 {
                    log::error!("The previous message was rate-limited [{count}]");
                }
            } else {
                // rate-limit reached
            }
        }};
    }

    #[allow(unused)]
    pub(crate) use debug;
    #[allow(unused)]
    pub(crate) use error;
    #[allow(unused)]
    pub(crate) use info;
    #[allow(unused)]
    pub(crate) use trace;
    #[allow(unused)]
    pub(crate) use warning as warn;

    pub(crate) fn is_within_rate_limit(p: &str) -> (bool, usize) {
        let mut map = LAST_LOG_MAP.write();
        let e = map.entry(p.to_string());

        match e {
            Entry::Occupied(mut e) => {
                let e = e.get_mut();
                let result = e.last.elapsed() > Duration::from_secs(60 / LIMIT_MSGS_PER_MIN);

                e.count += 1;
                let count = e.count;

                if result {
                    e.count = 0;
                    e.last = Instant::now();
                }

                (result, count)
            }

            Entry::Vacant(e) => {
                e.insert(Metadata {
                    count: 1,
                    last: Instant::now(),
                });

                (true, 1)
            }
        }
    }
}
