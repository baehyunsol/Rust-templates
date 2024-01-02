#![allow(dead_code)]

use std::collections::hash_map;
use std::ffi::OsString;
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// ```nohighlight
///       File Already Exists    File Does not Exist
///
///     AA       Append                  Dies
///    AoC       Append                 Create
///    CoT      Truncate                Create
///     AC        Dies                  Create
/// ```
pub enum WriteMode {
    AlwaysAppend,
    AppendOrCreate,
    CreateOrTruncate,
    AlwaysCreate,
}

impl From<WriteMode> for OpenOptions {
    fn from(m: WriteMode) -> OpenOptions {
        let mut result = OpenOptions::new();

        match m {
            WriteMode::AlwaysAppend => { result.append(true); },
            WriteMode::AppendOrCreate => { result.append(true).create(true); }
            WriteMode::CreateOrTruncate => { result.write(true).truncate(true).create(true); }
            WriteMode::AlwaysCreate => { result.write(true).create_new(true); }
        }

        result
    }
}

pub fn read_bytes(path: &str) -> Result<Vec<u8>, FileError> {
    fs::read(path).map_err(|e| FileError::from_std(e, path))
}

pub fn read_string(path: &str) -> Result<String, FileError> {
    let mut s = String::new();

    match File::open(path) {
        Err(e) => Err(FileError::from_std(e, path)),
        Ok(mut f) => match f.read_to_string(&mut s) {
            Ok(_) => Ok(s),
            Err(e) => Err(FileError::from_std(e, path)),
        }
    }
}

pub fn write_bytes(path: &str, bytes: &[u8], write_mode: WriteMode) -> Result<(), FileError> {
    let option: OpenOptions = write_mode.into();

    match option.open(path) {
        Ok(mut f) => match f.write_all(bytes) {
            Ok(_) => Ok(()),
            Err(e) => Err(FileError::from_std(e, path)),
        },
        Err(e) => Err(FileError::from_std(e, path)),
    }
}

pub fn write_string(path: &str, s: &str, write_mode: WriteMode) -> Result<(), FileError> {
    write_bytes(path, s.as_bytes(), write_mode)
}

/// `a/b/c.d` -> `c`
pub fn file_name(path: &str) -> Result<String, FileError> {
    let path_buf = PathBuf::from_str(path).unwrap();  // it's infallible

    match path_buf.file_stem() {
        None => Ok(String::new()),
        Some(s) => match s.to_str() {
            Some(ext) => Ok(ext.to_string()),
            None => Err(FileError::os_str_err(s.to_os_string())),
        }
    }
}

/// `a/b/c.d` -> `d`
pub fn extension(path: &str) -> Result<Option<String>, FileError> {
    let path_buf = PathBuf::from_str(path).unwrap();  // it's infallible

    match path_buf.extension() {
        None => Ok(None),
        Some(s) => match s.to_str() {
            Some(ext) => Ok(Some(ext.to_string())),
            None => Err(FileError::os_str_err(s.to_os_string())),
        }
    }
}

/// `a/b/c.d` -> `c.d`
pub fn basename(path: &str) -> Result<String, FileError> {
    let path_buf = PathBuf::from_str(path).unwrap();  // it's infallible

    match path_buf.file_name() {
        None => Ok(String::new()),  // when the path terminates in `..`
        Some(s) => match s.to_str() {
            Some(ext) => Ok(ext.to_string()),
            None => Err(FileError::os_str_err(s.to_os_string())),
        }
    }
}

/// `a/b/`, `c.d` -> `a/b/c.d`
pub fn join(path: &str, child: &str) -> Result<String, FileError> {
    let mut path_buf = PathBuf::from_str(path).unwrap();  // Infallible
    let child = PathBuf::from_str(child).unwrap();  // Infallible

    path_buf.push(child);

    match path_buf.to_str() {
        Some(result) => Ok(result.to_string()),
        None => Err(FileError::os_str_err(path_buf.into_os_string())),
    }
}

/// `a/b/c.d, e` -> `a/b/c.e`
pub fn set_ext(path: &str, ext: &str) -> Result<String, FileError> {
    let mut path_buf = PathBuf::from_str(path).unwrap();  // Infallible

    if path_buf.set_extension(ext) {
        match path_buf.to_str() {
            Some(result) => Ok(result.to_string()),
            None => Err(FileError::os_str_err(path_buf.into_os_string())),
        }
    } else {
        // has no filename
        Ok(path.to_string())
    }
}

/// It returns `false` if `path` doesn't exist
pub fn is_dir(path: &str) -> bool {
    PathBuf::from_str(path).map(|path| path.is_dir()).unwrap_or(false)
}

pub fn exists(path: &str) -> bool {
    PathBuf::from_str(path).map(|path| path.exists()).unwrap_or(false)
}

/// `a/b/c.d` -> `a/b/`
pub fn parent(path: &str) -> String {
    let path = Path::new(path);

    path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| String::new())
}

pub fn create_dir(path: &str) -> Result<(), FileError> {
    fs::create_dir(path).map_err(|e| FileError::from_std(e, path))
}

pub fn create_dir_all(path: &str) -> Result<(), FileError> {
    fs::create_dir_all(path).map_err(|e| FileError::from_std(e, path))
}

// it only returns the hash value of the modified time
pub fn last_modified(path: &str) -> Result<u64, FileError> {
    match fs::metadata(path) {
        Ok(m) => match m.modified() {
            Ok(m) => {
                let mut hasher = hash_map::DefaultHasher::new();
                m.hash(&mut hasher);
                let hash = hasher.finish();

                Ok(hash)
            },
            Err(e) => Err(FileError::from_std(e, path)),
        },
        Err(e) => Err(FileError::from_std(e, path)),
    }
}

pub fn read_dir(path: &str) -> Result<Vec<String>, FileError> {
    match fs::read_dir(path) {
        Err(e) => Err(FileError::from_std(e, path)),
        Ok(entries) => {
            let mut result = vec![];

            for entry in entries {
                match entry {
                    Err(e) => {
                        return Err(FileError::from_std(e, path));
                    }
                    Ok(e) => {
                        if let Some(ee) = e.path().to_str() {
                            result.push(ee.to_string());
                        }
                    }
                }
            }

            result.sort();
            Ok(result)
        }
    }
}

pub fn remove_file(path: &str) -> Result<(), FileError> {
    fs::remove_file(path).map_err(|e| FileError::from_std(e, path))
}

pub fn remove_dir(path: &str) -> Result<(), FileError> {
    fs::remove_dir(path).map_err(|e| FileError::from_std(e, path))
}

pub fn remove_dir_all(path: &str) -> Result<(), FileError> {
    fs::remove_dir_all(path).map_err(|e| FileError::from_std(e, path))
}

#[derive(Clone,  PartialEq)]
pub struct FileError {
    pub kind: FileErrorKind,
    pub given_path: Option<String>,
}

impl FileError {
    pub fn from_std(e: io::Error, given_path: &str) -> Self {
        let kind = match e.kind() {
            io::ErrorKind::NotFound => FileErrorKind::FileNotFound,
            io::ErrorKind::PermissionDenied => FileErrorKind::PermissionDenied,
            io::ErrorKind::AlreadyExists => FileErrorKind::AlreadyExists,
            _ => panic!("e: {e:?}, path: {given_path}"),
        };

        FileError {
            kind,
            given_path: Some(given_path.to_string()),
        }
    }

    pub(crate) fn os_str_err(os_str: OsString) -> Self {
        FileError {
            kind: FileErrorKind::OsStrErr(os_str),
            given_path: None,
        }
    }

    pub fn render_error(&self) -> String {
        let path = match self.kind {
            FileErrorKind::FileNotFound
            | FileErrorKind::PermissionDenied
            | FileErrorKind::AlreadyExists => {
                self.given_path.as_ref().unwrap().to_string()
            },
            FileErrorKind::OsStrErr(_) => String::new(),
        };

        match &self.kind {
            FileErrorKind::FileNotFound => format!(
                "file not found: `{path}`"
            ),
            FileErrorKind::PermissionDenied => format!(
                "permission denied: `{path}`",
            ),
            FileErrorKind::AlreadyExists => format!(
                "file already exists: `{path}`"
            ),
            FileErrorKind::OsStrErr(os_str) => format!(
                "error converting os_str: `{os_str:?}`"
            ),
        }
    }
}

impl fmt::Debug for FileError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.render_error())
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.render_error())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileErrorKind {
    FileNotFound,
    PermissionDenied,
    AlreadyExists,
    OsStrErr(OsString),
}
