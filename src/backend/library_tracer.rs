use std::path::Path;
use std::process::Command;
use std::io::{Write, BufRead, BufReader};
use std::fs::File;
use regex::Regex;
use super::CFlagManager;
use super::build_tracer::{build_tracer_to_file, COMPILATION_DB_FILE_NAME};
use log::error;

pub struct LibraryTracer<'i, 'n, 'w> {
    impl_file: &'i Path,
    lib_func_name: &'n str,
    flag_manager: CFlagManager,
    work_dir: Option<&'w Path>
}

impl<'i, 'n, 'w> LibraryTracer<'i, 'n, 'w> {
    pub fn new(impl_file: &'i Path, lib_func_name: &'n str, sketch_home: &Path) -> Self {
        let mut tracer = LibraryTracer {
            impl_file: impl_file,
            lib_func_name: lib_func_name,
            flag_manager: CFlagManager::new("clang++"),
            work_dir: None
        };
        tracer.flag_manager.add_include_path(sketch_home.join("include"));
        tracer
    }

    pub fn set_work_dir(&mut self, work_dir: &'w Path) {
        self.work_dir = Some(work_dir);
    }

    pub fn build_tracer_src<P: AsRef<Path>>(&self, main_file: P) -> Option<()> {
        let output_file = self.work_dir?.join("tracer.cpp");
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
            Ok(()) => {Some(())}
            Err(code) => {
                error!(target: "LibraryTracer", "CPP function error code: {}", code);
                None
            }
        }
    }

    pub fn build_tracer_bin<P: AsRef<Path>>(&self, non_main_src_files: &[P]) -> Option<()> {
        let mut all_src_files : Vec<&Path> = 
            non_main_src_files.iter().map(|path| path.as_ref()).collect();
        let tracer_file = self.work_dir?.join("tracer.cpp");
        all_src_files.push(tracer_file.as_path());
        all_src_files.push(self.impl_file);
        let out_bin_file = self.work_dir?.join("tracer");
        let mut compiler_cmd = self.flag_manager.get_binary_build_cmd(all_src_files.as_slice(), out_bin_file.as_path())?;
        let status = compiler_cmd.status().ok()?;
        if status.success() {
            Some(())
        } else  {
            None
        }
    }

    fn match_log_line<S: AsRef<str>>(&self, line: S) -> Option<(Vec<isize>, isize)> {
        let log_regex = Regex::new(format!(r"{}\(([\d, ]+)\) = (\d+)", self.lib_func_name).as_str()).ok()?;
        let caps = log_regex.captures(line.as_ref())?;
        let args = caps.get(0)?.as_str().split(",")
            .map(|arg| arg.trim()).map(|arg| arg.parse::<isize>().ok())
            .collect::<Option<Vec<_>>>()?;
        let rtn = caps.get(1)?.as_str().trim().parse::<isize>().ok()?;
        Some((args, rtn))
    }

    pub fn collect_traces(&self) -> Option<Vec<(Vec<isize>, isize)>> {
        let mut tracing_cmd = Command::new(self.work_dir?.join("tracer"));
        let tracing_output = tracing_cmd.output().ok()?;
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