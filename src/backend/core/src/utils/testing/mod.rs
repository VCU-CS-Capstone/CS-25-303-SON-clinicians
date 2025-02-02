use std::path::PathBuf;
#[cfg(test)]
pub(crate) mod config;

pub mod db;
pub fn find_file_with_name_check_parents(
    path: PathBuf,
    name: &str,
    parents_left_to_check: usize,
) -> Option<PathBuf> {
    if parents_left_to_check == 0 {
        return None;
    }
    let possible_path = path.join(name);
    if possible_path.exists() {
        return Some(possible_path);
    }
    let Some(parent_dir) = path.parent() else {
        println!("No parent dir found for path: {:?}", path);
        return None;
    };
    find_file_with_name_check_parents(parent_dir.to_path_buf(), name, parents_left_to_check - 1)
}
