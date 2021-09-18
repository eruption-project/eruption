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
*/

use std::path::{Path, PathBuf};
use std::{fs, io};

use nix::fcntl::{flock, open, FlockArg, OFlag};
use nix::sys::stat::Mode;
use nix::unistd::{ftruncate, getpid, write};

use crate::constants;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum UtilError {
    #[error("File not found: {description}")]
    FileNotFound { description: String },

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

    write(fd, &text.as_bytes())?;

    Ok(())
}

/// Returns the associated manifest path in `PathBuf` for the script `script_path`.
pub fn get_manifest_for(script_file: &Path) -> PathBuf {
    let mut manifest_path = script_file.to_path_buf();
    manifest_path.set_extension("lua.manifest");

    manifest_path
}

pub fn is_file_accessible<P: AsRef<Path>>(p: P) -> std::io::Result<String> {
    fs::read_to_string(p)
}

/// Checks whether a script file is readable
#[allow(dead_code)]
pub fn is_script_file_accessible(script_file: &Path) -> bool {
    is_file_accessible(script_file).is_ok()
}

/// Checks whether a script's manifest file is readable
#[allow(dead_code)]
pub fn is_manifest_file_accessible(script_file: &Path) -> bool {
    fs::read_to_string(get_manifest_for(script_file)).is_ok()
}

/// write `data` to file `filename`
pub fn write_file<P: AsRef<Path>>(path: &P, data: &String) -> Result<()> {
    let path = path.as_ref();

    log::info!("Writing to file: {}", &path.display());

    fs::write(&path, &data).map_err(|e| UtilError::FileWriteError {
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
        .unwrap()
        .get::<Vec<String>>("global.script_dirs")
        .unwrap_or_else(|_| vec![]);

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

    let mut result = Err(UtilError::FileNotFound {
        description: format!(
            "Could not find file in search path(s): {}",
            &script_file.display()
        ),
    }
    .into());

    'DIR_LOOP: for dir in get_script_dirs().iter() {
        let script_path = dir.join(&script_file);

        if let Ok(metadata) = fs::metadata(&script_path) {
            if metadata.is_file() {
                result = Ok(script_path);
                break 'DIR_LOOP;
            }
        }
    }

    result
}
