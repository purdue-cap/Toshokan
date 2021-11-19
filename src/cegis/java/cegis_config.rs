use std::path::PathBuf;
use std::ffi::OsString;
use crate::frontend::EncoderSource;
use crate::frontend::java::{JBMCConfig, JSketchConfig};
use derive_builder::Builder;

pub struct CEGISConfig {
    params: CEGISConfigParams
}

impl CEGISConfig {
    pub fn new(params: CEGISConfigParams) -> Self {
        Self {
            params: params
        }
    }
    pub fn get_params(&self) -> &CEGISConfigParams {&self.params}
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct CEGISConfigParams {
    pub jbmc_config: JBMCConfig,
    pub javac_bin: PathBuf,
    pub jsketch_config: JSketchConfig,
    #[builder(setter(each = "lib_func"))]
    pub lib_funcs: Vec<String>,
    pub c_e_encoder_src: EncoderSource,
    #[builder(setter(each = "verif_classpath"), default)]
    pub verif_classpaths: Vec<OsString>,
    #[builder(setter(each = "verif_src_file"), default)]
    pub verif_src_files: Vec<OsString>,
    pub verif_entrance: String,
    #[builder(setter(each = "synth_file"), default)]
    pub synth_files: Vec<PathBuf>,
    #[builder(setter(each = "output_class"))]
    pub output_classes: Vec<String>,
    pub n_inputs: usize,
    #[builder(default = "10")]
    pub n_unknowns: usize,
    pub output_dir: PathBuf,
    #[builder(default = "false")]
    pub keep_tmp: bool,
    #[builder(default = "false")]
    pub enable_record: bool,
}