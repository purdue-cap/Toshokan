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
        &[("ANONYMOUS::sort_proxy_real", 5)],
        "main",
        5,
        VerifyPointsConfig::NoSpec,
        10,
        vec![
            ExcludedHole::Position(7, -1),
            ExcludedHole::Position(8, -1),
            ExcludedHole::Position(9, -1),
            ExcludedHole::Position(10, -1),
            ExcludedHole::Position(11, -1),
            ExcludedHole::Position(12, -1),
            ExcludedHole::Position(13, -1),
            ExcludedHole::Position(14, -1),
            ExcludedHole::Position(15, -1),
            ExcludedHole::Position(16, -1),
        ].into_iter(),
        true,
        true,
        log_level == LevelFilter::Trace,
        synthesis.as_path(),
        verification.as_path(),
        &["arr_0_5_5_0","arr_1_6_6_0","arr_2_7_7_0","arr_3_8_8_0","arr_4_9_9_0",], Some(5.0));
    let mut main_loop = CEGISLoop::new(config);

    println!("{}", main_loop.run_loop()?.or(Some("Unsolvable benchmark".to_string())).unwrap());
    let (mut record_file, record_file_path) = Builder::new().prefix("closestpair_sortfull.").suffix(".record.json").tempfile_in(".")?.keep()?;
    main_loop.get_recorder().ok_or("Recorder uninitialized")?.write_json_pretty(&mut record_file)?;
    println!("Record File: {}", record_file_path.file_name().ok_or("No record file name")?.to_str().ok_or("Record file name decode failed")?);
    Ok(())
}