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
    StateQuery {
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
    pub hashcode_types: HashSet<String>,
    pub n_inputs: usize,
    pub v_p_config: VerifyPointsConfig,
    pub init_n_unknowns: usize,
    pub init_hist_cap_padding: usize,
    pub synthesis_sketch_config: SketchConfig,
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

pub struct CEGISConfigBuilder {
    sketch_fe_bin: Option<PathBuf>,
    sketch_be_bin: Option<PathBuf>,
    sketch_home: Option<PathBuf>,
    impl_file: Option<PathBuf>,
    harness_func_name: Option<String>,
    func_config: Option<HashMap<String, FuncConfig>>,
    hashcode_types: Option<HashSet<String>>,
    n_inputs: Option<usize>,
    v_p_config: Option<VerifyPointsConfig>,
    init_n_unknowns: Option<usize>,
    init_hist_cap_padding: Option<usize>,
    synthesis_sketch_config: Option<SketchConfig>,
    excluded_holes: Option<HashSet<ExcludedHole>>,
    enable_record: Option<bool>,
    keep_tmp: Option<bool>,
    retry_strategy_config: Option<RetryStrategyConfig>,
    cand_encoder_src: Option<EncoderSource>,
    input_tmp_file: Option<PathBuf>,
    be_verify_flags: Option<Vec<OsString>>,
    c_e_encoder_src: Option<EncoderSource>,
    generation_encoder_src: Option<EncoderSource>,
    c_e_names: Option<Vec<String>>,
    trace_timeout: Option<f32>,
    empty_harness_call: Option<bool>
}

impl CEGISConfigParams {
    pub fn get_trace_config_strings(&self) -> Vec<String> {
        let mut config : Vec<String> = self.func_config.iter().map(|(name, config)| {
            match config {
                FuncConfig::Init{args: _} => {
                    format!("trace_only@{}", name)
                },
                FuncConfig::NonPure{args:_, state_arg_idx: _} |
                FuncConfig::Pure{args: _} |
                FuncConfig::StateQuery{args:_, state_arg_idx: _}  => {
                    name.clone()
                }
            }
        }).collect();
        config.extend(
            self.hashcode_types.iter().map(|type_name| {
                format!("hashcode_type@{}", type_name)
            })
        );
        config
    }
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
                hashcode_types: HashSet::new(),
                harness_func_name: harness_func_name.as_ref().to_string(),
                n_inputs: n_inputs,
                v_p_config: v_p_config,
                init_n_unknowns: init_n_unknowns,
                init_hist_cap_padding: init_n_unknowns,
                synthesis_sketch_config: Default::default(),
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
            func_config: &[(&str, FuncConfig)], hashcode_types: &[&str], harness_func_name: S,
            n_inputs: usize, v_p_config: VerifyPointsConfig,
            init_n_unknowns: usize, init_hist_cap_padding: usize,
            excluded_holes: I,
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
                hashcode_types: hashcode_types.iter().map(|t| t.to_string()).collect(),
                harness_func_name: harness_func_name.as_ref().to_string(),
                n_inputs: n_inputs,
                v_p_config: v_p_config,
                init_n_unknowns: init_n_unknowns,
                init_hist_cap_padding: init_hist_cap_padding,
                synthesis_sketch_config: Default::default(),
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

// TODO: Refactor with derive_builder crate
impl CEGISConfigBuilder {
    pub fn new() -> Self {
        Self {
            sketch_fe_bin: None,
            sketch_be_bin: None,
            sketch_home: None,
            impl_file: None,
            harness_func_name: None,
            func_config: None,
            hashcode_types: None,
            n_inputs: None,
            v_p_config: None,
            init_n_unknowns: None,
            init_hist_cap_padding: None,
            synthesis_sketch_config: None,
            excluded_holes: None,
            enable_record: None,
            keep_tmp: None,
            retry_strategy_config: None,
            cand_encoder_src: None,
            input_tmp_file: None,
            be_verify_flags: None,
            c_e_encoder_src: None,
            generation_encoder_src: None,
            c_e_names: None,
            trace_timeout: None,
            empty_harness_call: None
        }
    }
    pub fn set_sketch_fe_bin<P: AsRef<Path>>(mut self, sketch_fe_bin: P) -> Self {
        self.sketch_fe_bin = Some(sketch_fe_bin.as_ref().to_path_buf());
        self
    }
    pub fn set_sketch_be_bin<P: AsRef<Path>>(mut self, sketch_be_bin: P) -> Self {
        self.sketch_be_bin = Some(sketch_be_bin.as_ref().to_path_buf());
        self
    }
    pub fn set_sketch_home<P: AsRef<Path>>(mut self, sketch_home: Option<P>) -> Self {
        self.sketch_home = sketch_home.map(|p| p.as_ref().to_path_buf());
        self
    }
    pub fn set_impl_file<P: AsRef<Path>>(mut self, impl_file: P) -> Self {
        self.impl_file = Some(impl_file.as_ref().to_path_buf());
        self
    }
    pub fn set_harness_func_name<S: AsRef<str>>(mut self, harness_func_name: S) -> Self {
        self.harness_func_name = Some(harness_func_name.as_ref().to_string());
        self
    }
    pub fn set_pure_func_config<I: Iterator<Item=(&'static str, usize)>>(mut self, func_config: I) -> Self{
        self.func_config = Some(func_config.map(|(name, args)|
            (name.to_string(), FuncConfig::Pure{args: args})
        ).collect());
        self
    }
    pub fn set_func_config<I: Iterator<Item=(&'static str, FuncConfig)>>(mut self, func_config: I) -> Self {
        self.func_config = Some(func_config.map(|(name, config)| 
                    (name.to_string(), config)).collect());
        self
    }
    pub fn set_hashcode_types<I: Iterator<Item=&'static str>>(mut self, hashcode_types: I) -> Self {
        self.hashcode_types = Some(hashcode_types.map(|t| t.to_string()).collect());
        self
    }
    pub fn set_n_inputs(mut self, n_inputs: usize) -> Self{
        self.n_inputs = Some(n_inputs);
        self
    }
    pub fn set_v_p_config(mut self, v_p_config: VerifyPointsConfig) -> Self{
        self.v_p_config = Some(v_p_config);
        self
    }
    pub fn set_init_n_unknowns(mut self, init_n_unknowns: usize) -> Self{
        self.init_n_unknowns = Some(init_n_unknowns);
        self
    }
    pub fn set_init_hist_cap_padding(mut self, init_hist_cap_padding: usize) -> Self{
        self.init_hist_cap_padding = Some(init_hist_cap_padding);
        self
    }
    pub fn set_synthesis_sketch_config(mut self, config: SketchConfig) -> Self{
        self.synthesis_sketch_config = Some(config);
        self
    }
    pub fn set_excluded_holes<I: Iterator<Item=ExcludedHole>>(mut self, excluded_holes: I) ->Self{
        self.excluded_holes = Some(excluded_holes.collect());
        self
    }
    pub fn set_enable_record(mut self, enable_record: bool) -> Self{
        self.enable_record = Some(enable_record);
        self
    }
    pub fn set_keep_tmp(mut self, keep_tmp: bool) -> Self {
        self.keep_tmp = Some(keep_tmp);
        self
    }
    pub fn set_retry_strategy_config(mut self, retry_strategy_config: RetryStrategyConfig) -> Self{
        self.retry_strategy_config = Some(retry_strategy_config);
        self
    }
    pub fn set_cand_encoder_src(mut self, cand_encoder_src: EncoderSource) -> Self {
        self.cand_encoder_src = Some(cand_encoder_src);
        self
    }
    pub fn set_cand_encoder_src_file<P: AsRef<Path>>(mut self, cand_encoder_src: P) -> Self {
        self.cand_encoder_src = Some(EncoderSource::LoadFromFile(cand_encoder_src.as_ref().to_path_buf()));
        self
    }
    pub fn set_input_tmp_file<P: AsRef<Path>>(mut self, input_tmp_file: P) -> Self {
        self.input_tmp_file = Some(input_tmp_file.as_ref().to_path_buf());
        self
    }
    pub fn set_be_verify_flags<I: Iterator<Item=OsString>>(mut self, be_verify_flags: I) -> Self {
        self.be_verify_flags = Some(be_verify_flags.collect());
        self
    }
    pub fn set_c_e_encoder_src(mut self, c_e_encoder_src: EncoderSource) -> Self {
        self.c_e_encoder_src = Some(c_e_encoder_src);
        self
    }
    pub fn set_c_e_encoder_src_file<P: AsRef<Path>>(mut self, c_e_encoder_src: P) -> Self {
        self.c_e_encoder_src = Some(EncoderSource::LoadFromFile(c_e_encoder_src.as_ref().to_path_buf()));
        self
    }
    pub fn set_generation_encoder_src(mut self, generation_encoder_src: EncoderSource) -> Self {
        self.generation_encoder_src = Some(generation_encoder_src);
        self
    }
    pub fn set_generation_encoder_src_file<P: AsRef<Path>>(mut self, generation_encoder_src: P) -> Self {
        self.generation_encoder_src = Some(EncoderSource::LoadFromFile(generation_encoder_src.as_ref().to_path_buf()));
        self
    }
    pub fn set_c_e_names<I: Iterator<Item=&'static str>>(mut self, c_e_names: I) -> Self {
        self.c_e_names = Some(c_e_names.map(|s| s.to_string()).collect());
        self
    }
    pub fn set_trace_timeout(mut self, trace_timeout: f32) -> Self {
        self.trace_timeout = Some(trace_timeout);
        self
    }
    pub fn set_empty_harness_call(mut self, empty_harness_call: bool) -> Self {
        self.empty_harness_call = Some(empty_harness_call);
        self
    }
    pub fn build(self) -> Option<CEGISConfig> {
        Some(CEGISConfig {
            params: CEGISConfigParams {
                sketch_be_bin: self.sketch_be_bin?,
                sketch_fe_bin: self.sketch_fe_bin?,
                sketch_home: self.sketch_home,
                impl_file: self.impl_file?,
                func_config: self.func_config?,
                hashcode_types: self.hashcode_types.unwrap_or(HashSet::new()),
                harness_func_name: self.harness_func_name.unwrap_or("main".to_string()),
                n_inputs: self.n_inputs?,
                v_p_config: self.v_p_config.unwrap_or(VerifyPointsConfig::NoSpec),
                init_n_unknowns: self.init_n_unknowns?,
                init_hist_cap_padding: self.init_hist_cap_padding.unwrap_or(self.init_n_unknowns?),
                synthesis_sketch_config: self.synthesis_sketch_config.unwrap_or(Default::default()),
                excluded_holes: self.excluded_holes.unwrap_or(HashSet::new()),
                enable_record: self.enable_record.unwrap_or(false),
                keep_tmp: self.keep_tmp.unwrap_or(false),
                retry_strategy_config: self.retry_strategy_config.unwrap_or(RetryStrategyConfig::Simple(20)),
                cand_encoder_src: self.cand_encoder_src.unwrap_or(EncoderSource::Rewrite),
                input_tmp_file: self.input_tmp_file,
                be_verify_flags: self.be_verify_flags,
                c_e_encoder_src: self.c_e_encoder_src?,
                generation_encoder_src: self.generation_encoder_src?,
                c_e_names: self.c_e_names.unwrap_or(vec![]),
                trace_timeout: self.trace_timeout,
                empty_harness_call: self.empty_harness_call.unwrap_or(false)
            },
            input_tmp_path: None
        })
    }
}

#[derive(Clone)]
pub struct SketchConfig {
    pub bnd_inline_amnt: Option<usize>,
    pub bnd_unroll_amnt: Option<usize>,
    pub bnd_inbits: Option<usize>,
    pub bnd_cbits: Option<usize>,
    pub slv_nativeints: bool,
    pub slv_parallel: bool,
    pub slv_p_cpus: Option<usize>,
    pub slv_randassign: bool,
    pub slv_randdegree: Option<usize>,
    pub extra_options: Vec<String>
}

impl SketchConfig {
    pub fn to_options(&self) -> Vec<String> {
        let mut options = Vec::new();

        macro_rules! push_options {
            ($name:ident, $cmdline:literal) => {
                if let Some($name) = self.$name {
                    options.push($cmdline.to_string());
                    options.push($name.to_string())
                }
            };
            (flag: $name:ident, $cmdline:literal) => {
                if self.$name {
                    options.push($cmdline.to_string());
                }
            };
        }

        push_options!(bnd_inline_amnt, "--bnd-inline-amnt");
        push_options!(bnd_unroll_amnt, "--bnd-unroll-amnt");
        push_options!(bnd_inbits, "--bnd-inbits"); 
        push_options!(bnd_cbits, "--bnd-cbits");
        push_options!(flag: slv_nativeints, "--slv-nativeints");
        push_options!(flag: slv_parallel, "--slv-parallel");
        push_options!(slv_p_cpus, "--slv-p-cpus");
        push_options!(flag: slv_randassign, "--slv-randassign");
        push_options!(slv_randdegree, "--slv-randdegree");
        options.append(&mut self.extra_options.clone());
        options
    }
}

impl Default for SketchConfig{
    fn default() -> Self {
        SketchConfig {
            bnd_inline_amnt: None,
            bnd_unroll_amnt: None,
            bnd_inbits: None,
            bnd_cbits: None,
            slv_nativeints: false,
            slv_parallel: false,
            slv_p_cpus: None,
            slv_randassign: false,
            slv_randdegree: None,
            extra_options: vec![]
        }
    }
}

impl ToString for SketchConfig{
    fn to_string(&self) -> String{
        self.to_options().join(" ")
    }
}