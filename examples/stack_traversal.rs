use libpartlibspec::cegis::{CEGISConfig, CEGISLoop, VerifyPointsConfig, ExcludedHole, FuncConfig};
use std::path::PathBuf;
use simplelog::{SimpleLogger, LevelFilter, Config};
use tempfile::Builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log_level = LevelFilter::Debug;
    if let Ok(_) = std::env::var("TRACE") {
        log_level = LevelFilter::Trace;
    }
    SimpleLogger::init(log_level, Config::default())?;
    let base_data_dir = PathBuf::from(file!()).parent().ok_or("Get parent failed")?.join("data/stack_traversal");
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
    let config = CEGISConfig::new_full_config(
        sketch_fe_bin.as_path(),
        sketch_be_bin.as_path(),
        sketch_home.as_ref().map(|p| p.as_path()),
        impl_file.as_path(),
        &[("ANONYMOUS::s_push_real", FuncConfig::NonPure{args:2, state_arg_idx: 0}),
        ("ANONYMOUS::s_pop_real", FuncConfig::NonPure{args:1, state_arg_idx: 0}),
        ("ANONYMOUS::s_new_real", FuncConfig::Init{args: 0}),
        ("ANONYMOUS::s_empty_real", FuncConfig::StateQuery{args:1, state_arg_idx: 0}),
        ],
        &[],
        "main",
        3,
        VerifyPointsConfig::NoSpec,
        10,
        10,
        vec![
            ExcludedHole::Position(16, -1),
            ExcludedHole::Position(17, -1),
            ExcludedHole::Position(18, -1),
            ExcludedHole::Position(23, -1),
            ExcludedHole::Position(24, -1),
            ExcludedHole::Position(25, -1),
            ExcludedHole::Position(30, -1),
            ExcludedHole::Position(31, -1),
            ExcludedHole::Position(32, -1),
        ].into_iter(),
        true,
        log_level == LevelFilter::Trace,
        synthesis.as_path(),
        verification.as_path(),
        &[
            "input_3_b_0"
        ], Some(1.0));
    let mut main_loop = CEGISLoop::new(config);

    println!("{}", main_loop.run_loop()?.or(Some("Unsolvable benchmark".to_string())).unwrap());
    let (mut record_file, record_file_path) = Builder::new().prefix("stack_traversal.").suffix(".record.json").tempfile_in(".")?.keep()?;
    main_loop.get_recorder().ok_or("Recorder uninitialized")?.write_json_pretty(&mut record_file)?;
    println!("Record File: {}", record_file_path.file_name().ok_or("No record file name")?.to_str().ok_or("Record file name decode failed")?);
    Ok(())
}