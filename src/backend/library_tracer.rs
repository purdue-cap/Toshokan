use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{thread, time};
use std::io::{Read, Write, BufRead, BufReader};
use std::fs::File;
use super::{CFlagManager, TraceError};
use super::build_tracer::{build_tracer_to_file, COMPILATION_DB_FILE_NAME};
use crate::cegis::TraceLog;
use log::{error, trace, warn};

pub struct LibraryTracer<'i, 'ln, 'hn, 'w> {
    impl_file: &'i Path,
    lib_func_name: &'ln str,
    harness_func_name: &'hn str,
    flag_manager: CFlagManager,
    work_dir: Option<&'w Path>,
    current_base_name: Option<String>,
    trace_timeout: Option<f32>
}

pub fn parse_log_from_json<S: AsRef<str>>(line: S) -> Result<TraceLog, TraceError> {
    trace!(target: "LibraryTracer", "Read trace log: {}", line.as_ref());
    let trace_log = serde_json::from_str(line.as_ref())?;
    trace!(target: "LibraryTracer", "result: {:?}", trace_log);
    Ok(trace_log)
}

static JSON_HPP: &'static str = include_str!("cpp/nlohmann/json.hpp");
static VOPS_H: &'static str = include_str!("cpp/vops.h");

impl<'i, 'ln, 'hn, 'w> LibraryTracer<'i, 'ln, 'hn, 'w> {
    pub fn new(impl_file: &'i Path, lib_func_name: &'ln str, harness_func_name: &'hn str, sketch_home: Option<&Path>, trace_timeout: Option<f32>) -> Self {
        let mut tracer = LibraryTracer {
            impl_file: impl_file,
            lib_func_name: lib_func_name,
            harness_func_name: harness_func_name,
            flag_manager: CFlagManager::new("clang++"),
            work_dir: None,
            current_base_name: None,
            trace_timeout: trace_timeout
        };
        if let Some(p) = sketch_home {
            tracer.flag_manager.add_include_path(p.join("include"));
        }
        tracer
    }

    pub fn set_work_dir(&mut self, work_dir: &'w Path) -> Option<()> {
        self.work_dir = Some(work_dir);
        self.add_static_file_to_work_dir("json.hpp", JSON_HPP)?;
        self.add_static_file_to_work_dir("vops.h", VOPS_H)?;
        Some(())
    }

    pub fn add_static_file_to_work_dir(&self, filename: &'static str, content: &'static str) -> Option<()> {
        let mut file = File::create(self.work_dir?.join(filename)).ok()?;
        write!(&mut file, "{}", content).ok()?;
        Some(())
    }

    pub fn set_base_name<S: AsRef<str>>(&mut self, base_name: S) {
        self.current_base_name = Some(base_name.as_ref().to_string());
    }

    pub fn build_tracer_src<P: AsRef<Path>>(&self, main_file: P) -> Option<PathBuf> {
        let output_file = self.work_dir?.join(format!("{}_tracer.cpp", self.current_base_name.as_ref()?));
        let compilation_db = self.work_dir?.join(COMPILATION_DB_FILE_NAME);
        let main_file_dir = main_file.as_ref().parent()?;
        let mut db_file = File::create(&compilation_db).ok()?;
        write!(db_file,
        r#"
        [
            {{
                "directory": "{}",
                "command": "{}",
                "file": "{}"
            }}
        ]
        "#, main_file_dir.to_str()?,
            self.flag_manager.get_compilation_cmd_line(main_file.as_ref())?,
            main_file.as_ref().to_str()?
        ).ok()?;
        match build_tracer_to_file(self.lib_func_name, main_file.as_ref(), output_file.as_path()) {
            Ok(()) => {Some(output_file)}
            Err(code) => {
                error!(target: "LibraryTracer", "CPP function error code: {}", code);
                None
            }
        }
    }

    pub fn build_entry_src(&self,  c_e_s: &Vec<Vec<isize>>) -> Option<PathBuf> {
        let mut func_calls_list: Vec<String> = Vec::new();
        let mut idx = 0;
        loop {
            if let Some(joined_params) =
                c_e_s.iter().map(|values| {
                    values.get(idx).map(|value| value.to_string())
                }).collect::<Option<Vec<String>>>().map(|params| params.join(", ")) {

                func_calls_list.push(format!(
r#"
    try{{
      ANONYMOUS::{harness_name}__WrapperNospec({arg_list});
      ANONYMOUS::{harness_name}__Wrapper({arg_list});
    }}catch(AssumptionFailedException& afe){{  }}
"#,
                    arg_list = joined_params,
                    harness_name = self.harness_func_name
                ));

                idx += 1;
            } else {
                break;
            }
        };

        let entry_src_content = format!(
r#"#include "{base_name}.h"
#include "vops.h"
int main(int argc, char** argv) {{
{func_calls}
}}
"#,
            func_calls = func_calls_list.join("\n"),
            base_name = self.current_base_name.as_ref()?
        );

        let output_file_name = self.work_dir?.join(format!("{}_test.cpp", self.current_base_name.as_ref()?));
        let mut output_file = File::create(&output_file_name).ok()?;
        write!(output_file, "{}", entry_src_content).ok().and(Some(output_file_name))
    }

    pub fn build_tracer_bin<P: AsRef<Path>>(&self, non_main_src_files: &[P]) -> Option<PathBuf> {
        let mut all_src_files : Vec<&Path> = 
            non_main_src_files.iter().map(|path| path.as_ref()).collect();
        let tracer_file = self.work_dir?.join(format!("{}_tracer.cpp", self.current_base_name.as_ref()?));
        all_src_files.push(tracer_file.as_path());
        all_src_files.push(self.impl_file);
        let out_bin_file = self.work_dir?.join(format!("{}_tracer", self.current_base_name.as_ref()?));
        let mut compiler_cmd = self.flag_manager.get_binary_build_cmd(all_src_files.as_slice(), out_bin_file.as_path())?;
        let status = compiler_cmd.status().ok()?;
        if status.success() {
            Some(out_bin_file)
        } else  {
            None
        }
    }

    pub fn collect_traces(&self) -> Result<Vec<TraceLog>, TraceError> {
        let mut tracing_cmd = Command::new(self.work_dir.ok_or(
                TraceError::OtherError("Work Directory not set")
            )?.join(format!("{}_tracer", self.current_base_name.as_ref().ok_or(
                TraceError::OtherError("Base Name not set")
            )?)));
        tracing_cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = tracing_cmd.spawn()?;
        let mut stdout: Vec<u8>;
        let mut stderr: Vec<u8>;
        if let Some(timeout) = self.trace_timeout {
            stdout = Vec::new();
            stderr = Vec::new();
            let timeout_duration = time::Duration::from_secs_f32(timeout);
            trace!(target: "LibraryTracer", "Tracing with timeout: {:?}", timeout_duration);
            let wall_start = time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        trace!(target: "LibraryTracer", "Sketch tracing_output.status: {:?}", status.clone());
                        break;
                    },
                    Ok(None) => {
                        let elapsed = time::Instant::now() - wall_start;
                        if  elapsed >= timeout_duration {
                            warn!(target: "LibraryTracer", "Tracing timed out!");
                            child.kill()?;
                            break;
                        }
                        thread::sleep(time::Duration::from_millis(100));
                    },
                    Err(e) => {
                        return Err(TraceError::IOError(e));
                    }
                }
            };
            child.stdout.as_mut().ok_or(
                TraceError::OtherError("Stdout not found")
            )?.read_to_end(&mut stdout)?;
            child.stderr.as_mut().ok_or(
                TraceError::OtherError("Stderr not found")
            )?.read_to_end(&mut stderr)?;
        } else {
            let output = child.wait_with_output()?;
            trace!(target: "LibraryTracer", "Sketch tracing_output.status: {:?}", output.status.clone());
            stdout = output.stdout;
            stderr = output.stderr;
        }
        trace!(target: "LibraryTracer", "Sketch tracing_output.stdout: {}",
            String::from_utf8(stdout.clone()).ok()
            .unwrap_or("<Failure>".to_string())
        );
        trace!(target: "LibraryTracer", "Sketch tracing_output.stderr: {}",
            String::from_utf8(stderr.clone()).ok()
            .unwrap_or("<Failure>".to_string())
        );
        let err_reader = BufReader::new(stderr.as_slice());
        let mut logs = Vec::new();
        for line_result in err_reader.lines() {
            let line = line_result?;
            let log = parse_log_from_json(line)?;
            logs.push(log);
        }
        Ok(logs)
    }
}

// TODO: More Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    #[test]
    fn parses_json_log_line() -> Result<(), Box<dyn Error>> {
        let json_str = r#"{"args":[5],"rtn":2}"#;
        let fixture = TraceLog {
            args: vec![json!(5)],
            rtn: json!(2)
        };
        let result = parse_log_from_json(json_str)?;
        assert_eq!(result, fixture);
        Ok(())
    }
}