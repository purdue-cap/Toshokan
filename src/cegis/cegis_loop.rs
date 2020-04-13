use crate::frontend::{Encoder, CandEncoder, CEEncoder, GenerationEncoder};
use crate::frontend::{SketchRunner, VerificationResult, SynthesisResult, GenerationResult};
use crate::frontend::template_helpers::register_helpers;
use crate::backend::{LogAnalyzer, HoleExtractor, LibraryTracer};
use super::CEGISConfig;
use super::CEGISState;
use handlebars::Handlebars;
use tempfile::{tempdir, TempDir};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::fs;

pub struct CEGISLoop<'r> {
    hb: RefCell<Handlebars<'r>>,
    config: CEGISConfig,
    state: CEGISState,
    work_dir: Option<TempDir>,
    output_dir: Option<PathBuf>
}

impl<'r> CEGISLoop<'r> {
    pub fn new(config: CEGISConfig) -> Self{
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);
        let state = CEGISState::new(config.n_f_args,
            config.n_inputs,
            config.init_n_unknowns,
            config.n_holes,
            config.pure_function);
        CEGISLoop {
            hb: RefCell::new(hb),
            config: config,
            state: state,
            work_dir: None,
            output_dir: None
        }
    }

    fn verify(&self, cand: &CandEncoder, 
        log_analyzer: &LogAnalyzer, runner: &mut SketchRunner)
            -> Result<Option<Vec<isize>>, Box<dyn std::error::Error>> {
        let verification_sk = cand.render(&self.state)?;
        let output = runner.verify_str(verification_sk.as_str());
        match output {
            VerificationResult::Pass => {Ok(None)},
            VerificationResult::ExecutionErr(err) => {Err(Box::new(err))},
            VerificationResult::CounterExample(exec_log) => {
                Ok(Some(log_analyzer.read_c_e_s_from_str(exec_log.as_str())?))
            }
        }
    }

    fn synthesize(&self, c_e: &CEEncoder,
        hole_extractor: &HoleExtractor, runner: &mut SketchRunner)
            -> Result<Option<Vec<isize>>, Box<dyn std::error::Error>> {
        let synthesis_sk = c_e.render(&self.state)?;
        let output = runner.synthesize_str(synthesis_sk.as_str());
        match output {
            SynthesisResult::Failure => {Ok(None)},
            SynthesisResult::ExecutionErr(err) => {Err(Box::new(err))},
            SynthesisResult::Candidate => {
                Ok(Some(hole_extractor.read_holes_from_file(self.output_dir.as_ref()
                    .ok_or("Output dir unset")?.join("holes.xml"))?))
            }
        }
    }

    fn generate(&self, generation: &GenerationEncoder, runner: &mut SketchRunner) 
            -> Result<PathBuf, Box<dyn std::error::Error>> {
       let generation_sk = generation.render(&self.state)?;
       let output = runner.generate_str(generation_sk.as_str());
       match output {
           GenerationResult::Err(err) => {Err(Box::new(err))},
           GenerationResult::Ok(base_name) => {Ok(base_name)}
       }

    }

    fn trace(&self, base_name: &Path, library_tracer: &LibraryTracer)
            -> Result<Vec<(Vec<isize>, isize)>, Box<dyn std::error::Error>> {
        let base_name_str = base_name.to_str().ok_or("Base name conversion to str failed")?;
        let main_src = PathBuf::from(format!("{}.cpp", base_name_str));
        let other_src = [PathBuf::from(format!("{}_test.cpp", base_name_str))];
        library_tracer.build_tracer_src(&main_src).ok_or("Build tracer source failed")?;
        library_tracer.build_tracer_bin(&other_src).ok_or("Build tracer binary failed")?;
        Ok(library_tracer.collect_traces().ok_or("Trace collection failed")?)
    }

    pub fn clear_output(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::remove_dir_all(self.output_dir.as_ref().ok_or("Output dir unset")?)?;
        fs::create_dir(self.output_dir.as_ref().ok_or("Output dir unset")?.as_path())?;
        Ok(())
    }

    pub fn run_loop(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        self.work_dir = Some(tempdir()?);
        self.output_dir = Some(self.work_dir.as_ref().ok_or("Work dir unset")?.path().join("output"));
        fs::create_dir(self.output_dir.as_ref().ok_or("Output dir unset")?)?;
        let mut sketch_runner = SketchRunner::new(self.config.sketch_bin.as_path(),
            self.output_dir.as_ref().ok_or("Output dir unset")?);

        let mut cand_encoder = CandEncoder::new(&self.hb);
        let mut c_e_encoder = CEEncoder::new(&self.hb);
        let mut generation_encoder = GenerationEncoder::new(&self.hb);
        cand_encoder.load(&self.config.cand_encoder_src)?;
        c_e_encoder.load(&self.config.c_e_encoder_src)?;
        generation_encoder.load(&self.config.generation_encoder_src)?;

        let c_e_names_in_log : Vec<_> = self.config.c_e_names.iter().map(|s| s.as_str()).collect();
        let log_analyzer = LogAnalyzer::new(c_e_names_in_log.as_slice());

        let hole_extractor = HoleExtractor::new(self.config.n_holes, self.config.hole_offset);

        let mut library_tracer = LibraryTracer::new(self.config.impl_file.as_path(),
            self.config.lib_func_name.as_str(),
            self.config.sketch_home.as_path());
        library_tracer.set_work_dir(self.output_dir.as_ref().ok_or("Output dir unset")?);

        let solved = loop {
            if let Some(new_c_e) = self.verify(&cand_encoder,
                &log_analyzer, &mut sketch_runner)? {
                self.state.add_c_e(new_c_e);
            } else {
                // Verification passed
                break true;
            }
            if let Some(new_holes) = self.synthesize(&c_e_encoder,
                &hole_extractor, &mut sketch_runner)? {
                self.state.update_holes(new_holes.as_slice());
            } else {
                break false;
            }
            let base_name = self.generate(&generation_encoder, &mut sketch_runner)?;
            for (args, rtn) in self.trace(base_name.as_path(), &library_tracer)?.into_iter() {
                self.state.add_log(args, rtn);
            }
        };
        Ok(solved)
    }
}