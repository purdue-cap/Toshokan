use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::os::raw::{c_int, c_char};
use std::path::Path;
use log::debug;

extern {
    fn build_tracer(func_names: *const *const c_char,
            func_names_len: c_int,
            input_file: *const c_char,
            output_file: *const c_char) -> c_int;
}

pub fn build_tracer_to_file<P: AsRef<Path>>(
            func_names: Vec<String>, input_file: P, output_file: P) -> Result<(), c_int> {
    let vec_c_string_func_names : Vec<CString> = func_names.iter().map(|n| CString::new(n.as_str())).collect::<Result<_,_>>().or(Err(-128))?;
    let vec_c_func_names : Vec<*const c_char> =  vec_c_string_func_names.iter().map(|c_string| c_string.as_ptr()).collect();
   
    let c_func_names_len = func_names.len() as i32;
    let c_input_file = CString::new(input_file.as_ref().as_os_str().as_bytes()).or(Err(-128))?;
    let c_output_file = CString::new(output_file.as_ref().as_os_str().as_bytes()).or(Err(-128))?;
    debug!(target:"LibraryTracer", "Calling CPP Function with args: ({:?}, {:?}, {:?}, {:?})", func_names, c_func_names_len, c_input_file, c_output_file); 
    let rtn_val = unsafe { build_tracer(
        vec_c_func_names.as_slice().as_ptr(),
        c_func_names_len,
        c_input_file.as_ptr(),
        c_output_file.as_ptr())
    };
    debug!(target:"LibraryTracer", "CPP Function rtn code: {}", rtn_val);
    if rtn_val == 0 {
        Ok(())
    } else {
        Err(rtn_val)
    }

}

pub static COMPILATION_DB_FILE_NAME: &'static str = "compile_commands.json";

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::{Read, Write};

    #[test]
    fn builds_simple_tracer() -> Result<(), Box<dyn Error>> {
        let tmp_dir = tempdir()?;
        let input_file_path = tmp_dir.path().join("input.cpp");
        let mut input_file = File::create(input_file_path.as_path())?;
        write!(input_file, "{}", r#"
void sqrt(int i, int& _out) {
    // Put your impl here
    _out = 0;
}
        "#)?;
        let db_file_path = tmp_dir.path().join(COMPILATION_DB_FILE_NAME);
        let mut db_file = File::create(db_file_path.as_path())?;
        write!(db_file, r#"
        [
            {{
                "directory": "{}",
                "command": "/usr/bin/clang++ -c input.cpp",
                "file": "input.cpp"
            }}
        ]
        "#,tmp_dir.path().to_str().ok_or("Path to str conversion failed")?)?;
        let output_file_path = tmp_dir.path().join("output.cpp");

        build_tracer_to_file(vec!["sqrt".to_string()], input_file_path.as_path(), output_file_path.as_path())
            .or(Err("Main function failed"))?;
        
        let mut output_file = File::open(output_file_path.as_path())?;
        let mut output = String::new();
        output_file.read_to_string(&mut output)?;
        assert_eq!(output, r#"int sqrt_impl(int);

void sqrt(int i, int& _out) {
  _out = sqrt_impl(i);
  fprintf(stderr, "sqrt(%d) = %d\n", i, _out);
}
        "#);

        Ok(())
    }

}