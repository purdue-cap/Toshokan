use crate::frontend::{Encoder, CandEncoder, CEEncoder, GenerationEncoder};
use crate::frontend::{SketchRunner, VerificationResult, SynthesisResult};
use crate::frontend::RewriteController;
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
use std::collections::HashMap;
use log::{trace, debug, info, warn};

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
            -> Result<Option<(HashMap<String, isize>, PathBuf)>, Box<dyn std::error::Error>> {
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
            SynthesisResult::Candidate(base_path) => {
                let holes_file = self.output_dir.as_ref()
                    .ok_or("Output dir unset")?.join("holes.xml");
                
                trace!(target: "Synthesis", "Holes file: {}", fs::read(&holes_file).ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string()));

                info!(target: "Synthesis", "Generated Code Base Path: {}", base_path.to_str().unwrap_or("<Failure>"));
                trace!(target: "Synthesis", "Main file: {}", 
                base_path.to_str()
                .map(|s| format!("{}.cpp", s))
                .and_then(|s| fs::read(&s).ok())
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string())
                );
                trace!(target: "Synthesis", "Harness file: {}", 
                base_path.to_str()
                .map(|s| format!("{}_test.cpp", s))
                .and_then(|s| fs::read(&s).ok())
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string())
                );

                Ok(Some((hole_extractor.read_holes_from_file(holes_file)?, base_path)))
            }
        }
    }

    fn generate(&self, generation: &GenerationEncoder, runner: &mut SketchRunner) 
            -> Result<PathBuf, Box<dyn std::error::Error>> {
        info!(target: "Generation", "Filling sketch template");
        let generation_sk = self.work_dir.as_ref().ok_or("Work dir unset")?.path().join(
            PathBuf::from("generation"));
        generation.render_to_file(&self.state, &generation_sk)?;
        trace!(target: "Generation", "Sketch template {}:\n{}",
            generation_sk.to_str().unwrap_or("<Failure>"),
            fs::read(&generation_sk).ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or("<Failure>".to_string()));
        info!(target: "Generation", "Running sketch");
        let output = runner.generate_file_and_setup_be(&generation_sk);
        match output {
            Err(err) => {Err(Box::new(err))},
            Ok(input_tmp) => {
                info!(target: "Generation", "Generated input.tmp path: {}", input_tmp.to_str().unwrap_or("<Failure>"));
                debug!(target: "Generation", "Extracted backend flags: {:?}", runner.get_be_flags());
                trace!(target: "Generation", "input.tmp content: {}", 
                input_tmp.to_str()
                .and_then(|s| fs::read(&s).ok())
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string())
                );

                Ok(input_tmp)
            }
       }

    }

    fn trace(&self, base_path: &Path, library_tracer: &mut LibraryTracer)
            -> Result<Vec<(Vec<isize>, isize)>, Box<dyn std::error::Error>> {
        let base_name = base_path.file_name().ok_or("Get base name from base path failed")?
            .to_str().ok_or("Base name conversion to str failed")?;
        let main_src = PathBuf::from(format!("{}.cpp", base_path.to_str().ok_or("Base path conversion to str failed")?));

        info!(target: "Trace", "Current base name: {}", base_name);
        library_tracer.set_base_name(&base_name);

        info!(target: "Trace", "Building tracer source");
        let tracer_src = library_tracer.build_tracer_src(&main_src).ok_or("Build tracer source failed")?;

        info!(target: "Trace", "Building tracer entry source");
        let entry_src = library_tracer.build_entry_src(&self.get_state().get_params().c_e_s)
            .ok_or("Build entry source failed")?;
        trace!(target: "Trace", "Entry source: {}", 
            fs::read(entry_src.as_path()).ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string())
        );

        let other_src = [entry_src];
        info!(target: "Trace", "Building tracer source successful");

        trace!(target: "Trace", "Tracer source: {}",
            fs::read(tracer_src.as_path()).ok()
                .and_then(|bytes| String::from_utf8(bytes).ok())
                .unwrap_or("<Failure>".to_string())
        );

        info!(target: "Trace", "Building tracer binary");
        library_tracer.build_tracer_bin(&other_src).ok_or("Build tracer binary failed")?;
        info!(target: "Trace", "Building tracer binary successful");
        info!(target: "Trace", "Running tracer");
        Ok(library_tracer.collect_traces().ok_or("Trace collection failed")?)
    }

    pub fn get_state(&self) -> &CEGISState {&self.state}

    pub fn get_state_mut(&mut self) -> &mut CEGISState {&mut self.state}

    pub fn get_recorder(&self) -> Option<&CEGISRecorder> {self.recorder.as_ref()}

    pub fn run_loop(&mut self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        info!(target: "CEGISMainLoop", "Start initialization");

        self.work_dir = Some(tempdir()?);
        info!(target: "CEGISMainLoop", "Working directory: {:?}", self.work_dir);
        self.output_dir = Some(self.work_dir.as_ref().ok_or("Work dir unset")?.path().join("output"));
        fs::create_dir(self.output_dir.as_ref().ok_or("Output dir unset")?)?;
        let mut sketch_runner = SketchRunner::new(
            self.config.get_params().sketch_fe_bin.as_path(),
            self.config.get_params().sketch_be_bin.as_path(),
            self.output_dir.as_ref().ok_or("Output dir unset")?,
            self.work_dir.as_ref().ok_or("Work dir unset")?.path()
        );
        let mut rewrite_controller = RewriteController::new(&self.config);

        if self.config.is_be_config_unresolved() {
            info!(target: "CEGISMainLoop", "Running generation to resolve backend config");
            let mut generation_encoder = GenerationEncoder::new(&self.hb);
            generation_encoder.setup_rewrite(&rewrite_controller)?;
            generation_encoder.load(&self.config.get_params().generation_encoder_src)?;
            let generated_input_tmp = self.generate(&generation_encoder, &mut sketch_runner)?;
            self.config.set_input_tmp_path(generated_input_tmp);
        }

        self.config.populate_be_config(&mut sketch_runner);

        rewrite_controller.update_with_config(&self.config);
        
        self.config.populate_v_p_s(&mut self.state);
        debug!(target: "CEGISMainLoop", "Verify points: {:?}", self.state.get_params().verify_points);
        warn!(target: "CEGISMainLoop", "Verify points are currently unused in sketch backend based verification phrase.");

        let mut cand_encoder = CandEncoder::new(&self.hb);
        let mut c_e_encoder = CEEncoder::new(&self.hb);
        cand_encoder.setup_rewrite(&rewrite_controller)?;
        c_e_encoder.setup_rewrite(&rewrite_controller)?;
        cand_encoder.load(&self.config.get_params().cand_encoder_src)?;
        c_e_encoder.load(&self.config.get_params().c_e_encoder_src)?;

        let h_names = cand_encoder.get_hole_names().ok_or("Failed to get hole names from input.tmp")?;
        debug!(target:"CEGISMainLoop", "Hole names: {:?}", h_names);
        self.state.set_h_names(h_names);

        if self.config.get_params().enable_record {
            self.recorder = Some(CEGISRecorder::new());
        } else {
            self.recorder = None;
        }

        self.recorder.as_mut().map(|r| r.reset_clock());

        let c_e_names_in_log : Vec<_> = self.config.get_params().c_e_names.iter().map(|s| s.as_str()).collect();
        let log_analyzer = LogAnalyzer::new(c_e_names_in_log.as_slice());

        let hole_extractor = HoleExtractor::new(self.config.get_params().hole_offset);

        let mut library_tracer = LibraryTracer::new(self.config.get_params().impl_file.as_path(),
            self.config.get_params().lib_func_name.as_str(),
            self.config.get_params().harness_func_name.as_str(),
            self.config.get_params().sketch_home.as_path());
        library_tracer.set_work_dir(self.output_dir.as_ref().ok_or("Output dir unset")?);
        info!(target: "CEGISMainLoop", "Initialization complete");

        let mut base_path : Result<PathBuf, &str> = Err("Base name uninitialized");
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
                // Read code for final output
                info!(target: "CEGISMainLoop", "Reading synthesized cpp code as final output");
                let result = fs::read_to_string(format!("{}.cpp", base_path?.to_str().ok_or("Base name conversion to str failed")?))?;
                break Some(result);
            }

            if self.state.get_iter_count() == 0 {
                info!(target: "CEGISMainLoop", "Iter 0: Pre-run synthesis before tracing to generate runnable candidate");
                info!(target: "CEGISMainLoop", "Synthesizing(Pre-run)");
                if let Some((new_holes, new_base_path)) = self.synthesize(&c_e_encoder,
                    &hole_extractor, &mut sketch_runner)? {
                    info!(target: "CEGISMainLoop", "Synthesis(Pre-run) returned candidate");
                    debug!(target: "CEGISMainLoop", "Updated Holes: {:?}", new_holes);
                    self.recorder.as_mut().map(|r| r.set_holes(&new_holes));
                    self.state.update_holes(new_holes);
                    base_path = Ok(new_base_path);
                } else {
                    info!(target: "CEGISMainLoop", "Synthesis(Pre-run) failed");
                    break None;
                }
            }

            info!(target: "CEGISMainLoop", "Tracing");
            let traces = self.trace(base_path?.as_path(), &mut library_tracer)?;
            info!(target: "CEGISMainLoop", "Tracing successful");
            debug!(target: "CEGISMainLoop", "New Traces: {:?}", traces);
            self.recorder.as_mut().map(|r| r.set_new_traces(&traces));
            for (args, rtn) in traces.into_iter() {
                self.state.add_log(args, rtn);
            }

            info!(target: "CEGISMainLoop", "Synthesizing");
            if let Some((new_holes, new_base_path)) = self.synthesize(&c_e_encoder,
                &hole_extractor, &mut sketch_runner)? {
                info!(target: "CEGISMainLoop", "Synthesis returned candidate");
                debug!(target: "CEGISMainLoop", "Updated Holes: {:?}", new_holes);
                self.recorder.as_mut().map(|r| r.set_holes(&new_holes));
                self.state.update_holes(new_holes);
                base_path = Ok(new_base_path);
            } else {
                info!(target: "CEGISMainLoop", "Synthesis failed");
                break None;
            }

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
        self.recorder.as_ref().map(|r| info!(target:"CEGISMainLoop", "Total elapsed time: {}", r.get_time()));
        info!(target:"CEGISMainLoop", "Total iterations run: {}", self.state.get_iter_count() + 1);
        Ok(solved)
    }
}