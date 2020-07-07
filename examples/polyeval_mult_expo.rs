extern crate libpartlibspec;
use libpartlibspec::cegis::{CEGISConfig, CEGISLoop, VerifyPointsConfig, ExcludedHole};
use std::path::PathBuf;
use simplelog::{SimpleLogger, LevelFilter, Config};
use tempfile::Builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log_level = LevelFilter::Debug;
    if let Ok(_) = std::env::var("TRACE") {
        log_level = LevelFilter::Trace;
    }
    SimpleLogger::init(log_level, Config::default())?;
    let base_data_dir = PathBuf::from(file!()).parent().ok_or("Get parent failed")?.join("data/polyeval_mult_expo");
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
        &[("ANONYMOUS::mult_proxy_real", 10),
        ("ANONYMOUS::exp_real", 2)],
        "main",
        16,
        VerifyPointsConfig::NoSpec,
        10,
        vec![
            ExcludedHole::Position(12, -1),
            ExcludedHole::Position(13, -1),
            ExcludedHole::Position(14, -1),
            ExcludedHole::Position(15, -1),
            ExcludedHole::Position(16, -1),
            ExcludedHole::Position(18, -1),
            ExcludedHole::Position(19, -1),
            ExcludedHole::Position(20, -1),
            ExcludedHole::Position(21, -1),
            ExcludedHole::Position(22, -1),
            ExcludedHole::Position(24, -1),
            ExcludedHole::Position(25, -1),
            ExcludedHole::Position(26, -1),
            ExcludedHole::Position(27, -1),
            ExcludedHole::Position(28, -1),
            ExcludedHole::Position(29, -1),
            ExcludedHole::Position(30, -1),
            ExcludedHole::Position(31, -1),
            ExcludedHole::Position(32, -1),
            ExcludedHole::Position(33, -1),
            ExcludedHole::Position(73, -1),
            ExcludedHole::Position(74, -1),
            ExcludedHole::Position(75, -1),
            ExcludedHole::Position(76, -1),
            ExcludedHole::Position(77, -1),
            ExcludedHole::Position(78, -1),
            ExcludedHole::Position(79, -1),
        ].into_iter(),
        true,
        true,
        log_level == LevelFilter::Trace,
        synthesis.as_path(),
        verification.as_path(),
        &[
        "p_0_0_10_10_0",
        "p_1_0_11_11_0",
        "p_2_0_12_12_0",
        "p_0_1_13_13_0",
        "p_1_1_14_14_0",
        "p_2_1_15_15_0",
        "p_0_2_16_16_0",
        "p_1_2_17_17_0",
        "p_2_2_18_18_0",
        "p_0_3_19_19_0",
        "p_1_3_1a_1a_0",
        "p_2_3_1b_1b_0",
        "p_0_4_1c_1c_0",
        "p_1_4_1d_1d_0",
        "p_2_4_1e_1e_0",
        "x_1f_1f_0"
        ], None);
    let mut main_loop = CEGISLoop::new(config);

    println!("{}", main_loop.run_loop()?.or(Some("Unsolvable benchmark".to_string())).unwrap());
    let (mut record_file, record_file_path) = Builder::new().prefix("polyeval_mult_expo.").suffix(".record.json").tempfile_in(".")?.keep()?;
    main_loop.get_recorder().ok_or("Recorder uninitialized")?.write_json_pretty(&mut record_file)?;
    println!("Record File: {}", record_file_path.file_name().ok_or("No record file name")?.to_str().ok_or("Record file name decode failed")?);
    Ok(())
}