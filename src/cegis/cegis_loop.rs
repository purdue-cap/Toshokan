use crate::frontend::{Encoder, CandEncoder, CEEncoder, GenerationEncoder};
use crate::frontend::{SketchRunner, VerificationResult, SynthesisResult, GenerationResult};
use crate::frontend::template_helpers::register_helpers;
use crate::backend::{LogAnalyzer, HoleExtractor, LibraryTracer};
use super::CEGISConfig;
use super::CEGISState;
use super::CEGISRecorder;
use handlebars::Handlebars;
use tempfile::{tempdir, TempDir};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::fs;
use log::{trace, debug, info};

pub struct CEGISLoop<'r> {
    hb: RefCell<Handlebars<'r>>,
    config: CEGISConfig,
    state: CEGISState,
    recorder: Option<CEGISRecorder>,
    work_dir: Option<TempDir>,
    output_dir: Option<PathBuf>
}

impl<'r> CEGISLoop<'r> {
    pub fn new(config: CEGISConfig) -> Self{
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);
        let state = CEGISState::new(config.get_params().n_f_args,
            config.get_params().n_inputs,
            config.get_params().init_n_unknowns,
            config.get_params().n_holes,
            config.get_params().pure_function);
        CEGISLoop {
            hb: RefCell::new(hb),
            config: config,
            state: state,
            recorder: None,
            work_dir: None,
            output_dir: None
        }
    }

    fn verify(&self, cand: &CandEncoder, 
        log_analyzer: &LogAnalyzer, runner: &mut SketchRunner)
            -> Result<Option<Vec<isize>>, Box<dyn std::error::Error>> {
        info!(target: "Verification", "Filling sketch template");
        let verification_sk = self.work_dir.as_ref().ok_or("Work dir unset")?.path().join(
            PathBuf::from(format!("verification_{}", self.state.get_iter_count())));
        cand.render_to_file(&self.state, &verification_sk)?;
        trace!(target: "Verification", "Sketch template {}:\n{}",
            verification_sk.to_str().unwrap_or("<Failure>"),
            fs::read(&verification_sk).ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or("<Failure>".to_string()));
        info!(target: "Verification", "Running sketch");
        let output = runner.verify_file(&verification_sk);
        match output {
            VerificationResult::Pass => {Ok(None)},
            VerificationResult::ExecutionErr(err) => {Err(Box::new(err))},
            VerificationResult::CounterExample(exec_log) => {
                trace!(target:"Verification", "Sketch execution log: {}", exec_log);
                Ok(Some(log_analyzer.read_c_e_s_from_str(exec_log.as_str())?))
            }
        }
    }

    fn synthesize(&self, c_e: &CEEncoder,
        hole_extractor: &HoleExtractor, runner: &mut SketchRunner)
            -> Result<Option<Vec<isize>>, Box<dyn std::error::Error>> {
        info!(target: "Synthesis", "Filling sketch template");
        let synthesis_sk = self.work_dir.as_ref().ok_or("Work dir unset")?.path().join(
            PathBuf::from(format!("synthesis_{}", self.state.get_iter_count())));
        c_e.render_to_file(&self.state, &synthesis_sk)?;
        trace!(target: "Synthesis", "Sketch template {}:\n{}",
            synthesis_sk.to_str().unwrap_or("<Failure>"),
            fs::read(&synthesis_sk).ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or("<Failure>".to_string()));
        info!(target: "Synthesis", "Running sketch");
        let output = runner.synthesize_file(&synthesis_sk);
        match output {
            SynthesisResult::Failure => {Ok(None)},
            SynthesisResult::ExecutionErr(err) => {Err(Box::new(err))},
            SynthesisResult::Candidate => {
                let holes_file = self.output_dir.as_ref()
                    .ok_or("Output dir unset")?.join("holes.xml");
                
                trace!(target: "Synthesis", "Holes file: {}", fs::read(&holes_file).ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string()));
                
                Ok(Some(hole_extractor.read_holes_from_file(holes_file)?))
            }
        }
    }

    fn generate(&self, generation: &GenerationEncoder, runner: &mut SketchRunner) 
            -> Result<PathBuf, Box<dyn std::error::Error>> {
        info!(target: "Generation", "Filling sketch template");
        let generation_sk = self.work_dir.as_ref().ok_or("Work dir unset")?.path().join(
            PathBuf::from(format!("generation_{}", self.state.get_iter_count())));
        generation.render_to_file(&self.state, &generation_sk)?;
        trace!(target: "Generation", "Sketch template {}:\n{}",
            generation_sk.to_str().unwrap_or("<Failure>"),
            fs::read(&generation_sk).ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or("<Failure>".to_string()));
        info!(target: "Generation", "Running sketch");
        let output = runner.generate_file(&generation_sk);
        match output {
            GenerationResult::Err(err) => {Err(Box::new(err))},
            GenerationResult::Ok(base_name) => {
                info!(target: "Generation", "Base Name: {}", base_name.to_str().unwrap_or("<Failure>"));
                trace!(target: "Generation", "Main file: {}", 
                base_name.to_str()
                .map(|s| format!("{}.cpp", s))
                .and_then(|s| fs::read(&s).ok())
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string())
                );
                trace!(target: "Generation", "Harness file: {}", 
                base_name.to_str()
                .map(|s| format!("{}_test.cpp", s))
                .and_then(|s| fs::read(&s).ok())
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string())
                );

                Ok(base_name)
            }
       }

    }

    fn trace(&self, base_name: &Path, library_tracer: &LibraryTracer)
            -> Result<Vec<(Vec<isize>, isize)>, Box<dyn std::error::Error>> {
        let base_name_str = base_name.to_str().ok_or("Base name conversion to str failed")?;
        let main_src = PathBuf::from(format!("{}.cpp", base_name_str));

        info!(target: "Trace", "Building tracer source");
        library_tracer.build_tracer_src(&main_src).ok_or("Build tracer source failed")?;

        info!(target: "Trace", "Building tracer entry source");
        let entry_src = library_tracer.build_entry_src(base_name_str, &self.get_state().get_params().c_e_s)
            .ok_or("Build entry source failed")?;
        trace!(target: "Trace", "Entry source: {}", 
            fs::read(entry_src.as_path()).ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string())
        );

        let other_src = [entry_src];
        info!(target: "Trace", "Building tracer source successful");

        trace!(target: "Trace", "Tracer source: {}",
            self.output_dir.as_ref().map(|dir| dir.join("tracer.cpp"))
                .and_then(|p| fs::read(&p).ok())
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string())
        );

        info!(target: "Trace", "Building tracer binary");
        library_tracer.build_tracer_bin(&other_src).ok_or("Build tracer binary failed")?;
        info!(target: "Trace", "Building tracer binary successful");
        info!(target: "Trace", "Running tracer");
        Ok(library_tracer.collect_traces().ok_or("Trace collection failed")?)
    }

    fn clear_output(&self) -> Result<(), Box<dyn std::error::Error>> {
        fs::remove_dir_all(self.output_dir.as_ref().ok_or("Output dir unset")?)?;
        fs::create_dir(self.output_dir.as_ref().ok_or("Output dir unset")?.as_path())?;
        Ok(())
    }

    pub fn get_state(&self) -> &CEGISState {&self.state}

    pub fn get_state_mut(&mut self) -> &mut CEGISState {&mut self.state}

    pub fn get_recorder(&self) -> Option<&CEGISRecorder> {self.recorder.as_ref()}

    pub fn run_loop(&mut self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        info!(target: "CEGISMainLoop", "Start initialization");

        self.config.populate_v_p_s(&mut self.state);
        debug!(target: "CEGISMainLoop", "Verify points: {:?}", self.state.get_params().verify_points);

        self.work_dir = Some(tempdir()?);
        self.output_dir = Some(self.work_dir.as_ref().ok_or("Work dir unset")?.path().join("output"));
        fs::create_dir(self.output_dir.as_ref().ok_or("Output dir unset")?)?;
        let mut sketch_runner = SketchRunner::new(self.config.get_params().sketch_bin.as_path(),
            self.output_dir.as_ref().ok_or("Output dir unset")?);

        let mut cand_encoder = CandEncoder::new(&self.hb);
        let mut c_e_encoder = CEEncoder::new(&self.hb);
        let mut generation_encoder = GenerationEncoder::new(&self.hb);
        cand_encoder.load(&self.config.get_params().cand_encoder_src)?;
        c_e_encoder.load(&self.config.get_params().c_e_encoder_src)?;
        generation_encoder.load(&self.config.get_params().generation_encoder_src)?;

        if self.config.get_params().enable_record {
            self.recorder = Some(CEGISRecorder::new());
        } else {
            self.recorder = None;
        }

        self.recorder.as_mut().map(|r| r.reset_clock());

        let c_e_names_in_log : Vec<_> = self.config.get_params().c_e_names.iter().map(|s| s.as_str()).collect();
        let log_analyzer = LogAnalyzer::new(c_e_names_in_log.as_slice());

        let hole_extractor = HoleExtractor::new(self.config.get_params().n_holes, self.config.get_params().hole_offset);

        let mut library_tracer = LibraryTracer::new(self.config.get_params().impl_file.as_path(),
            self.config.get_params().lib_func_name.as_str(),
            self.config.get_params().harness_func_name.as_str(),
            self.config.get_params().sketch_home.as_path());
        library_tracer.set_work_dir(self.output_dir.as_ref().ok_or("Output dir unset")?);
        info!(target: "CEGISMainLoop", "Initialization complete");

        let solved = loop {
            info!(target: "CEGISMainLoop", "Entering iteration #{}", self.state.get_iter_count());
            info!(target: "CEGISMainLoop", "Verifying");
            if let Some(new_c_e) = self.verify(&cand_encoder,
                &log_analyzer, &mut sketch_runner)? {
                info!(target: "CEGISMainLoop", "Verification returned C.E.");
                debug!(target: "CEGISMainLoop", "New C.E: {:?}", new_c_e);
                self.recorder.as_mut().map(|r| r.set_new_c_e_s(&new_c_e));
                self.state.add_c_e(new_c_e);
            } else {
                // Verification passed
                info!(target: "CEGISMainLoop", "Verification successful, returning solution");
                debug!(target: "CEGISMainLoop", "Final holes: {:?}", self.state.get_params().holes);
                let result = generation_encoder.render(&self.state)?;
                break Some(result);
            }
            info!(target: "CEGISMainLoop", "Generating");
            let base_name = self.generate(&generation_encoder, &mut sketch_runner)?;
            info!(target: "CEGISMainLoop", "Generation successful");
            info!(target: "CEGISMainLoop", "Tracing");
            let traces = self.trace(base_name.as_path(), &library_tracer)?;
            info!(target: "CEGISMainLoop", "Tracing successful");
            debug!(target: "CEGISMainLoop", "New Traces: {:?}", traces);
            self.recorder.as_mut().map(|r| r.set_new_traces(&traces));
            for (args, rtn) in traces.into_iter() {
                self.state.add_log(args, rtn);
            }
            info!(target: "CEGISMainLoop", "Synthesizing");
            if let Some(new_holes) = self.synthesize(&c_e_encoder,
                &hole_extractor, &mut sketch_runner)? {
                info!(target: "CEGISMainLoop", "Synthesis returned candidate");
                debug!(target: "CEGISMainLoop", "Updated Holes: {:?}", new_holes);
                self.recorder.as_mut().map(|r| r.set_holes(&new_holes));
                self.state.update_holes(new_holes.as_slice());
            } else {
                info!(target: "CEGISMainLoop", "Synthesis failed");
                break None;
            }
            self.clear_output()?;
            debug!(target: "CEGISMainLoop", "Current C.E.s: {:?}", self.state.get_params().c_e_s);
            debug!(target: "CEGISMainLoop", "Current Holes: {:?}", self.state.get_params().holes);
            debug!(target: "CEGISMainLoop", "Current trace count: {}", self.state.get_params().n_logs);
            debug!(target: "CEGISMainLoop", "Current Trace args: {:?}", self.state.get_params().logs_i);
            debug!(target: "CEGISMainLoop", "Current Trace rtns: {:?}", self.state.get_params().logs_r);
            info!(target: "CEGISMainLoop", "Exiting iteration #{}", self.state.get_iter_count());
            let current_iter_nth = self.state.get_iter_count();
            self.recorder.as_mut().map(|r| {
                r.set_iter_nth(current_iter_nth);
                r.commit();
            });
            self.state.incr_iteration();
        };
        self.recorder.as_mut().map(|r| r.set_solved(solved.is_some()));
        self.recorder.as_mut().map(|r| r.commit_time());
        Ok(solved)
    }
}