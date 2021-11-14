use crate::frontend::{Encoder, Renderer, CandEncoder, CEEncoder, GenerationEncoder};
use crate::frontend::{SketchRunner, VerificationResult, SynthesisResult};
use crate::frontend::RewriteController;
use crate::frontend::template_helpers::register_helpers;
use crate::backend::{LogAnalyzer, HoleExtractor, LibraryTracer};
use super::CEGISConfig;
use super::CEGISState;
use super::CEGISRecorder;
use super::TraceLog;
use handlebars::Handlebars;
use tempfile::{tempdir, Builder, TempDir};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use log::{trace, debug, info, warn};
use regex::Regex;

pub struct CEGISLoop<'r> {
    hb: RefCell<Handlebars<'r>>,
    config: CEGISConfig,
    state: CEGISState,
    recorder: Option<CEGISRecorder>,
    work_dir: Option<PathBuf>,
    output_dir: Option<PathBuf>
}

fn validate_filled_template(path: &Path) -> Option<()> {
    let unfilled_var = Regex::new(r"\{\{.*\}\}").expect("Hard coded regex should not fail");
    let content = fs::read_to_string(path).ok()?;
    match unfilled_var.find(content.as_str()) {
        Some(_) => None,
        None => Some(())
    }
}

struct TempDirSaver{
    temp_dir_obj: Option<TempDir>,
}

impl Drop for TempDirSaver {
    fn drop(&mut self) {
        if let Some(taken_temp_dir_obj) = self.temp_dir_obj.take() {
            taken_temp_dir_obj.into_path();
        }
    }
}

impl TempDirSaver {
    pub fn new() -> Self {
        Self {
            temp_dir_obj: None
        }
    }

    pub fn set_temp_dir_obj(&mut self, temp_dir_obj: TempDir) {
        self.temp_dir_obj = Some(temp_dir_obj);
    }
}

impl<'r> CEGISLoop<'r> {
    pub fn new(config: CEGISConfig) -> Self{
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);
        CEGISLoop {
            hb: RefCell::new(hb),
            state: CEGISState::new(config.get_params()),
            config: config,
            recorder: None,
            work_dir: None,
            output_dir: None
        }
    }

    fn verify(&self, cand: &CandEncoder, 
        log_analyzer: &LogAnalyzer, runner: &mut SketchRunner)
            -> Result<(Option<Vec<isize>>, PathBuf), Box<dyn std::error::Error>> {
        info!(target: "Verification", "Filling sketch template");
        let verification_sk = self.work_dir.as_ref().ok_or("Work dir unset")?.join(
            PathBuf::from(format!("verification_{}", self.state.get_iter_count())));
        cand.render_to_file(
            self.state.get_params().ok_or("State param not present")?,
            &verification_sk)?;
        trace!(target: "Verification", "Sketch template {}:\n{}",
            verification_sk.to_str().unwrap_or("<Failure>"),
            fs::read(&verification_sk).ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or("<Failure>".to_string()));
        validate_filled_template(verification_sk.as_path()).ok_or("Verification Sketch not completed filled")?;
        info!(target: "Verification", "Running sketch");
        let output = runner.verify_file(&verification_sk);
        match output {
            VerificationResult::Pass => {Ok((None, verification_sk))},
            VerificationResult::ExecutionErr(err) => {Err(Box::new(err))},
            VerificationResult::CounterExample(exec_log) => {
                trace!(target:"Verification", "Sketch execution log: {}", exec_log);
                Ok((Some(log_analyzer.read_c_e_s_from_str(exec_log.as_str())?), verification_sk))
            }
        }
    }

    fn synthesize(&self, c_e: &CEEncoder,
        hole_extractor: &mut HoleExtractor, runner: &mut SketchRunner)
            -> Result<(
                    Option<(HashMap<String, isize>, PathBuf/* Generated CPP code base path */)>,
                    PathBuf/* Last synthesis sketch path*/),
                Box<dyn std::error::Error>> {
        info!(target: "Synthesis", "Filling sketch template");
        let synthesis_sk = self.work_dir.as_ref().ok_or("Work dir unset")?.join(
            PathBuf::from(format!("synthesis_{}", self.state.get_iter_count())));
        c_e.render_to_file(
            self.state.get_params().ok_or("State param not present")?,
            &synthesis_sk)?;
        trace!(target: "Synthesis", "Sketch template {}:\n{}",
            synthesis_sk.to_str().unwrap_or("<Failure>"),
            fs::read(&synthesis_sk).ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or("<Failure>".to_string()));
        validate_filled_template(synthesis_sk.as_path()).ok_or("Synthesis Sketch not completed filled")?;
        info!(target: "Synthesis", "Running sketch");
        let output = runner.synthesize_file(&synthesis_sk);
        match output {
            SynthesisResult::Failure => {Ok((None, synthesis_sk))},
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

                Ok(((Some((hole_extractor.read_holes_from_file(holes_file)?, base_path))), synthesis_sk))
            }
        }
    }

    fn generate(&self, generation: &GenerationEncoder, runner: &mut SketchRunner) 
            -> Result<PathBuf/* Last verification input.tmp path */, Box<dyn std::error::Error>> {
        info!(target: "Generation", "Filling sketch template");
        let generation_sk = self.work_dir.as_ref().ok_or("Work dir unset")?.join(
            PathBuf::from("generation"));
        generation.render_to_file(
            self.state.get_params().ok_or("State param not present")?,
            &generation_sk)?;
        trace!(target: "Generation", "Sketch template {}:\n{}",
            generation_sk.to_str().unwrap_or("<Failure>"),
            fs::read(&generation_sk).ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or("<Failure>".to_string()));
        validate_filled_template(generation_sk.as_path()).ok_or("Generation Sketch not completed filled")?;
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
            -> Result<(Vec<TraceLog>, bool/* Trace timeout indicator */), Box<dyn std::error::Error>> {
        let base_name = base_path.file_name().ok_or("Get base name from base path failed")?
            .to_str().ok_or("Base name conversion to str failed")?;
        let main_src = PathBuf::from(format!("{}.cpp", base_path.to_str().ok_or("Base path conversion to str failed")?));

        info!(target: "Trace", "Current base name: {}", base_name);
        library_tracer.set_base_name(&base_name);

        info!(target: "Trace", "Setting up current compiler flags");
        library_tracer.setup_compiler_flags(self.get_state());

        info!(target: "Trace", "Building tracer source");
        let tracer_src = library_tracer.build_tracer_src(&main_src).ok_or("Build tracer source failed")?;

        info!(target: "Trace", "Building tracer entry source");
        let entry_src = library_tracer.build_entry_src(&self.get_state().get_params().ok_or("State params not available")?.c_e_s)
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
        Ok(library_tracer.collect_traces()?)
    }

    pub fn get_state(&self) -> &CEGISState {&self.state}

    pub fn get_state_mut(&mut self) -> &mut CEGISState {&mut self.state}

    pub fn get_recorder(&self) -> Option<&CEGISRecorder> {self.recorder.as_ref()}

    pub fn run_loop(&mut self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        self.recorder.as_mut().map(|r| r.start_total_clock());

        info!(target: "CEGISMainLoop", "Start initialization");

        let temp_dir_obj = tempdir()?;
        let mut temp_dir_saver = TempDirSaver::new();
        self.work_dir = Some(temp_dir_obj.path().to_path_buf());
        if self.config.get_params().keep_tmp {
            temp_dir_saver.set_temp_dir_obj(temp_dir_obj);
        }
        info!(target: "CEGISMainLoop", "Working directory: {:?}", self.work_dir);
        self.output_dir = Some(self.work_dir.as_ref().ok_or("Work dir unset")?.join("output"));
        fs::create_dir(self.output_dir.as_ref().ok_or("Output dir unset")?)?;
        let mut sketch_runner = SketchRunner::new(
            self.config.get_params().sketch_fe_bin.as_path(),
            self.config.get_params().sketch_be_bin.as_path(),
            self.output_dir.as_ref().ok_or("Output dir unset")?,
            self.work_dir.as_ref().ok_or("Work dir unset")?
        );
        let mut rewrite_controller = RewriteController::new(&self.config);

        if self.config.is_be_config_unresolved() {
            info!(target: "CEGISMainLoop", "Running generation to resolve backend config");
            let mut generation_encoder = GenerationEncoder::new(&self.hb);
            generation_encoder.setup_rewrite(&rewrite_controller)?;
            generation_encoder.load(&self.config.get_params().generation_encoder_src)?;
            self.state.update_params().ok_or("State param update failed")?;
            let generated_input_tmp = self.generate(&generation_encoder, &mut sketch_runner)?;
            self.config.set_input_tmp_path(generated_input_tmp);
        }

        self.config.populate_be_config(&mut sketch_runner);

        rewrite_controller.update_with_config(&self.config);
        
        self.config.populate_v_p_s(&mut self.state);
        debug!(target: "CEGISMainLoop", "Verify points: {:?}", self.state.get_v_p_set());
        warn!(target: "CEGISMainLoop", "Verify points are currently unused in sketch backend based verification phrase.");

        let mut cand_encoder = CandEncoder::new();
        let mut c_e_encoder = CEEncoder::new(&self.hb);
        cand_encoder.setup_rewrite(&rewrite_controller)?;
        c_e_encoder.setup_rewrite(&rewrite_controller)?;
        cand_encoder.load(&self.config.get_params().cand_encoder_src)?;
        c_e_encoder.load(&self.config.get_params().c_e_encoder_src)?;

        let h_names = cand_encoder.get_hole_names().ok_or("Failed to get hole names from input.tmp")?;
        debug!(target:"CEGISMainLoop", "Hole names: {:?}", h_names);
        self.state.set_h_names(h_names);

        if self.config.get_params().enable_record {
            self.recorder = Some(CEGISRecorder::new(Some(
                Builder::new().prefix("emphermal_record.").suffix(".json").tempfile()?
            )));
        } else {
            self.recorder = None;
        }

        let c_e_names_in_log : Vec<_> = self.config.get_params().c_e_names.iter().map(|s| s.as_str()).collect();
        let log_analyzer = LogAnalyzer::new(c_e_names_in_log.as_slice());

        let mut hole_extractor = HoleExtractor::new(
            self.config.get_params().excluded_holes.iter().cloned(),
            self.state.get_h_names().clone());

        let mut library_tracer = LibraryTracer::new(self.config.get_params().impl_file.as_path(),
            self.config.get_params().get_trace_config_strings(),
            self.config.get_params().harness_func_name.as_str(),
            self.config.get_params().sketch_home.as_ref().map(|p| p.as_path()), self.config.get_params().trace_timeout,
            self.config.get_params().empty_harness_call);
        library_tracer.set_work_dir(self.output_dir.as_ref().ok_or("Output dir unset")?)
            .ok_or("Prepare work dir for library tracer failed")?;
        let mut retry_strategy = self.config.new_retry_strategy();
        let mut base_path : Result<PathBuf, &str> = Err("Base name uninitialized");

        info!(target: "CEGISMainLoop", "Initialization complete");
        self.recorder.as_mut().map(|r| r.step_initialization());
        let solved = loop {
            info!(target: "CEGISMainLoop", "Entering iteration #{}", self.state.get_iter_count());

            self.recorder.as_mut().map(|r| r.start_verification());
            info!(target: "CEGISMainLoop", "Verifying");
            self.state.update_params().ok_or("State param update failed")?;
            let (verify_result, last_verification) = self.verify(&cand_encoder,
                &log_analyzer, &mut sketch_runner)?;
            self.recorder.as_mut().map(|r| r.set_last_verification(&last_verification));
            self.recorder.as_mut().map(|r| r.set_verification_seed(sketch_runner.get_last_verification_seed_used()));
            self.recorder.as_mut().map(|r| r.stop_verification());
            if let Some(new_c_e) = verify_result{
                info!(target: "CEGISMainLoop", "Verification returned C.E.");
                debug!(target: "CEGISMainLoop", "New C.E: {:?}", new_c_e);
                self.recorder.as_mut().map(|r| r.set_new_c_e_s(&new_c_e));
                self.state.add_c_e(new_c_e);
            } else {
                // Verification passed
                info!(target: "CEGISMainLoop", "Verification successful, returning solution");
                debug!(target: "CEGISMainLoop", "Final holes: {:?}", self.state.get_holes());
                // Read code for final output
                info!(target: "CEGISMainLoop", "Reading synthesized cpp code as final output");
                let current_iter_nth = self.state.get_iter_count();
                self.recorder.as_mut().map(|r| {
                    r.set_iter_nth(current_iter_nth);
                    r.commit();
                });
                let result = fs::read_to_string(format!("{}.cpp", base_path?.to_str().ok_or("Base name conversion to str failed")?))?;
                break Some(result);
            }

            if self.state.get_iter_count() == 0 {
                info!(target: "CEGISMainLoop", "Iter 0: Pre-run synthesis before tracing to generate runnable candidate");
                info!(target: "CEGISMainLoop", "Synthesizing(Pre-run)");
                let synthesize_result = loop {
                    self.recorder.as_mut().map(|r| r.start_synthesis());
                    self.state.update_params().ok_or("State param update failed")?;
                    let (result, last_synthesis) = self.synthesize(&c_e_encoder,
                        &mut hole_extractor, &mut sketch_runner)?;
                    self.recorder.as_mut().map(|r| r.set_last_synthesis(&last_synthesis));
                    self.recorder.as_mut().map(|r| r.add_pre_synthesis_seed(sketch_runner.get_last_synthesis_seed_used()));
                    if result.is_some() {
                        retry_strategy.succeed(&self.state);
                        break result;
                    } else if !retry_strategy.fail_and_retry(&mut self.state){
                        break None;
                    } else {
                        warn!(target: "CEGISMainLoop", "Synthesis failed, retry initiated by RetryStrategy");
                    }
                };
                self.recorder.as_mut().map(|r| r.stop_pre_synthesis());
                if let Some((new_holes, new_base_path)) = synthesize_result {
                    info!(target: "CEGISMainLoop", "Synthesis(Pre-run) returned candidate");
                    debug!(target: "CEGISMainLoop", "Updated Holes: {:?}", new_holes);
                    self.recorder.as_mut().map(|r| r.set_holes(&new_holes));
                    self.state.update_holes(new_holes);
                    base_path = Ok(new_base_path);
                } else {
                    info!(target: "CEGISMainLoop", "Synthesis(Pre-run) failed");
                    let current_iter_nth = self.state.get_iter_count();
                    self.recorder.as_mut().map(|r| {
                        r.set_iter_nth(current_iter_nth);
                        r.commit();
                    });
                    break None;
                }
            }

            self.recorder.as_mut().map(|r| r.start_trace());
            info!(target: "CEGISMainLoop", "Tracing");
            self.state.update_params().ok_or("State param update failed")?;
            let (traces, timed_out) = self.trace(base_path?.as_path(), &mut library_tracer)?;
            info!(target: "CEGISMainLoop", "Tracing successful");
            debug!(target: "CEGISMainLoop", "New Traces: {:?}", traces);
            self.recorder.as_mut().map(|r| r.set_new_traces(&traces));
            self.recorder.as_mut().map(|r| r.set_trace_timed_out(timed_out));
            self.recorder.as_mut().map(|r| r.stop_trace());
            for trace in traces.into_iter() {
                self.state.add_log(trace);
            }

            info!(target: "CEGISMainLoop", "Synthesizing");
            let synthesize_result = loop {
                self.recorder.as_mut().map(|r| r.start_synthesis());
                self.state.update_params().ok_or("State param update failed")?;
                let (result, last_synthesis) = self.synthesize(&c_e_encoder,
                    &mut hole_extractor, &mut sketch_runner)?;
                self.recorder.as_mut().map(|r| r.set_last_synthesis(&last_synthesis));
                self.recorder.as_mut().map(|r| r.add_synthesis_seed(sketch_runner.get_last_synthesis_seed_used()));
                if result.is_some() {
                    retry_strategy.succeed(&self.state);
                    break result;
                } else if !retry_strategy.fail_and_retry(&mut self.state){
                    break None;
                } else {
                    warn!(target: "CEGISMainLoop", "Synthesis failed, retry initiated by RetryStrategy");
                }
            };
            self.recorder.as_mut().map(|r| r.stop_synthesis());
            if let Some((new_holes, new_base_path)) = synthesize_result {
                info!(target: "CEGISMainLoop", "Synthesis returned candidate");
                debug!(target: "CEGISMainLoop", "Updated Holes: {:?}", new_holes);
                self.recorder.as_mut().map(|r| r.set_holes(&new_holes));
                self.state.update_holes(new_holes);
                base_path = Ok(new_base_path);
            } else {
                info!(target: "CEGISMainLoop", "Synthesis failed");
                let current_iter_nth = self.state.get_iter_count();
                self.recorder.as_mut().map(|r| {
                    r.set_iter_nth(current_iter_nth);
                    r.commit();
                });
                break None;
            }

            debug!(target: "CEGISMainLoop", "Current C.E.s: {:?}", self.state.get_c_e_set());
            debug!(target: "CEGISMainLoop", "Current Holes: {:?}", self.state.get_holes());
            debug!(target: "CEGISMainLoop", "Current trace count: {}", self.state.get_n_logs());
            trace!(target: "CEGISMainLoop", "Current Traces: {:?}", self.state.get_logs());
            info!(target: "CEGISMainLoop", "Exiting iteration #{}", self.state.get_iter_count());
            let current_iter_nth = self.state.get_iter_count();
            self.recorder.as_mut().map(|r| {
                r.set_iter_nth(current_iter_nth);
                r.step_iteration();
                r.commit();
                r.write_ephemeral_record().ok().map(
                    |_| info!(target:"CEGISMainLoop", "Ephemeral record logged to {}",
                        r.get_ephemeral_record_path().map(|p| p.to_str()).flatten().unwrap_or("<Failure>")));
            });
            self.state.incr_iteration();
        };
        let final_iter_count = self.state.get_iter_count();
        self.recorder.as_mut().map(|r| r.set_solved(solved.is_some()));
        self.recorder.as_mut().map(|r| r.set_total_iter(final_iter_count));
        self.recorder.as_mut().map(|r| r.commit_time());
        self.recorder.as_ref().map(|r| info!(target:"CEGISMainLoop", "Total elapsed time: {}", r.get_time()));
        if self.config.get_params().keep_tmp {
            self.recorder.as_mut().map(|r| r.commit_last_files());
        }
        info!(target:"CEGISMainLoop", "Total iterations run: {}", self.state.get_iter_count() + 1);
        Ok(solved)
    }
}