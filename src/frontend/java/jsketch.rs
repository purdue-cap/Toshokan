use std::ffi::OsString;
use std::path::Path;

pub struct JSketchRunner {
    jsketch_dir: OsString
}

impl JSketchRunner {
    pub fn new<P>(jsketch_dir: P) -> Self
        where P: AsRef<Path> {
        Self {
            jsketch_dir: jsketch_dir.as_ref().as_os_str().to_os_string()
        }
    }
}