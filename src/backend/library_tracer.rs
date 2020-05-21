use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::{Write, BufRead, BufReader};
use std::fs::File;
use regex::Regex;
use super::CFlagManager;
use super::build_tracer::{build_tracer_to_file, COMPILATION_DB_FILE_NAME};
use log::{error, trace};

pub struct LibraryTracer<'i, 'ln, 'hn, 'w> {
    impl_file: &'i Path,
    lib_func_name: &'ln str,
    harness_func_name: &'hn str,
    flag_manager: CFlagManager,
    work_dir: Option<&'w Path>,
    current_base_name: Option<String>
}

impl<'i, 'ln, 'hn, 'w> LibraryTracer<'i, 'ln, 'hn, 'w> {
    pub fn new(impl_file: &'i Path, lib_func_name: &'ln str, harness_func_name: &'hn str, sketch_home: &Path) -> Self {
        let mut tracer = LibraryTracer {
            impl_file: impl_file,
            lib_func_name: lib_func_name,
            harness_func_name: harness_func_name,
            flag_manager: CFlagManager::new("clang++"),
            work_dir: None,
            current_base_name: None
        };
        tracer.flag_manager.add_include_path(sketch_home.join("include"));
        tracer
    }

    pub fn set_work_dir(&mut self, work_dir: &'w Path) {
        self.work_dir = Some(work_dir);
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

    fn match_log_line<S: AsRef<str>>(&self, line: S) -> Option<(Vec<isize>, isize)> {
        trace!(target: "LibraryTracer", "Read trace log: {}", line.as_ref());
        let log_regex = Regex::new(format!(r"{}\(([-\d, ]+)\) = (-?\d+)", self.lib_func_name).as_str()).ok()?;
        trace!(target: "LibraryTracer", "Regex: {:?}", log_regex);
        let caps = log_regex.captures(line.as_ref())?;
        trace!(target: "LibraryTracer", "Captures: {:?}", caps);
        let args = caps.get(1)?.as_str().split(",")
            .map(|arg| arg.trim()).map(|arg| arg.parse::<isize>().ok())
            .collect::<Option<Vec<_>>>()?;
        let rtn = caps.get(2)?.as_str().trim().parse::<isize>().ok()?;
        trace!(target: "LibraryTracer", "result: {:?}", (&args, &rtn));
        Some((args, rtn))
    }

    pub fn collect_traces(&self) -> Option<Vec<(Vec<isize>, isize)>> {
        let mut tracing_cmd = Command::new(self.work_dir?.join(format!("{}_tracer", self.current_base_name.as_ref()?)));
        let tracing_output = tracing_cmd.output().ok()?;
        trace!(target: "LibraryTracer", "Sketch tracing_output.status: {:?}", tracing_output.status.clone());
        trace!(target: "LibraryTracer", "Sketch tracing_output.stdout: {}",
            String::from_utf8(tracing_output.stdout.clone()).ok()
            .unwrap_or("<Failure>".to_string())
        );
        trace!(target: "LibraryTracer", "Sketch tracing_output.stderr: {}",
            String::from_utf8(tracing_output.stderr.clone()).ok()
            .unwrap_or("<Failure>".to_string())
        );
        let err_reader = BufReader::new(tracing_output.stderr.as_slice());
        let mut logs = Vec::new();
        for line_result in err_reader.lines() {
            if let Ok(line) = line_result {
                if let Some(log) = self.match_log_line(line) {
                    logs.push(log);
                }
            }
        }
        Some(logs)
    }
}

// TODO: Unit tests