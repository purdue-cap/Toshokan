use std::ffi::{OsString, OsStr};
use std::path::Path;
use std::io;
use std::process::{Command, Output};

pub struct JavacRunner {
    javac_path: OsString,
    class_dir: OsString,
    extra_class_path: Vec<OsString>,
    extra_flags: Vec<OsString>
}

impl JavacRunner {
    pub fn new<P1, P2>(javac_path: P1, class_dir: P2) -> Self
        where P1: AsRef<Path>, P2: AsRef<Path>{
        Self {
            javac_path: javac_path.as_ref().as_os_str().to_os_string(),
            class_dir: class_dir.as_ref().as_os_str().to_os_string(),
            extra_class_path: vec![],
            extra_flags: vec![]
        }
    }

    fn build_flags(&self) -> Vec<OsString> {
        // Default flags
        let mut flags = vec![
            OsString::from("-d"),
            self.class_dir.clone()];

        flags.push(OsString::from("-classpath"));
        // Build classpath
        let mut cp_joined = self.class_dir.clone();
        for cp in self.extra_class_path.iter() {
            cp_joined.push(":");
            cp_joined.push(cp);
        }
        flags.push(cp_joined);
        flags.extend(self.extra_flags.iter().cloned());
        flags
    }

    pub fn add_class_path<P: AsRef<Path>>(&mut self, new_path: P) {
        self.extra_class_path.push(new_path.as_ref().as_os_str().to_os_string());
    }

    pub fn add_flag<S: AsRef<OsStr>>(&mut self, new_flag: S) {
        self.extra_flags.push(new_flag.as_ref().to_os_string());
    }
    
    pub fn run<S, I>(&self, files: I) -> io::Result<Output>
        where I: Iterator<Item=S>, S: AsRef<OsStr> {
        let mut args = self.build_flags();
        args.extend(files.map(|s| s.as_ref().to_os_string()));

        let mut cmd = Command::new(self.javac_path.as_os_str());
        cmd.args(args);

        let result = cmd.output();
        result
    }
}