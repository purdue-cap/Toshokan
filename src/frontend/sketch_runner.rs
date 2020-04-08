use std::process::{Command, Output};
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub struct SketchRunner {
    path: OsString,
    sketch: Command,
    flags: Vec<OsString>,
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

pub type GenerationResult = io::Result<()>;

impl SketchRunner{
    pub fn new<P: AsRef<Path>>(path: P, output_dir: P) -> Self{
        SketchRunner{
            path: path.as_ref().as_os_str().to_os_string(),
            sketch: Command::new(path.as_ref().as_os_str()),
            flags: Vec::<OsString>::new(),
            output_dir: output_dir.as_ref().to_path_buf()
        }
    }
    pub fn clear(&mut self) -> &mut Self {
        self.flags.clear();
        self
    }

    pub fn flag<S: AsRef<OsStr>>(&mut self, opt: S) -> &mut Self{
        self.flags.push(opt.as_ref().to_os_string());
        self
    }

    pub fn output<P: AsRef<Path>>(&mut self, input_file:P) -> io::Result<Output> {
        self.sketch.args(&self.flags);
        self.sketch.arg(input_file.as_ref());
        let result = self.sketch.output();
        self.sketch = Command::new(self.path.clone());
        result
    }

    pub fn flag_verify(&mut self) -> &mut Self {
        self.clear().flag("--debug-cex").flag("-V").flag("3")
    }

    pub fn flag_synthesize(&mut self) -> &mut Self {
        let output_dir = self.output_dir.clone();
        self.clear().flag("--fe-output-test")
            .flag("--fe-output-dir").flag(output_dir.join("./"))
            .flag("--fe-output-xml").flag(output_dir.join("holes.xml"))
            .flag("--fe-output-hole-func").flag(output_dir.join("holes.txt"))
    }

    pub fn flag_generate(&mut self) -> &mut Self {
        let output_dir = self.output_dir.clone();
        self.clear().flag("--fe-output-test")
            .flag("--fe-output-dir").flag(output_dir.join("./"))
            .flag("--fe-force-codegen")
    }

    pub fn verify_file<P: AsRef<Path>>(&mut self, input_file:P) -> VerificationResult {
        self.flag_verify();
        match self.output(input_file) {
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
        self.flag_synthesize();
        match self.output(input_file) {
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
        self.flag_generate();
        self.output(input_file).and(Ok(()))
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