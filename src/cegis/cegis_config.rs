use crate::frontend::{SketchRunner, EncoderSource};
use std::path::{Path, PathBuf};
use std::collections::{HashSet, HashMap};
use std::ffi::OsString;
use rand::Rng;
use super::{CEGISState, RetryStrategy};
use super::retry_strategies::{SimpleRetryStrategy, NeverRetryStrategy};

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum ExcludedHole {
    Name(String),
    Position(isize, isize)
}

#[derive(Clone, Debug)]
pub enum FuncConfig {
    Pure {args: usize},
    NonPure {
        args: usize,
        state_arg_idx: usize
    },
    Init {args: usize}
}

pub struct CEGISConfigParams {
    pub sketch_fe_bin: PathBuf,
    pub sketch_be_bin: PathBuf,
    pub sketch_home: Option<PathBuf>,
    pub impl_file: PathBuf,
    pub harness_func_name: String,
    pub func_config: HashMap<String, FuncConfig>,
    pub n_inputs: usize,
    pub v_p_config: VerifyPointsConfig,
    pub init_n_unknowns: usize,
    pub excluded_holes: HashSet<ExcludedHole>,
    pub enable_record: bool,
    pub keep_tmp: bool,
    pub retry_strategy_config: RetryStrategyConfig,
    pub cand_encoder_src: EncoderSource,
    pub input_tmp_file: Option<PathBuf>,
    pub be_verify_flags: Option<Vec<OsString>>,
    pub c_e_encoder_src: EncoderSource,
    pub generation_encoder_src: EncoderSource,
    pub c_e_names: Vec<String>,
    pub trace_timeout: Option<f32>,
    pub empty_harness_call: bool
}

pub enum VerifyPointsConfig {
    Fixed(HashSet<Vec<isize>>),
    Permutation(HashSet<Vec<isize>>),
    Random(usize),
    RandomWithRange{
        num: usize,
        begin: usize,
        end: usize
    },
    NoSpec
}

pub enum RetryStrategyConfig {
    Simple(usize),
    Never
}

pub struct CEGISConfig {
    params: CEGISConfigParams,
    input_tmp_path: Option<PathBuf>
}

impl CEGISConfig {
    pub fn new<P: AsRef<Path>, S: AsRef<str>, I: Iterator<Item=ExcludedHole>>(
            sketch_fe_bin: P, sketch_be_bin: P, sketch_home: Option<P>, impl_file: P,
            func_config: &[(&str, usize)], harness_func_name: S,
            n_inputs: usize, v_p_config: VerifyPointsConfig,
            init_n_unknowns: usize, excluded_holes: I,
            pure_function: bool, enable_record: bool, keep_tmp: bool,
            synthesis_sk: P, verify_generation_sk: P, c_e_names: &[&str],
            trace_timeout: Option<f32>) -> Self {
        CEGISConfig {
            params: CEGISConfigParams {
                sketch_fe_bin: sketch_fe_bin.as_ref().to_path_buf(),
                sketch_be_bin: sketch_be_bin.as_ref().to_path_buf(),
                sketch_home: sketch_home.as_ref().map(|p| p.as_ref().to_path_buf()),
                impl_file: impl_file.as_ref().to_path_buf(),
                func_config: func_config.iter().map(|(name, args)| 
                    (name.to_string(),
                        if pure_function {
                            FuncConfig::Pure{args: *args}
                        } else {
                            FuncConfig::NonPure{args: *args, state_arg_idx: 0}
                        }
                    )
                ).collect(),
                harness_func_name: harness_func_name.as_ref().to_string(),
                n_inputs: n_inputs,
                v_p_config: v_p_config,
                init_n_unknowns: init_n_unknowns,
                excluded_holes: excluded_holes.collect(),
                enable_record: enable_record,
                keep_tmp: keep_tmp,
                retry_strategy_config: RetryStrategyConfig::Simple(20),
                cand_encoder_src: EncoderSource::Rewrite,
                input_tmp_file: None,
                be_verify_flags: None,
                c_e_encoder_src: EncoderSource::LoadFromFile(synthesis_sk.as_ref().to_path_buf()),
                generation_encoder_src: EncoderSource::LoadFromFile(verify_generation_sk.as_ref().to_path_buf()),
                c_e_names: c_e_names.iter().map(|s| s.to_string()).collect(),
                trace_timeout: trace_timeout,
                empty_harness_call: false
            },
            input_tmp_path: None
        }
    }

    pub fn new_full_config<P: AsRef<Path>, S: AsRef<str>, I: Iterator<Item=ExcludedHole>>(
            sketch_fe_bin: P, sketch_be_bin: P, sketch_home: Option<P>, impl_file: P,
            func_config: &[(&str, FuncConfig)], harness_func_name: S,
            n_inputs: usize, v_p_config: VerifyPointsConfig,
            init_n_unknowns: usize, excluded_holes: I,
            enable_record: bool, keep_tmp: bool,
            synthesis_sk: P, verify_generation_sk: P, c_e_names: &[&str],
            trace_timeout: Option<f32>) -> Self {
        CEGISConfig {
            params: CEGISConfigParams {
                sketch_fe_bin: sketch_fe_bin.as_ref().to_path_buf(),
                sketch_be_bin: sketch_be_bin.as_ref().to_path_buf(),
                sketch_home: sketch_home.as_ref().map(|p| p.as_ref().to_path_buf()),
                impl_file: impl_file.as_ref().to_path_buf(),
                func_config: func_config.iter().map(|(name, config)| 
                    (name.to_string(), config.clone())
                ).collect(),
                harness_func_name: harness_func_name.as_ref().to_string(),
                n_inputs: n_inputs,
                v_p_config: v_p_config,
                init_n_unknowns: init_n_unknowns,
                excluded_holes: excluded_holes.collect(),
                enable_record: enable_record,
                keep_tmp: keep_tmp,
                retry_strategy_config: RetryStrategyConfig::Simple(20),
                cand_encoder_src: EncoderSource::Rewrite,
                input_tmp_file: None,
                be_verify_flags: None,
                c_e_encoder_src: EncoderSource::LoadFromFile(synthesis_sk.as_ref().to_path_buf()),
                generation_encoder_src: EncoderSource::LoadFromFile(verify_generation_sk.as_ref().to_path_buf()),
                c_e_names: c_e_names.iter().map(|s| s.to_string()).collect(),
                trace_timeout: trace_timeout,
                empty_harness_call: false
            },
            input_tmp_path: None
        }
    }

    pub fn get_params(&self) -> &CEGISConfigParams {&self.params}

    pub fn get_params_mut(&mut self) -> &mut CEGISConfigParams {&mut self.params}

    pub fn is_be_config_unresolved(&self) -> bool {self.params.input_tmp_file.is_none() || self.params.be_verify_flags.is_none()}

    pub fn set_input_tmp_path(&mut self, path: PathBuf) {self.input_tmp_path = Some(path)}

    pub fn get_input_tmp_path(&self) -> Option<&Path> {self.input_tmp_path.as_ref().map(|p| p.as_path())}

    pub fn populate_be_config(&mut self, runner: &mut SketchRunner) {
        if let Some(path) = self.params.input_tmp_file.clone() {
            self.set_input_tmp_path(path);
        }
        if let Some(ref flags) = self.params.be_verify_flags {
            runner.set_be_verify_flags(flags);
        }
    }

    pub fn new_retry_strategy(&self) -> Box<dyn RetryStrategy> {
        match self.params.retry_strategy_config {
            RetryStrategyConfig::Simple(retry_amount) => Box::new(SimpleRetryStrategy::new(retry_amount)),
            RetryStrategyConfig::Never => Box::new(NeverRetryStrategy{})
        } 
    }

    pub fn populate_v_p_s(&self, state: &mut CEGISState) -> Option<()> {
        match self.params.v_p_config {
            VerifyPointsConfig::Fixed(ref points) | VerifyPointsConfig::Permutation(ref points) => {
                for point in points {
                    state.add_verify_point(point.clone())?;
                }
            },
            VerifyPointsConfig::Random(num) => {
                let mut rng = rand::thread_rng();
                for _ in 0..num {
                    state.add_verify_point(
                        (0..self.params.n_inputs).map(|_|{
                            rng.gen::<isize>()
                        }).collect()
                    )?;
                }
            },
            VerifyPointsConfig::RandomWithRange{num, begin, end} => {
                let mut rng = rand::thread_rng();
                for _ in 0..num {
                    state.add_verify_point(
                        (0..self.params.n_inputs).map(|_|{
                            rng.gen_range(begin, end) as isize
                        }).collect()
                    )?;
                }
            },
            _ => {}

        };
        Some(())
    }

}