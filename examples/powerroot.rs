extern crate libpartlibspec;
use libpartlibspec::cegis::{CEGISConfig, CEGISLoop, VerifyPointsConfig};
use std::path::PathBuf;
use std::fs::File;
use simplelog::{TermLogger, LevelFilter, Config, TerminalMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log_level = LevelFilter::Debug;
    if let Ok(_) = std::env::var("TRACE") {
        log_level = LevelFilter::Trace;
    }
    TermLogger::init(log_level, Config::default(), TerminalMode::Mixed)?;
    let base_data_dir = PathBuf::from(file!()).parent().ok_or("Get parent failed")?.join("data/powerroot");
    let synthesis = base_data_dir.join("synthesisMain.sk");
    let verification = base_data_dir.join("verificationMain.sk");
    let impl_file = base_data_dir.join("impl.cpp");
    let mut sketch_fe_bin = PathBuf::from("sketchsynth");
    if let Ok(env_path) = std::env::var("SKETCH_FE") {
        sketch_fe_bin = PathBuf::from(env_path);
    }
    let mut sketch_be_bin = PathBuf::from("/usr/share/sketchsynth/bin/cegis");
    if let Ok(env_path) = std::env::var("SKETCH_BE") {
        sketch_be_bin = PathBuf::from(env_path);
    }
    let mut sketch_home = None;
    if let Ok(env_path) = std::env::var("SKETCH_HOME") {
        sketch_home = Some(PathBuf::from(env_path));
    }
    let config = CEGISConfig::new(
        sketch_fe_bin.as_path(),
        sketch_be_bin.as_path(),
        sketch_home.as_ref().map(|p| p.as_path()),
        impl_file.as_path(),
        "sqrt",
        "main",
        1,
        1,
        VerifyPointsConfig::NoSpec,
        10,
        1,
        true,
        true,
        synthesis.as_path(),
        verification.as_path(),
        &["x_1_1_0"]);
    let mut main_loop = CEGISLoop::new(config);

    println!("{}", main_loop.run_loop()?.or(Some("Unsolvable benchmark".to_string())).unwrap());
    let mut record_file = File::create("powerroot.record.json")?;
    main_loop.get_recorder().ok_or("Recorder uninitialized")?.write_json_pretty(&mut record_file)?;
    Ok(())
}