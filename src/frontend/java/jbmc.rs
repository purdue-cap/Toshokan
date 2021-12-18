use std::ffi::{OsString, OsStr};
use std::path::PathBuf;
use std::process::Command;
use crate::backend::{TraceError ,java::JBMCLogs};
use derive_builder::Builder;
use quick_error::ResultExt;

pub struct JBMCRunner<'c> {
    jbmc_config: &'c JBMCConfig,
    current_unwind: Option<usize>,
    pub extra_class_path: Vec<OsString>,
    pub extra_flags: Vec<OsString>,
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct JBMCConfig {
    pub bin_path: PathBuf,
    #[builder(default = "Some(32)")]
    pub unwind: Option<usize>,
    #[builder(default)]
    pub unwind_growth_step: Option<usize>,
    #[builder(default)]
    pub unwind_maximum: Option<usize>,
    #[builder(default)]
    pub depth: Option<usize>,
    #[builder(setter(each = "unwind_set"), default)]
    pub unwind_sets: Vec<String>,
    #[builder(default)]
    pub primitive_input_bound: Option<(usize, usize)>,
    #[builder(default)]
    pub nondet_array_len_bound: Option<usize>,
    #[builder(default)]
    pub nondet_tree_depth_bound: Option<usize>,
    #[builder(default)]
    pub nondet_string_len_bound: Option<usize>,
    #[builder(default = "false")]
    pub assume_input_non_null: bool,
    #[builder(default = "false")]
    pub assume_input_integral: bool,
    #[builder(setter(each = "string_input"), default)]
    pub string_inputs: Vec<String>,
}

test_fixture!(JBMCConfig, dummy, builder{bin_path("")});

impl<'c> JBMCRunner<'c> {
    pub fn new(jbmc_config: &'c JBMCConfig) -> Self {
        Self {
            jbmc_config: jbmc_config,
            current_unwind: jbmc_config.unwind.clone(),
            extra_class_path: vec![],
            extra_flags: vec![]
        }
    }

    pub fn get_current_unwind(&self) -> Option<usize> {self.current_unwind.clone()}

    pub fn grow_unwind(&mut self, err_loops: &Vec<String>) -> Result<(), TraceError> {
        // Upon enter this function, an unwind error should already be detected
        if let (Some(current), Some(step)) = (self.current_unwind, self.jbmc_config.unwind_growth_step) {
            let new_unwind = current + step;
            if let Some(bound) = self.jbmc_config.unwind_maximum {
                if new_unwind > bound {
                    // Exceeded bound, throw error
                    return Err(TraceError::JBMCUnwindError(err_loops.join(",")));
                }
            }
            // Within bound/no bound, set new unwind
            self.current_unwind = Some(new_unwind);
            Ok(())
        } else {
            // Either no step set, or no unwind set
            // Either case, an unwind error should be a hard error, throw it
            Err(TraceError::JBMCUnwindError(err_loops.join(",")))
        }
    }

    fn build_flags<S>(&self, class_dir: S) -> Vec<OsString>
        where S: AsRef<OsStr> {
        // Default flags
        let mut flags = vec!["--json-ui".into(),
            "--trace-json-extended".into(),
            "--unwinding-assertions".into()];

        if let Some(unwind) = self.current_unwind {
            flags.push("--unwind".into());
            flags.push(unwind.to_string().into());
        }

        if let Some(depth) = self.jbmc_config.depth.as_ref() {
            flags.push("--depth".into());
            flags.push(depth.to_string().into());
        }

        if ! self.jbmc_config.unwind_sets.is_empty() {
            flags.push("--unwindset".into());
            flags.push(self.jbmc_config.unwind_sets.join(",").into());
        }

        if let Some(&(lower, upper)) = self.jbmc_config.primitive_input_bound.as_ref() {
            flags.push("--java-assume-inputs-interval".into());
            flags.push(format!("[{}:{}]", lower, upper).into());
        }

        if let Some(bound) = self.jbmc_config.nondet_array_len_bound.as_ref() {
            flags.push("--max-nondet-array-length".into());
            flags.push(bound.to_string().into());
        }
        
        if let Some(bound) = self.jbmc_config.nondet_tree_depth_bound.as_ref() {
            flags.push("--max-nondet-tree-depth".into());
            flags.push(bound.to_string().into());
        }

        if let Some(bound) = self.jbmc_config.nondet_string_len_bound.as_ref() {
            flags.push("--max-nondet-string-length".into());
            flags.push(bound.to_string().into());
        }

        if self.jbmc_config.assume_input_non_null {
            flags.push("--java-assume-inputs-non-null".into());
        }

        if self.jbmc_config.assume_input_integral {
            flags.push("--java-assume-inputs-integral".into());
        }

        for input_val in self.jbmc_config.string_inputs.iter() {
            flags.push("--string-input-value".into());
            flags.push(input_val.into());
        }

        flags.push("--classpath".into());
        // Build classpath
        // input classpath must be put before the extra ones
        // So that in case of class override input takes priority
        let mut cp_joined = class_dir.as_ref().to_os_string();
        for cp in self.extra_class_path.iter() {
            cp_joined.push(":");
            cp_joined.push(cp);
        }
        flags.push(cp_joined);
        flags.extend(self.extra_flags.iter().cloned());
        flags
    }

    pub fn run<S: AsRef<str>, Sp: AsRef<OsStr>>(&self, entrance: S, class_dir: Sp) -> Result<JBMCLogs, TraceError>{
        let mut args = self.build_flags(class_dir);
        args.push(OsString::from(entrance.as_ref()));

        let mut cmd = Command::new(self.jbmc_config.bin_path.as_os_str());
        cmd.args(args);

        let result = cmd.output()?;
        Ok(serde_json::from_slice(&result.stdout).context(result.stdout)?)
    }
}

// TODO: Unit tests