use std::ffi::{OsString, OsStr};
use std::path::PathBuf;
use derive_builder::Builder;
use std::collections::HashMap;
use crate::backend::TraceError;
use serde_json::{Value, json};
use std::process::Command;
#[cfg(feature = "inline_java_tracer")]
use std::io::{Error as IOError, Write};
#[cfg(feature = "inline_java_tracer")]
use tempfile::{NamedTempFile, TempPath};

pub struct JavaTracerRunner<'c, 'l> {
    config: &'c JavaTracerConfig,
    lib_funcs: Vec<&'l str>,
    #[cfg(feature = "inline_java_tracer")]
    temp_agent_jar: Option<TempPath>,
    pub extra_tracer_config: HashMap<String, Value>,
    pub extra_class_path: Vec<PathBuf>,
}

fn parse_lib_func_name(name: &str)-> Option<(&str, &str)> {
    let dot_idx = name.rfind(".")?;
    let class_name = &name[..dot_idx];
    let left_par_idx = name.rfind("(")?;
    let func_name = &name[dot_idx+1..left_par_idx];
    Some((class_name, func_name))
}

#[derive(Builder)]
#[builder(pattern = "owned", setter(into))]
pub struct JavaTracerConfig {
    pub java_bin_path: PathBuf,
    #[builder(default)]
    pub agent_jar_path: Option<PathBuf>,
    #[builder(setter(each = "other_tracer_config"), default)]
    pub other_tracer_configs: HashMap<String, Value>,
    #[builder(setter(each = "other_class_path"), default)]
    pub other_class_paths: Vec<PathBuf>,
}

#[cfg(feature = "inline_java_tracer")]
static INLINED_AGENT_JAR: &'static [u8] = include_bytes!("../../../javaTracer/target/javaTracer-1.0-SNAPSHOT-jar-with-dependencies.jar");

impl<'c, 'l> JavaTracerRunner<'c, 'l> {
    pub fn new<I>(config: &'c JavaTracerConfig, lib_func_iter: I) -> Self
        where I: IntoIterator<Item=&'l str>{
        Self {
            config: config,
            #[cfg(feature = "inline_java_tracer")]
            temp_agent_jar: None,
            lib_funcs: lib_func_iter.into_iter().collect(),
            extra_tracer_config: HashMap::new(),
            extra_class_path: vec![],
        }
    }
    #[cfg(feature = "inline_java_tracer")]
    pub fn new_with_inlined_jar<I>(config: &'c JavaTracerConfig, lib_func_iter: I) -> Result<Self, IOError>
        where I: IntoIterator<Item=&'l str>{
        let mut temp_agent_jar = NamedTempFile::new()?;
        temp_agent_jar.write(INLINED_AGENT_JAR)?;
        temp_agent_jar.flush()?;
        Ok(Self {
            config: config,
            temp_agent_jar: Some(temp_agent_jar.into_temp_path()),
            lib_funcs: lib_func_iter.into_iter().collect(),
            extra_tracer_config: HashMap::new(),
            extra_class_path: vec![],
        })
    }

    fn build_java_agent_flag(&self, agent_jar_path: &OsStr) -> OsString{
        let mut java_agent_flag = OsString::new();

        java_agent_flag.push("-javaagent:");
        java_agent_flag.push(agent_jar_path);

        java_agent_flag.push("=");

        let method_info_dicts = self.lib_funcs.iter()
            .flat_map(|name| parse_lib_func_name(name))
            .map(|(class_name, method_name)|
                json!({
                    "className": class_name,
                    "method": method_name
                })).collect::<Vec<_>>();
        
        let mut tracer_config: serde_json::Map<String, Value> = serde_json::Map::new();
        tracer_config.insert("methods".into(), method_info_dicts.into());
        tracer_config.extend(self.config.other_tracer_configs.clone());
        tracer_config.extend(self.extra_tracer_config.clone());
        let config_string = Value::Object(tracer_config).to_string();
        java_agent_flag.push(config_string);

        java_agent_flag
    }

    fn build_flags<S>(&self, class_dir: S) -> Vec<OsString> 
        where S: AsRef<OsStr> { 
        let mut flags = Vec::new();
        #[cfg(not(feature = "inline_java_tracer"))]
        if let Some(ref agent_jar_path) = self.config.agent_jar_path {
            flags.push(self.build_java_agent_flag(agent_jar_path.as_os_str()));
        }
        #[cfg(feature = "inline_java_tracer")]
        if let Some(ref agent_jar_path) = self.temp_agent_jar {
            flags.push(self.build_java_agent_flag(agent_jar_path.as_os_str()));
        } else if let Some(ref agent_jar_path) = self.config.agent_jar_path {
            flags.push(self.build_java_agent_flag(agent_jar_path.as_os_str()));
        }

        flags.push("-cp".into());
        let mut cp_combined: OsString = class_dir.as_ref().into();
        for cp in self.config.other_class_paths.iter().chain(self.extra_class_path.iter()) {
            cp_combined.push(":");
            cp_combined.push(cp);
        }

        return flags;
    }

    pub fn run<S: AsRef<str>, Sp: AsRef<OsStr>>(&self, entrance: S, class_dir: Sp) -> Result<Vec<u8>, TraceError>{
        let mut args = self.build_flags(class_dir);
        args.push(OsString::from(entrance.as_ref()));

        let mut cmd = Command::new(self.config.java_bin_path.as_os_str());
        cmd.args(args);

        let result = cmd.output()?;
        Ok(result.stderr)
    }
}