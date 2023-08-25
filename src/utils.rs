use std::path::{Path, PathBuf};


pub fn pathbuf_to_string(pathbuf: &Path, desired_length: usize, error_str: &str) -> String {
    let filestem: &str = pathbuf
        .file_stem()
        .unwrap_or(error_str.as_ref())
        .to_str()
        .unwrap_or(error_str);
    if filestem.len() > desired_length {
        String::from_iter([&filestem[0..desired_length - 2], ".."])
    } else {
        String::from(filestem)
    }
}

pub fn pathbuf_helper(pathbuf: &Path, prefix: &Path, extension: &str) -> Option<PathBuf> {
    let mut pathbuf_clone = pathbuf.to_owned();
    if !pathbuf_clone.set_extension(extension) {
        return None;
    };
    Some(PathBuf::from_iter(
        [prefix.as_os_str(), pathbuf_clone.file_name()?].iter(),
    ))
}
