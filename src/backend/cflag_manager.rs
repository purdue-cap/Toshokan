use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub struct CFlagManager {
    compiler_bin: &'static str,
    include_paths: Vec<PathBuf>
}

impl CFlagManager {
    pub fn new(compiler: &'static str) -> Self{
        CFlagManager {
            compiler_bin: compiler,
            include_paths: Vec::new()
        }
    }

    pub fn get_include_paths(&self) -> Option<Vec<String>> {
        let mut include_path_run = Command::new(self.compiler_bin);
        include_path_run.args(&["-E", "-Wp,-v", "-"])
                .stdin(Stdio::null());
        let output = include_path_run.output().ok()?;
        let err_reader = BufReader::new(output.stderr.as_slice());
        let mut in_list = false;
        let mut paths = Vec::<String>::new();
        for line_result in err_reader.lines() {
            let line = line_result.ok()?;
            match line.as_str() {
                "#include <...> search starts here:" => {in_list = true;},
                "End of search list." => {in_list = false;},
                _ => {
                    if in_list {
                        paths.push(line);
                    }
                }
            }
        }
        Some(paths)
    }

    pub fn add_include_path<P: AsRef<Path>>(&mut self, path: P) {
        self.include_paths.push(path.as_ref().to_path_buf())
    }

    pub fn get_include_flags(&self) -> Option<Vec<String>> {
        let mut other_include_flags : Vec<String> = self.include_paths.iter()
            .map(|path| path.to_str())
            .flatten()
            .map(|path| format!("-I{}", path.trim())).collect();

        let mut include_flags : Vec<String> = self.get_include_paths()?.iter()
            .map(|path| format!("-I{}", path.trim())).collect();
        include_flags.append(&mut other_include_flags);
        Some(include_flags)
    }

    pub fn get_compilation_cmd_line<P: AsRef<Path>>(&self, filename: P) -> Option<String> {
        let include_flags = self.get_include_flags()?;
        Some(format!("{} {} -c {}", self.compiler_bin, include_flags.join(" "), filename.as_ref().to_str()?))
    }

    pub fn get_binary_build_cmd<P: AsRef<Path>>(&self, filenames: &[P], output_file: P)  -> Option<Command> {
        let mut cmd = Command::new(self.compiler_bin);
        let source_files : Vec<&str> = filenames.iter().map(|path| path.as_ref().to_str())
            .flatten()
            .collect();
        let include_flags = self.get_include_flags()?;
        cmd.args(include_flags.as_slice());
        cmd.args(source_files.as_slice());
        cmd.arg("-o");
        cmd.arg(output_file.as_ref());
        Some(cmd)

    }
}


// TODO: Unify the tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    #[test]
    fn gets_include_paths() -> Result<(), Box<dyn Error>> {
        let manager = CFlagManager::new("clang++");
        println!("{:?}", manager.get_include_paths().ok_or("Could not get include paths")?);
        Ok(())
    }

    #[test]
    fn gets_include_flags() -> Result<(), Box<dyn Error>> {
        let manager = CFlagManager::new("clang++");
        println!("{:?}", manager.get_include_flags().ok_or("Could not get include flags")?);
        Ok(())
    }

    #[test]
    fn gets_compilation_cmd() -> Result<(), Box<dyn Error>> {
        let manager = CFlagManager::new("clang++");
        println!("{:?}", manager.get_compilation_cmd_line("test.cpp")
            .ok_or("Could not get compilation cmd")?);
        Ok(())
    }
}