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

use std::env;
use std::{fs, path::Path};
use std::{io, path::PathBuf};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum UtilError {
    #[error("Operation failed")]
    OpFailed {},
}

pub fn get_process_comm(pid: i32) -> Result<String> {
    Ok(
        std::fs::read_to_string(Path::new(&format!("/proc/{}/comm", pid)))?
            .trim()
            .to_string(),
    )
}

pub fn get_process_file_name(pid: i32) -> Result<String> {
    let tmp = format!("/proc/{}/exe", pid);
    let filename = Path::new(&tmp);
    let result = nix::fcntl::readlink(filename);

    Ok(result
        .map_err(|_| UtilError::OpFailed {})?
        .into_string()
        .map_err(|_| UtilError::OpFailed {})?)
}

pub fn tilde_expand(path: &str) -> Result<PathBuf> {
    let home = env::var("HOME")?;

    let result = path.replacen("~", &home, 1);
    let result = PathBuf::from(result);

    Ok(result)
}

pub fn create_dir<P: AsRef<Path>>(path: &P) -> io::Result<()> {
    let path = path.as_ref();

    fs::create_dir_all(&path)
}

pub fn create_rules_file_if_not_exists<P: AsRef<Path>>(path: &P) -> io::Result<()> {
    let path = path.as_ref();

    if fs::metadata(&path).is_err() {
        fs::write(&path, "[]")?;
    }

    Ok(())
}
