use std::process::{Command, Output};
use std::ffi::{OsStr, OsString};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use regex::Regex;
use tempfile::NamedTempFile;
use log::{debug, trace, warn};

pub struct SketchRunner {
    frontend_path: OsString,
    backend_path: OsString,
    frontend_cmd: Command,
    backend_cmd: Command,
    fe_flags: Vec<OsString>,
    be_flags: Vec<OsString>,
    output_dir: PathBuf,
    generation_dir: PathBuf,
    last_synthesis_seed_used: Option<u64>,
    last_verification_seed_used: Option<u64>,
    synthesis_seed_to_use: Option<u64>,
    verification_seed_to_use: Option<u64>

}

pub enum VerificationResult {
    CounterExample(String),
    Pass,
    ExecutionErr(io::Error)
}

pub enum SynthesisResult {
    Candidate(PathBuf), // Generated base file name
    Failure,
    ExecutionErr(io::Error)
}

pub type GenerationResult = io::Result<(PathBuf, String)>; // Generated input tmp file path 

impl SketchRunner{
    pub fn new<P: AsRef<Path>>(frontend_path: P, backend_path:P, output_dir: P, generation_dir: P) -> Self{
        SketchRunner{
            frontend_path: frontend_path.as_ref().as_os_str().to_os_string(),
            backend_path: backend_path.as_ref().as_os_str().to_os_string(),
            frontend_cmd: Command::new(frontend_path.as_ref().as_os_str()),
            backend_cmd: Command::new(backend_path.as_ref().as_os_str()),
            fe_flags: Vec::<OsString>::new(),
            be_flags: Vec::<OsString>::new(),
            output_dir: output_dir.as_ref().to_path_buf(),
            generation_dir: generation_dir.as_ref().to_path_buf(),
            last_synthesis_seed_used: None,
            last_verification_seed_used: None,
            synthesis_seed_to_use: None,
            verification_seed_to_use: None
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
    
    pub fn get_fe_flags(&self) -> &Vec<OsString> {&self.fe_flags}

    pub fn get_be_flags(&self) -> &Vec<OsString> {&self.be_flags}

    pub fn get_last_synthesis_seed_used(&self) -> Option<u64> {self.last_synthesis_seed_used}

    pub fn get_last_verification_seed_used(&self) -> Option<u64> {self.last_verification_seed_used}

    pub fn set_synthesis_seed_to_use(&mut self, seed: Option<u64>) {self.synthesis_seed_to_use = seed;}

    pub fn set_verification_seed_to_use(&mut self, seed: Option<u64>) {self.verification_seed_to_use = seed;}

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
        if let Some(seed) = self.verification_seed_to_use {
            self.backend_cmd.arg("--seed").arg(seed.to_string());
        }
        self.backend_cmd.arg("-o").arg(self.generation_dir.join("be_solution"));
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

    pub fn extract_be_verify_flags<S: AsRef<str>>(&self, gen_output: S) -> Option<Vec<OsString>> {
        let regex = Regex::new(r"\[SATBackend\] Launching: _sketch_dummy (.*)")
            .expect("Hard coded regex should not fail.");
        let dummy_args = regex.captures(gen_output.as_ref())?.get(1)?.as_str()
            .trim().split_whitespace().collect::<Vec<_>>();
        let flags = dummy_args.iter().rev().skip(3).rev().map(|s| OsString::from(s)).collect::<Vec<_>>();
        Some(flags)
    }

    pub fn fe_flag_synthesize(&mut self) -> &mut Self {
        let output_dir = self.output_dir.clone();
        self.fe_clear().fe_flag("--fe-output-test").fe_flag("--fe-low-overhead")
            .fe_flag("--fe-output-dir").fe_flag(output_dir.join("./"))
            .fe_flag("--fe-tempdir").fe_flag(output_dir.join("./synthesis_tmp/"))
            .fe_flag("--fe-output-xml").fe_flag(output_dir.join("holes.xml"))
            .fe_flag("-V").fe_flag("3");
        if let Some(seed) = self.synthesis_seed_to_use {
            self.fe_flag("--slv-seed").fe_flag(seed.to_string())
        } else {
            self
        }
    }

    pub fn fe_flag_generate(&mut self) -> &mut Self {
        let generation_dir = self.generation_dir.clone();
        self.fe_clear().fe_flag("--fe-keep-tmp").fe_flag("--fe-low-overhead")
            .fe_flag("--fe-cegis-path").fe_flag("_sketch_dummy")
            .fe_flag("--fe-tempdir").fe_flag(generation_dir.join("./"))
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
                            Ok(decoded) => {
                                let pattern = Regex::new(r"SOLVER RAND SEED = (\d+)").expect("Hard coded regex should not fail");
                                self.last_verification_seed_used = pattern.captures(decoded.as_str())
                                    .and_then(|caps| caps.get(1))
                                    .and_then(|seed_match| seed_match.as_str().parse::<u64>().ok());
                                VerificationResult::CounterExample(decoded)
                            },
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
        match self.fe_output(input_file.as_ref()) {
            Ok(output) => {
                if let Some(code) = output.status.code() {
                    if code == 0 {
                        if let Some(base_name) = input_file.as_ref().file_name() {
                            self.last_synthesis_seed_used = std::str::from_utf8(&output.stdout).ok()
                                .and_then(|stdout_str| {
                                    let pattern = Regex::new(r"SOLVER RAND SEED = (\d+)").expect("Hard coded regex should not fail");
                                    pattern.captures(stdout_str)
                                })
                                .and_then(|caps| caps.get(1))
                                .and_then(|seed_match| seed_match.as_str().parse::<u64>().ok());
                            SynthesisResult::Candidate(self.output_dir.join(base_name))
                        } else {
                            SynthesisResult::ExecutionErr(
                                io::Error::new(io::ErrorKind::InvalidInput, "Input file has no base file name"))
                        }
                        
                    } else {
                        warn!(target:"SketchRunner", "Sketch synthesis failure: {:?}", output);
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
            let tmp_path = self.generation_dir.join("tmp").join(base_name).join("input0.tmp");
            match String::from_utf8(output.stdout) {
                Ok(decoded) => Ok((tmp_path, decoded)),
                Err(error) => Err(io::Error::new(io::ErrorKind::Other, error))
            }
        })
    }

    pub fn generate_file_and_setup_be<P: AsRef<Path>>(&mut self, input_file: P) -> io::Result<PathBuf> {
        let result = self.generate_file(input_file)?;
        let flags = self.extract_be_verify_flags(&result.1)
            .ok_or(io::Error::new(io::ErrorKind::InvalidData, "No flags found in generation output"))?;
        self.set_be_verify_flags(&flags);
        Ok(result.0)
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

    pub fn generate_str_and_setup_be<S: AsRef<str>>(&mut self, input: S) -> io::Result<PathBuf> {
        let result = self.generate_str(input)?;
        let flags = self.extract_be_verify_flags(&result.1)
            .ok_or(io::Error::new(io::ErrorKind::InvalidData, "No flags found in generation output"))?;
        self.set_be_verify_flags(&flags);
        Ok(result.0)
    }
}

// TODO: Unit Tests for SketchRunner