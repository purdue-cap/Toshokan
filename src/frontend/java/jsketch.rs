use std::ffi::{OsString, OsStr};
use std::path::{Path, PathBuf};
use std::io;
use std::process::{Command, Output};
use std::fs;
use derive_builder::Builder;

pub struct JSketchRunner<'c> {
    jsketch_config: &'c JSketchConfig,
    pub extra_flags: Vec<OsString>,
    pub common_files: Vec<PathBuf>
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct JSketchConfig {
    pub dir_path: PathBuf,
    #[builder(default)]
    pub inline: Option<usize>,
    #[builder(default)]
    pub unroll: Option<usize>,
    #[builder(default)]
    pub inbits: Option<usize>,
    #[builder(default)]
    pub cbits: Option<usize>,
    #[builder(default)]
    pub array_bound: Option<usize>,
    #[builder(setter(each = "sk_opt"), default)]
    pub sk_opts: Vec<String>,
}

test_fixture!(JSketchConfig, dummy, builder{dir_path("")});

impl<'c> JSketchRunner<'c> {
    pub fn new(jsketch_config: &'c JSketchConfig) -> Self {
        Self {
            jsketch_config: jsketch_config,
            extra_flags: vec![],
            common_files: vec![]
        }
    }

    fn build_flags(&self) -> Vec<OsString> {
        // Default flags
        let mut flags = vec![OsString::from("--java_codegen")];

        if let Some(bound) = self.jsketch_config.inline {
            flags.push(format!("--inline={}", bound).into());
        }

        if let Some(bound) = self.jsketch_config.unroll {
            flags.push(format!("--unroll={}", bound).into());
        }

        if let Some(bound) = self.jsketch_config.inbits {
            flags.push(format!("--inbits={}", bound).into());
        }

        if let Some(bound) = self.jsketch_config.cbits {
            flags.push(format!("--cbits={}", bound).into());
        }

        if let Some(bound) = self.jsketch_config.array_bound {
            flags.push("-sk_opts=--bnd-arr-size".into());
            flags.push(format!("--sk_opts={}", bound).into());
            flags.push("-sk_opts=--bnd-arr1d-size".into());
            flags.push(format!("--sk_opts={}", bound).into());
        }

        for opt in self.jsketch_config.sk_opts.iter() {
            flags.push(format!("--sk_opts={}", opt).into());
        }

        flags.extend(self.extra_flags.iter().cloned());
        flags
    }
    
    // Generated Java files will be written to <save_dir>/<class_name>.java
    pub fn run<SIter, PIter, S, P>(&self, classes: SIter, files: PIter, save_dir: P)
        -> io::Result<Output>
        where SIter: Iterator<Item=S>, S: AsRef<str>,
            PIter: Iterator<Item=P>, P: AsRef<Path> {
        let mut args = self.build_flags();
        // Add common files
        args.extend(self.common_files.iter().map(|p| p.clone().into_os_string()));
        // Other files
        args.extend(files.map(|p| p.as_ref().as_os_str().to_os_string()));

        let script: PathBuf = [self.jsketch_config.dir_path.as_os_str(), OsStr::new("jsk.sh")].iter().collect();
        let mut script_cmd = Command::new(script.as_os_str());
        script_cmd.args(args);
        script_cmd.current_dir(&self.jsketch_config.dir_path);

        let result = script_cmd.output()?;
        if !result.status.success() {
            Err(io::Error::new(io::ErrorKind::Other, 
                format!("JSketch failure: {}", std::str::from_utf8(&result.stderr).unwrap_or("<decode failure>"))))
        } else {
            for class in classes {
                let mut src_file: PathBuf = [self.jsketch_config.dir_path.as_os_str(),
                    OsStr::new("result"), OsStr::new("java"),
                    OsStr::new(class.as_ref())].iter().collect();
                src_file.set_extension("java");
                let mut dst_file: PathBuf = save_dir.as_ref().to_path_buf();
                dst_file.push(Path::new(class.as_ref()));
                dst_file.set_extension("java");
                fs::copy(&src_file, &dst_file)?;
            }
            Ok(result)
        }
    }
}