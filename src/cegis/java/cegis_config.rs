use std::path::{Path, PathBuf};
use std::ffi::OsString;
use crate::frontend::EncoderSource;
use crate::frontend::java::{JBMCConfig, JSketchConfig, JavaTracerConfig};
use crate::frontend::java::{JBMCRunner, JavaTracerRunner};
use crate::frontend::traits::*;
use crate::backend::java::{JBMCLogAnalyzer, JavaTracerLogAnalyzer};
use crate::backend::TraceError;
use crate::backend::traits::*;
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

pub enum JavaClassVerifierConfig {
    Jbmc(JBMCConfig),
    Test(JavaTracerConfig)
}

impl From<JBMCConfig> for JavaClassVerifierConfig {
    fn from(c: JBMCConfig) -> Self {JavaClassVerifierConfig::Jbmc(c)}
}

impl From<JavaTracerConfig> for JavaClassVerifierConfig {
    fn from(c: JavaTracerConfig) -> Self {JavaClassVerifierConfig::Test(c)}
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct CEGISConfigParams {
    #[builder(setter(strip_option))]
    pub verifier_config: Option<JavaClassVerifierConfig>,
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
    #[builder(default = "10")]
    pub hist_cap_padding: usize,
    pub output_dir: PathBuf,
    #[builder(default = "false")]
    pub keep_tmp: bool,
    #[builder(default = "false")]
    pub enable_record: bool,
}

impl CEGISConfigParamsBuilder {
    pub fn jbmc_config(self, config: JBMCConfig) -> Self {self.verifier_config(config)}
    pub fn java_tracer_config(self, config: JavaTracerConfig) -> Self {self.verifier_config(config)}
}

impl CEGISConfig {
    pub fn make_java_verifier_controllers(&mut self, class_dir: &Path)
        -> Option<(Box<dyn RunJavaClassVerifier<Error=TraceError>>, Box<dyn AnalyzeTracingVerifierLog<Error=TraceError>>)> {
        match self.params.verifier_config.take()? {
            JavaClassVerifierConfig::Jbmc(config)
                => {
                    let mut runner = JBMCRunner::new(config.clone());
                    runner.extra_class_path.extend(
                        self.get_params().verif_classpaths.iter().cloned()
                    );
                    runner.extra_class_path.push(class_dir.as_os_str().into());
                    let log_analyzer = JBMCLogAnalyzer::new(
                        self.get_params().lib_funcs.iter()
                    );
                    Some((Box::new(runner), Box::new(log_analyzer)))
                },
            JavaClassVerifierConfig::Test(config)
                => {
                    let mut runner = JavaTracerRunner::new(config.clone(), self.get_params().lib_funcs.iter());
                    runner.extra_class_path.extend(
                        self.get_params().verif_classpaths.iter().map(|s| s.into())
                    );
                    runner.extra_class_path.push(class_dir.as_os_str().into());
                    let log_analyzer = JavaTracerLogAnalyzer::new();
                    Some((Box::new(runner), Box::new(log_analyzer)))
                }
        }

    }
}

test_fixture!(CEGISConfigParams, dummy, builder{
    verifier_config(JBMCConfig::test_fixture_dummy()),
    javac_bin(""),
    jsketch_config(JSketchConfig::test_fixture_dummy()),
    lib_funcs(vec![]),
    c_e_encoder_src(EncoderSource::test_fixture_dummy()),
    verif_entrance(""),
    output_classes(vec![]),
    n_inputs(0usize),
    output_dir("")
});