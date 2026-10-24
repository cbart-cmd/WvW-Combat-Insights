//! Existing user storage locations; never relative to the source checkout.

use super::project;
use std::path::PathBuf;

pub fn documents() -> Option<PathBuf> {
    use windows::Win32::{
        System::Com::CoTaskMemFree,
        UI::Shell::{FOLDERID_Documents, KF_FLAG_DEFAULT, SHGetKnownFolderPath},
    };
    unsafe {
        let path = SHGetKnownFolderPath(&FOLDERID_Documents, KF_FLAG_DEFAULT, None).ok()?;
        let text = path.to_string().ok();
        CoTaskMemFree(Some(path.0 as _));
        text.map(PathBuf::from)
    }
}

pub fn data_dir() -> Option<PathBuf> {
    documents().map(|docs| {
        docs.join("Guild Wars 2")
            .join("addons")
            .join(project::DATA_DIRECTORY)
    })
}

pub fn settings_file() -> Option<PathBuf> {
    data_dir().map(|dir| dir.join(project::SETTINGS_FILE))
}
