use std::{
    thread::sleep as tsleep,
    time::Duration,
    path::{PathBuf, self},
};

pub fn sleep(seconds: f64) {
    tsleep(Duration::from_secs_f64(seconds));
}

pub fn pathbuf_to_string(pathbuf: &PathBuf, desired_length: usize, error_str: &str) -> String {
    let filestem: &str = pathbuf.file_stem().unwrap_or(error_str.as_ref()).to_str().unwrap_or(error_str);
    if filestem.len() > desired_length {
        String::from_iter([&filestem[0..desired_length-2], ".."])
    } else {
        String::from(filestem)
    }
}