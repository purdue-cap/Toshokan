use std::ffi::{OsString, OsStr};
use std::path::Path;
use std::process::Command;
use crate::backend::{TraceError ,java::JBMCLogs};

pub struct JBMCRunner {
   jbmc_path: OsString,
   class_dir: OsString,
   extra_class_path: Vec<OsString>,
   extra_flags: Vec<OsString>
}

impl JBMCRunner {
    pub fn new<P1, P2>(jbmc_path: P1, class_dir: P2) -> Self
        where P1: AsRef<Path>, P2: AsRef<Path>{
        Self {
            jbmc_path: jbmc_path.as_ref().as_os_str().to_os_string(),
            class_dir: class_dir.as_ref().as_os_str().to_os_string(),
            extra_class_path: vec![],
            extra_flags: vec![]
        }
    }

    fn build_flags(&self) -> Vec<OsString> {
        // Default flags
        let mut flags = vec![OsString::from("--json-ui"),
            OsString::from("--trace-json-extended")];

        flags.push(OsString::from("--classpath"));
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
    
    pub fn run<S: AsRef<str>>(&self, entrance: S) -> Result<JBMCLogs, TraceError>{
        let mut args = self.build_flags();
        args.push(OsString::from(entrance.as_ref()));

        let mut cmd = Command::new(self.jbmc_path.as_os_str());
        cmd.args(args);

        let result = cmd.output()?;
        Ok(serde_json::from_slice(&result.stdout)?)
    }
}

// TODO: Unit tests