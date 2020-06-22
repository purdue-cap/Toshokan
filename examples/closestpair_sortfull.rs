extern crate libpartlibspec;
use libpartlibspec::cegis::{CEGISConfig, CEGISLoop, VerifyPointsConfig, ExcludedHole};
use std::path::PathBuf;
use std::fs::File;
use simplelog::{TermLogger, LevelFilter, Config, TerminalMode};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log_level = LevelFilter::Debug;
    if let Ok(_) = std::env::var("TRACE") {
        log_level = LevelFilter::Trace;
    }
    TermLogger::init(log_level, Config::default(), TerminalMode::Mixed)?;
    let base_data_dir = PathBuf::from(file!()).parent().ok_or("Get parent failed")?.join("data/closestpair_sortfull");
    let verification = base_data_dir.join("verificationMain.sk");
    let synthesis = base_data_dir.join("synthesisMain.sk");
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
        "sorti_proxy",
        "main",
        3,
        9,
        VerifyPointsConfig::NoSpec,
        10,
        vec![
            ExcludedHole::Name("H__0".to_string()),
            ExcludedHole::Name("H__1".to_string()),
            ExcludedHole::Name("H__2".to_string())
        ].into_iter(),
        true,
        true,
        log_level == LevelFilter::Trace,
        synthesis.as_path(),
        verification.as_path(),
        &["start_0_9_9_0", "start_1_a_a_0","start_2_b_b_0",
        "finish_0_c_c_0", "finish_1_d_d_0","finish_2_e_e_0",
        "sel_0_f_f_0", "sel_1_10_10_0","sel_2_11_11_0"], None);
    let mut main_loop = CEGISLoop::new(config);

    println!("{}", main_loop.run_loop()?.or(Some("Unsolvable benchmark".to_string())).unwrap());
    let record_file_name = format!("closestpair_sortfull.{}.record.json", SystemTime::now().duration_since(UNIX_EPOCH).expect("Time goes backward").as_secs());
    let mut record_file = File::create(&record_file_name)?;
    main_loop.get_recorder().ok_or("Recorder uninitialized")?.write_json_pretty(&mut record_file)?;
    println!("Record File: {}", record_file_name);
    Ok(())
}