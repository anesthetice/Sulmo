use std::{
    thread::sleep as tsleep,
    time::Duration,
};

pub fn sleep(seconds: f64) {
    tsleep(Duration::from_secs_f64(seconds));
}