use std::ffi::{OsString, OsStr};
use std::path::{Path, PathBuf};
use std::io;
use std::process::{Command, Output};
use std::fs;

pub struct JSketchRunner {
    jsketch_dir: OsString,
    extra_flags: Vec<OsString>
}

impl JSketchRunner {
    pub fn new<P>(jsketch_dir: P) -> Self
        where P: AsRef<Path> {
        Self {
            jsketch_dir: jsketch_dir.as_ref().as_os_str().to_os_string(),
            extra_flags: vec![]
        }
    }

    pub fn add_flag<S>(&mut self, flag: S)
        where S: AsRef<OsStr> {
        self.extra_flags.push(flag.as_ref().to_os_string());
    }

    fn build_flags(&self) -> Vec<OsString> {
        // Default flags
        let mut flags = vec![OsString::from("--java_codegen")];

        flags.extend(self.extra_flags.iter().cloned());
        flags
    }

    // Generated Java files will be written to <save_dir>/<class_name>.java
    pub fn run<SIter, PIter, S, P>(&self, classes: SIter, files: PIter, save_dir: P)
        -> io::Result<Output>
        where SIter: Iterator<Item=S>, S: AsRef<str>,
            PIter: Iterator<Item=P>, P: AsRef<Path> {
        let mut args = self.build_flags();
        args.extend(files.map(|p| p.as_ref().as_os_str().to_os_string()));

        let script: PathBuf = [self.jsketch_dir.as_os_str(), OsStr::new("jsk.sh")].iter().collect();
        let mut script_cmd = Command::new(script.as_os_str());
        script_cmd.args(args);

        let result = script_cmd.output();
        for class in classes {
            let mut src_file: PathBuf = [self.jsketch_dir.as_os_str(),
                OsStr::new("result"), OsStr::new("java"),
                OsStr::new(class.as_ref())].iter().collect();
            src_file.set_extension("java");
            let mut dst_file: PathBuf = save_dir.as_ref().to_path_buf();
            dst_file.push(Path::new(class.as_ref()));
            dst_file.set_extension("java");
            fs::copy(&src_file, &dst_file)?;
        }
        result
    }
}