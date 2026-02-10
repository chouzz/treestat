use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use treestat::cli::HeaderMode;
use treestat::lang::apply_header_mode;
use treestat::model::DirData;
use treestat::scanner::compute_tree_counts;

#[test]
fn header_modes_work() {
    let mut set = ["c", "cpp", "h", "hpp"]
        .into_iter()
        .map(str::to_string)
        .collect();
    apply_header_mode(&mut set, HeaderMode::Exclude);
    assert!(set.contains("cpp"));
    assert!(!set.contains("h"));

    let mut set = ["c", "cpp", "h", "hpp"]
        .into_iter()
        .map(str::to_string)
        .collect();
    apply_header_mode(&mut set, HeaderMode::Only);
    assert!(!set.contains("c"));
    assert!(set.contains("hpp"));
}

#[test]
fn aggregate_counts() {
    let root = PathBuf::from("/tmp/root");
    let child = PathBuf::from("/tmp/root/src");
    let mut dirs = HashMap::new();
    dirs.insert(
        root.clone(),
        DirData {
            name: "root".to_string(),
            children: [child.clone()].into_iter().collect(),
            direct_files: 1,
        },
    );
    dirs.insert(
        child.clone(),
        DirData {
            name: "src".to_string(),
            children: BTreeSet::new(),
            direct_files: 2,
        },
    );

    let counts = compute_tree_counts(&root, &dirs);
    assert_eq!(counts.get(&root), Some(&3));
    assert_eq!(counts.get(&child), Some(&2));
}
