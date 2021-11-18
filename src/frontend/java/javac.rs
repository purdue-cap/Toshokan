use std::ffi::{OsString, OsStr};
use std::path::Path;
use std::io;
use std::process::{Command, Output};

pub struct JavacRunner {
    javac_path: OsString,
    pub extra_class_path: Vec<OsString>,
    pub extra_flags: Vec<OsString>
}

impl JavacRunner {
    pub fn new<P>(javac_path: P) -> Self
        where P: AsRef<Path> {
        Self {
            javac_path: javac_path.as_ref().as_os_str().to_os_string(),
            extra_class_path: vec![],
            extra_flags: vec![]
        }
    }

    fn build_flags<S>(&self, class_dir: S) -> Vec<OsString>
        where S: AsRef<OsStr> {
        // Default flags
        let mut flags = vec![
            OsString::from("-d"),
            class_dir.as_ref().to_os_string()];

        flags.push(OsString::from("-classpath"));
        // Build classpath
        // input classpath must be put before the extra ones
        // So that in case of class override input takes priority
        let mut cp_joined = class_dir.as_ref().to_os_string();
        for cp in self.extra_class_path.iter() {
            cp_joined.push(":");
            cp_joined.push(cp);
        }
        flags.push(cp_joined);
        flags.extend(self.extra_flags.iter().cloned());
        flags
    }

    pub fn run<S, I, Sp>(&self, files: I, class_dir: Sp) -> io::Result<Output>
        where I: Iterator<Item=S>,
            S: AsRef<OsStr>, Sp: AsRef<OsStr> {
        let mut args = self.build_flags(class_dir);
        args.extend(files.map(|s| s.as_ref().to_os_string()));

        let mut cmd = Command::new(self.javac_path.as_os_str());
        cmd.args(args);

        let result = cmd.output()?;
        if !result.status.success() {
            Err(io::Error::new(io::ErrorKind::Other, 
                format!("Compilation failure: {}", std::str::from_utf8(&result.stderr).unwrap_or("<decode failure>"))))
        } else {
            Ok(result)
        }
    }
}