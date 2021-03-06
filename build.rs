#[cfg(any(feature = "libclang", feature = "inline_java_tracer"))]
use std::process::Command;

fn main() {
    #[cfg(feature = "libclang")]
    {
        let llvm_config_output = Command::new("llvm-config")
            .arg("--libs").output()
            .expect("llvm-config call failed")
            .stdout;
        let llvm_flags : Vec<&str>= std::str::from_utf8(&llvm_config_output)
            .expect("llvm-config output decoding failed")
            .trim().split(" ").collect();
        let mut build_tracer = cc::Build::new();
        build_tracer.cpp(true).include("src/backend/cpp/").file("src/backend/cpp/build_tracer.cpp").compile("build_tracer");
        let mut lib_list = vec![ "clang-cpp" ];
        for f in llvm_flags {
            lib_list.push(&f[2..]);
        }
        for lib in lib_list {
            println!("cargo:rustc-link-lib=dylib={}", lib);
        }
    }
    #[cfg(feature = "inline_java_tracer")]
    {
        Command::new("make").current_dir("javaTracer/").output().expect("Make JavaTracer failed");
    }
}