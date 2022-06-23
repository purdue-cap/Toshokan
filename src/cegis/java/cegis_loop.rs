use handlebars::Handlebars;
use std::cell::RefCell;
use std::path::PathBuf;
use super::{CEGISState, CEGISConfig, CEGISRecorder, super::FuncLog};
use crate::frontend::template_helpers::register_helpers;
use crate::frontend::{Encoder, Renderer, CEEncoder};
use crate::frontend::java::{JSketchRunner, JBMCRunner, JavacRunner};
use crate::backend::{java::JBMCLogAnalyzer, TraceError};
use tempfile::{tempdir, TempDir};
use tempfile::Builder as TempFileBuilder;
use std::io::Write;
use std::fs;

// TODO: Use logging utilities to replace println in java procedures
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

pub struct CEGISLoop<'r> {
    hb: RefCell<Handlebars<'r>>,
    state: CEGISState,
    config: CEGISConfig,
    work_dir: Option<PathBuf>,
    recorder: Option<CEGISRecorder>,
}

impl<'r> CEGISLoop<'r> {
    pub fn new(config: CEGISConfig) -> Self {
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);
        Self {
            hb: RefCell::new(hb),
            state: CEGISState::new(config.get_params()),
            config: config,
            work_dir: None,
            recorder: None,
        }
    }

    pub fn get_work_dir(&self) -> Option<&PathBuf> {self.work_dir.as_ref()}
    pub fn get_recorder(&self) -> Option<&CEGISRecorder> {self.recorder.as_ref()}

    fn synthesize(&self, c_e: &CEEncoder, runner: &mut JSketchRunner)
        -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        let synthesis_dir = self.work_dir.as_ref().ok_or("Work dir unset")?.join(
            format!("synthesis_{}", self.state.get_iter_count()));
        fs::create_dir(&synthesis_dir)?;
        let output_dir = synthesis_dir.join("results");
        fs::create_dir(&output_dir)?;
        let synthesis_jsk = synthesis_dir.join("Synthesis.java");
        c_e.render_to_file(
            self.state.get_params().ok_or("State param not present")?,
            &synthesis_jsk
        )?;
        let jsk_files = vec![synthesis_jsk];
        let _jsk_output = runner.run(
            self.config.get_params().output_classes.iter(),
            jsk_files.iter(), &output_dir)?;
        let output_files: Result<Vec<PathBuf>, _> =
            fs::read_dir(&output_dir)?
                .map(|entry_result| 
                    entry_result.map(|entry| entry.path())
                ).collect();
        Ok(output_files?)
    }

    // Returning Ok(None) means verification passed
    // Otherwise returns (C.E.s, Traces)
    fn verify<'a>(&self, compiler: &mut JavacRunner,
        runner: &mut JBMCRunner, analyzer: &'a mut JBMCLogAnalyzer)
        -> Result<Option<(&'a Vec<Vec<i32>>, &'a Vec<Vec<FuncLog>>)>, Box<dyn std::error::Error>> {
        let verification_dir = self.work_dir.as_ref().ok_or("Work dir unset")?.join(
            format!("verification_{}", self.state.get_iter_count()));
        fs::create_dir(&verification_dir)?;
        let _compiler_output = compiler.run(
            self.state.current_cand.iter(), &verification_dir)?;
        let logs_result = runner.run(
            &self.config.get_params().verif_entrance, &verification_dir);
        
        let logs = match logs_result {
            Ok(logs) => logs,
            Err(TraceError::JSONError(Some(Ok(json_buf)), json_err)) => {
                let mut failed_file = fs::File::create(&verification_dir.join("failed_log.json"))?;
                failed_file.write(json_buf.as_bytes())?;
                return Err(Box::new(TraceError::JSONError(Some(Ok(json_buf)), json_err)))
            },
            Err(other_error) => return Err(Box::new(other_error))
        };
        let log_file = fs::File::create(&verification_dir.join("jbmc_log.json"))?;
        serde_json::to_writer_pretty(log_file, &logs)?;
        let verif_passed = analyzer.analyze_logs(&logs)?;
        if verif_passed {
            Ok(None)
        } else {
            if !analyzer.get_unwind_err_loops().is_empty() {
                runner.grow_unwind(analyzer.get_unwind_err_loops())?;
                println!("Unwind growth:{:?}", runner.get_current_unwind());
            }
            Ok(Some((analyzer.get_c_e_s(), analyzer.get_traces())))
        }
    }

    pub fn run_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir_obj = tempdir()?;

        let mut temp_dir_saver = TempDirSaver::new();
        self.work_dir = Some(temp_dir_obj.path().to_path_buf());
        let class_dir = temp_dir_obj.path().join("classes");
        fs::create_dir(&class_dir)?;
        // Output directory for JSketch
        let jsketch_out_dir = temp_dir_obj.path().join("jsketch_out");
        fs::create_dir(&jsketch_out_dir)?;
        if self.config.get_params().keep_tmp {
            temp_dir_saver.set_temp_dir_obj(temp_dir_obj);
        }

        // Initialize CEEncoder and JSketchRunner
        // TODO: wire up CEEncoder to RewriterController
        let mut c_e_encoder = CEEncoder::new(&self.hb);
        c_e_encoder.load(&self.config.get_params().c_e_encoder_src)?;
        let mut jsketch_runner = JSketchRunner::new(
            &self.config.get_params().jsketch_config);
        jsketch_runner.common_files.extend(
            self.config.get_params().synth_files.iter().cloned()
        );
        jsketch_runner.out_dir = Some(jsketch_out_dir);

        // Initialize JavaRunner, JBMCRunner and JBMCLogAnalyzer
        let mut javac_runner = JavacRunner::new(
            &self.config.get_params().javac_bin);
        javac_runner.extra_class_path.extend(
            self.config.get_params().verif_classpaths.iter().cloned()
        );
        javac_runner.extra_class_path.push(class_dir.as_os_str().into());
        // Compile reference libraries
        let _pre_compile_output = javac_runner.run(
            self.config.get_params().verif_src_files.iter(),
            &class_dir
        )?;
        let mut jbmc_runner = JBMCRunner::new(
            &self.config.get_params().jbmc_config);
        jbmc_runner.extra_class_path.extend(
            self.config.get_params().verif_classpaths.iter().cloned()
        );
        jbmc_runner.extra_class_path.push(class_dir.as_os_str().into());
        let mut log_analyzer = JBMCLogAnalyzer::new(
            self.config.get_params().lib_funcs.iter()
        );

        self.recorder = if self.config.get_params().enable_record {
            Some(CEGISRecorder::new(Some(
                TempFileBuilder::new().prefix("emphermal_record.").suffix(".json").tempfile()?
            )))
        } else {
            None
        };

        if let Some(ref recorder) = self.recorder {
            println!("Emphermal Record:{:?}", recorder.get_ephemeral_record_path());
        }

        // Initial counter examples
        let mut init_c_e : Vec<Vec<i32>> = vec![vec![0; self.config.get_params().n_inputs]];
        if let Some(ref mut recorder) = self.recorder {
            recorder.set_new_c_e_s(&init_c_e);
            recorder.start_total_clock();
        }
        self.state.c_e_set.insert(init_c_e.pop().expect("Index checked"));
        self.state.update_params()
            .ok_or("Param update failure")?;

        loop {
            // Synthesis
            if let Some(ref mut recorder) = self.recorder {
                recorder.set_iter_nth(self.state.get_iter_count());
                recorder.start_synthesis();
            }
            let synth_result = self.synthesize(
                &c_e_encoder, 
                &mut jsketch_runner)?;
            self.state.current_cand = synth_result;

            if let Some(ref mut recorder) = self.recorder {
                recorder.stop_synthesis();
                recorder.start_verification();
            }

            let verif_result = self.verify(
                &mut javac_runner,
                &mut jbmc_runner,
                &mut log_analyzer
            )?;
            if let Some(ref mut recorder) = self.recorder {
                recorder.stop_verification();
            }
            if let Some((c_e_s, traces)) = verif_result {
                if let Some(ref mut recorder) = self.recorder {
                    recorder.set_new_c_e_s(c_e_s);
                    recorder.set_new_traces(traces);
                }
                self.state.c_e_set.extend(c_e_s.iter().cloned());
                self.state.logs.extend(traces.iter().cloned());
                self.state.update_params()
                    .ok_or("Param update failure")?;
            } else {
                break;
            }
            if let Some(ref mut recorder) = self.recorder {
                recorder.step_iteration();
                recorder.commit();
                recorder.write_ephemeral_record()?;
            }
            self.state.incr_iteration();
        }
        for path in self.state.current_cand.iter() {
            let target = self.config.get_params().output_dir
                .join(path.file_name().ok_or("No file name in cand path")?);
            fs::copy(path, &target)?;
        }
        if let Some(ref mut recorder) = self.recorder {
            recorder.set_total_iter(self.state.get_iter_count());
            recorder.set_solved(true);
            recorder.commit_time();
        }
        Ok(())
    }
}