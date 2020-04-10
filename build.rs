use std::process::Command;

fn main() {
    let llvm_config_output = Command::new("llvm-config")
        .arg("--libs").output()
        .expect("llvm-config call failed")
        .stdout;
    let llvm_flags : Vec<&str>= std::str::from_utf8(&llvm_config_output)
        .expect("llvm-config output decoding failed")
        .trim().split(" ").collect();
    let mut build_tracer = cc::Build::new();
    build_tracer.cpp(true).file("src/backend/cpp/build_tracer.cpp").compile("build_tracer");
    let mut lib_list = vec![
        "clangAST",
        "clangASTMatchers",
        "clangAnalysis",
        "clangBasic",
        "clangDriver",
        "clangEdit",
        "clangFrontend",
        "clangFrontendTool",
        "clangLex",
        "clangParse",
        "clangSema",
        "clangEdit",
        "clangRewrite",
        "clangRewriteFrontend",
        "clangStaticAnalyzerFrontend",
        "clangStaticAnalyzerCheckers",
        "clangStaticAnalyzerCore",
        "clangCrossTU",
        "clangIndex",
        "clangSerialization",
        "clangToolingCore",
        "clangTooling",
        "clangFormat"];
    for f in llvm_flags {
        lib_list.push(&f[2..]);
    }
    for lib in lib_list {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }
}