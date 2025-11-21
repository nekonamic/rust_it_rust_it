use std::path::{Path, PathBuf};

use bevy::prelude::*;
use bms_rs::bms::model::Header;

pub struct BmsEntry {
    pub header: Header,
    pub path: PathBuf,
}

#[derive(Resource)]
pub struct BmsLib {
    pub cursor: u32,
    pub bms_arr: Vec<BmsEntry>,
}

impl BmsLib {
    pub fn cursor_entry(&self) -> Option<&BmsEntry> {
        self.bms_arr.get(self.cursor as usize)
    }
}

impl BmsLib {
    pub fn cursor_dir(&self) -> Option<&Path> {
        self.bms_arr
            .get(self.cursor as usize)
            .and_then(|entry| entry.path.parent())
    }
}
