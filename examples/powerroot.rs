extern crate libpartlibspec;
use libpartlibspec::cegis::{CEGISConfig, CEGISLoop, VerifyPointsConfig};
use std::path::PathBuf;
use std::collections::HashSet;
use std::fs::File;
use simplelog::{TermLogger, LevelFilter, Config, TerminalMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log_level = LevelFilter::Debug;
    if let Ok(_) = std::env::var("TRACE") {
        log_level = LevelFilter::Trace;
    }
    TermLogger::init(log_level, Config::default(), TerminalMode::Mixed)?;
    let base_data_dir = PathBuf::from(file!()).parent().ok_or("Get parent failed")?.join("data/powerroot");
    let verification = base_data_dir.join("verificationMain.sk");
    let synthesis = base_data_dir.join("synthesisMain.sk");
    let generation = base_data_dir.join("generationMain.sk");
    let impl_file = base_data_dir.join("impl.cpp");
    let sketch_bin = PathBuf::from("sketchsynth");
    let sketch_home = PathBuf::from("/usr/share/sketchsynth/runtime/");
    let v_p_set : HashSet<_> = (0..32).map(
        |n| vec![n]
    ).collect();
    let config = CEGISConfig::new(
        sketch_bin.as_path(),
        sketch_home.as_path(),
        impl_file.as_path(),
        "sqrt",
        "main",
        1,
        1,
        VerifyPointsConfig::Fixed(v_p_set),
        10,
        5,
        1,
        true,
        true,
        verification.as_path(),
        synthesis.as_path(),
        generation.as_path(),
        &["x_1_1_0"]);
    let mut main_loop = CEGISLoop::new(config);

    println!("{}", main_loop.run_loop()?.or(Some("Unsolvable benchmark".to_string())).unwrap());
    let mut record_file = File::create("powerroot.record.json")?;
    main_loop.get_recorder().ok_or("Recorder uninitialized")?.write_json_pretty(&mut record_file)?;
    Ok(())
}