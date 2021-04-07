extern crate libpartlibspec;
use libpartlibspec::cegis::{CEGISLoop, ExcludedHole, FuncConfig, CEGISConfigBuilder};
use std::path::PathBuf;
use simplelog::{SimpleLogger, LevelFilter, Config};
use tempfile::Builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log_level = LevelFilter::Debug;
    if let Ok(_) = std::env::var("TRACE") {
        log_level = LevelFilter::Trace;
    }
    SimpleLogger::init(log_level, Config::default())?;
    let base_data_dir = PathBuf::from(file!()).parent().ok_or("Get parent failed")?.join("data/stack_sort");
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
    let config = CEGISConfigBuilder::new()
        .set_sketch_fe_bin(sketch_fe_bin.as_path())
        .set_sketch_be_bin(sketch_be_bin.as_path())
        .set_sketch_home(sketch_home.as_ref().map(|p| p.as_path()))
        .set_impl_file(impl_file.as_path())
        .set_func_config(
            vec![("ANONYMOUS::s_push_real", FuncConfig::NonPure{args:2, state_arg_idx: 0}),
            ("ANONYMOUS::s_pop_real", FuncConfig::NonPure{args:1, state_arg_idx: 0}),
            ("ANONYMOUS::s_new_real", FuncConfig::Init{args: 0}),
            ("ANONYMOUS::s_peek_real", FuncConfig::StateQuery{args:1, state_arg_idx: 0}),
            ("ANONYMOUS::s_empty_real", FuncConfig::StateQuery{args:1, state_arg_idx: 0}),
            ].into_iter())
        .set_n_inputs(3)
        .set_init_n_unknowns(10)
        .set_init_hist_cap_padding(10)
        .set_excluded_holes(
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
                ExcludedHole::Position(37, -1),
                ExcludedHole::Position(38, -1),
                ExcludedHole::Position(39, -1)
            ].into_iter())
        .set_enable_record(true)
        .set_keep_tmp(log_level == LevelFilter::Trace)
        .set_c_e_encoder_src_file(synthesis.as_path())
        .set_generation_encoder_src_file(verification.as_path())
        .set_c_e_names(
            vec![
                "i0_6_f_0",
                "i1_7_10_0",
                "i2_8_11_0"
            ].into_iter())
        .set_trace_timeout(1.0)
        .build().ok_or("Config building failure")?;
    let mut main_loop = CEGISLoop::new(config);

    println!("{}", main_loop.run_loop()?.or(Some("Unsolvable benchmark".to_string())).unwrap());
    let (mut record_file, record_file_path) = Builder::new().prefix("stack_sort.").suffix(".record.json").tempfile_in(".")?.keep()?;
    main_loop.get_recorder().ok_or("Recorder uninitialized")?.write_json_pretty(&mut record_file)?;
    println!("Record File: {}", record_file_path.file_name().ok_or("No record file name")?.to_str().ok_or("Record file name decode failed")?);
    Ok(())
}