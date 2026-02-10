use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct DirData {
    pub name: String,
    pub children: BTreeSet<PathBuf>,
    pub direct_files: usize,
}

#[derive(Debug)]
pub struct ScanResult {
    pub root: PathBuf,
    pub dirs: HashMap<PathBuf, DirData>,
    pub total_files: usize,
    pub dirs_with_files: usize,
}
