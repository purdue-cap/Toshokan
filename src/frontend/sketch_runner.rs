use std::process::{Command, Output};
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use log::{debug, trace};

pub struct SketchRunner {
    frontend_path: OsString,
    backend_path: OsString,
    frontend_cmd: Command,
    backend_cmd: Command,
    fe_flags: Vec<OsString>,
    be_flags: Vec<OsString>,
    output_dir: PathBuf
}

pub enum VerificationResult {
    CounterExample(String),
    Pass,
    ExecutionErr(io::Error)
}

pub enum SynthesisResult {
    Candidate,
    Failure,
    ExecutionErr(io::Error)
}

pub type GenerationResult = io::Result<(PathBuf, String)>; // Generated base file name

impl SketchRunner{
    pub fn new<P: AsRef<Path>>(frontend_path: P, backend_path:P, output_dir: P) -> Self{
        SketchRunner{
            frontend_path: frontend_path.as_ref().as_os_str().to_os_string(),
            backend_path: backend_path.as_ref().as_os_str().to_os_string(),
            frontend_cmd: Command::new(frontend_path.as_ref().as_os_str()),
            backend_cmd: Command::new(backend_path.as_ref().as_os_str()),
            fe_flags: Vec::<OsString>::new(),
            be_flags: Vec::<OsString>::new(),
            output_dir: output_dir.as_ref().to_path_buf()
        }
    }
    pub fn fe_clear(&mut self) -> &mut Self {
        self.fe_flags.clear();
        self
    }

    pub fn fe_flag<S: AsRef<OsStr>>(&mut self, opt: S) -> &mut Self{
        self.fe_flags.push(opt.as_ref().to_os_string());
        self
    }

    pub fn be_clear(&mut self) -> &mut Self {
        self.be_flags.clear();
        self
    }

    pub fn be_flag<S: AsRef<OsStr>>(&mut self, opt: S) -> &mut Self{
        self.be_flags.push(opt.as_ref().to_os_string());
        self
    }

    pub fn fe_output<P: AsRef<Path>>(&mut self, input_file:P) -> io::Result<Output> {
        self.frontend_cmd.args(&self.fe_flags);
        self.frontend_cmd.arg(input_file.as_ref());
        debug!(target: "SketchRunner", "Sketch Frontend command: {:?}", self.frontend_cmd);
        let result = self.frontend_cmd.output();
        trace!(target: "SketchRunner", "Sketch Frontend result.status: {:?}", result.as_ref().ok().map(|r| r.status));
        trace!(target: "SketchRunner", "Sketch Frontend result.stdout: {}",
            result.as_ref().ok().map(|r| &r.stdout)
            .and_then(|b| String::from_utf8(b.clone()).ok())
            .unwrap_or("<Failure>".to_string())
        );
        trace!(target: "SketchRunner", "Sketch Frontend result.stderr: {}",
            result.as_ref().ok().map(|r| &r.stderr)
            .and_then(|b| String::from_utf8(b.clone()).ok())
            .unwrap_or("<Failure>".to_string())
        );
        self.frontend_cmd = Command::new(self.frontend_path.clone());
        result
    }

    pub fn be_output<P: AsRef<Path>>(&mut self, input_file:P) -> io::Result<Output> {
        self.backend_cmd.args(&self.be_flags);
        self.backend_cmd.arg(input_file.as_ref());
        debug!(target: "SketchRunner", "Sketch Backend command: {:?}", self.backend_cmd);
        let result = self.backend_cmd.output();
        trace!(target: "SketchRunner", "Sketch Backend result.status: {:?}", result.as_ref().ok().map(|r| r.status));
        trace!(target: "SketchRunner", "Sketch Backend result.stdout: {}",
            result.as_ref().ok().map(|r| &r.stdout)
            .and_then(|b| String::from_utf8(b.clone()).ok())
            .unwrap_or("<Failure>".to_string())
        );
        trace!(target: "SketchRunner", "Sketch Backend result.stderr: {}",
            result.as_ref().ok().map(|r| &r.stderr)
            .and_then(|b| String::from_utf8(b.clone()).ok())
            .unwrap_or("<Failure>".to_string())
        );
        self.backend_cmd = Command::new(self.backend_path.clone());
        result
    }

    pub fn set_be_verify_flags(&mut self, flags: &Vec<OsString>) -> &mut Self {
        self.be_clear();
        for flag in flags {
            self.be_flag(&flag);
        }
        self
    }

    pub fn fe_flag_synthesize(&mut self) -> &mut Self {
        let output_dir = self.output_dir.clone();
        self.fe_clear().fe_flag("--fe-output-test")
            .fe_flag("--fe-output-dir").fe_flag(output_dir.join("./"))
            .fe_flag("--fe-output-xml").fe_flag(output_dir.join("holes.xml"))
    }

    pub fn fe_flag_generate(&mut self) -> &mut Self {
        let output_dir = self.output_dir.clone();
        self.fe_clear().fe_flag("--fe-keep-tmp")
            .fe_flag("--fe-cegis-path").fe_flag("_sketch_dummy")
            .fe_flag("--fe-tempdir").fe_flag(output_dir.join("./"))
            .fe_flag("--debug-cex").fe_flag("-V").fe_flag("3")
    }

    pub fn verify_file<P: AsRef<Path>>(&mut self, input_file:P) -> VerificationResult {
        match self.be_output(input_file) {
            Ok(output) => {
                if let Some(code) = output.status.code() {
                    if code == 0 {
                        VerificationResult::Pass
                    } else {
                        match String::from_utf8(output.stdout) {
                            Ok(decoded) => VerificationResult::CounterExample(decoded),
                            Err(error) =>
                                VerificationResult::ExecutionErr(io::Error::new(io::ErrorKind::Other, error))
                        }
                    }
                } else {
                    VerificationResult::ExecutionErr(io::Error::new(io::ErrorKind::Interrupted, "Terminated by signal"))
                }
            },
            Err(error) => VerificationResult::ExecutionErr(error)
        }
    }

    pub fn synthesize_file<P: AsRef<Path>>(&mut self, input_file:P) -> SynthesisResult {
        self.fe_flag_synthesize();
        match self.fe_output(input_file) {
            Ok(output) => {
                if let Some(code) = output.status.code() {
                    if code == 0 {
                        SynthesisResult::Candidate
                    } else {
                        SynthesisResult::Failure
                    }
                } else {
                    SynthesisResult::ExecutionErr(io::Error::new(io::ErrorKind::Interrupted, "Terminated by signal"))
                }
            },
            Err(error) => SynthesisResult::ExecutionErr(error)
        }
    }

    pub fn generate_file<P: AsRef<Path>>(&mut self, input_file:P) -> GenerationResult {
        self.fe_flag_generate();
        self.fe_output(input_file.as_ref()).and_then(|output| {
            let base_name = input_file.as_ref().file_name()
                .ok_or(io::Error::new(io::ErrorKind::InvalidInput, "Input file has no base file name"))?;
            let tmp_path = self.output_dir.join("tmp").join(base_name).join("input0.tmp");
            match String::from_utf8(output.stdout) {
                Ok(decoded) => Ok((tmp_path, decoded)),
                Err(error) => Err(io::Error::new(io::ErrorKind::Other, error))
            }
        })
    }

    pub fn verify_str<S: AsRef<str>>(&mut self, input: S) -> VerificationResult {
        match NamedTempFile::new() {
            Ok(mut temp_file) => {
                match temp_file.write(input.as_ref().as_bytes()) {
                    Ok(bytes_written) => {
                        if bytes_written == input.as_ref().len() {
                            self.verify_file(temp_file.path())
                        } else {
                            VerificationResult::ExecutionErr(io::Error::new(io::ErrorKind::UnexpectedEof, "Content truncated"))
                        }
                    },
                    Err(error) => VerificationResult::ExecutionErr(error)
                }
            }, 
            Err(error) => VerificationResult::ExecutionErr(error)
        }
    }

    pub fn synthesize_str<S: AsRef<str>>(&mut self, input: S) -> SynthesisResult {
        match NamedTempFile::new() {
            Ok(mut temp_file) => {
                match temp_file.write(input.as_ref().as_bytes()) {
                    Ok(bytes_written) => {
                        if bytes_written == input.as_ref().len() {
                            self.synthesize_file(temp_file.path())
                        } else {
                            SynthesisResult::ExecutionErr(io::Error::new(io::ErrorKind::UnexpectedEof, "Content truncated"))
                        }
                    },
                    Err(error) => SynthesisResult::ExecutionErr(error)
                }
            }, 
            Err(error) => SynthesisResult::ExecutionErr(error)
        }
    }

    pub fn generate_str<S: AsRef<str>>(&mut self, input: S) -> GenerationResult {
        NamedTempFile::new().and_then(|mut temp_file|{
            temp_file.write(input.as_ref().as_bytes()).and_then(|bytes_written| {
                if bytes_written == input.as_ref().len() {
                    self.generate_file(temp_file.path())
                } else {
                    Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Content truncated"))
                }
            })
        })
    }
}

// TODO: Unit Tests for SketchRunner