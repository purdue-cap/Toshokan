extern crate libpartlibspec;
use libpartlibspec::cegis::{CEGISLoop, ExcludedHole, CEGISConfigBuilder, SketchConfig};
use std::path::PathBuf;
use simplelog::{SimpleLogger, LevelFilter, Config};
use tempfile::Builder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut log_level = LevelFilter::Debug;
    if let Ok(_) = std::env::var("TRACE") {
        log_level = LevelFilter::Trace;
    }
    SimpleLogger::init(log_level, Config::default())?;
    let base_data_dir = PathBuf::from(file!()).parent().ok_or("Get parent failed")?.join("data/closest_power_two_ilog");
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
    let mut sketch_config = SketchConfig {
        bnd_inbits: Some(4),
        bnd_inline_amnt: Some(2),
        bnd_unroll_amnt: Some(16),
        bnd_cbits: Some(3),
        slv_nativeints: true,
        ..Default::default()
    };
    if let Some(n_cpu) = std::env::var("SKETCH_N_CPU").ok().and_then(|s| s.parse::<usize>().ok()) {
        sketch_config.slv_parallel = true;
        sketch_config.slv_p_cpus = Some(n_cpu);
    }
    if let Some(randdegree) = std::env::var("SKETCH_RANDDEGREE").ok().and_then(|s| s.parse::<usize>().ok()) {
        sketch_config.slv_randassign = true;
        sketch_config.slv_randdegree = Some(randdegree);
    }
    let config = CEGISConfigBuilder::new()
        .set_sketch_fe_bin(sketch_fe_bin.as_path())
        .set_sketch_be_bin(sketch_be_bin.as_path())
        .set_sketch_home(sketch_home.as_ref().map(|p| p.as_path()))
        .set_impl_file(impl_file.as_path())
        .set_pure_func_config(
            vec![("ANONYMOUS::log_real", 1)].into_iter()
        )
        .set_harness_func_name("closestTwoPower")
        .set_n_inputs(5)
        .set_init_n_unknowns(10)
        .set_excluded_holes(
            vec![
                ExcludedHole::Position(6, -1),
                ExcludedHole::Position(7, -1)
            ].into_iter())
        .set_enable_record(true)
        .set_keep_tmp(log_level == LevelFilter::Trace)
        .set_c_e_encoder_src_file(synthesis.as_path())
        .set_generation_encoder_src_file(verification.as_path())
        .set_c_e_names(
            vec![
                "a_0_5_5_0","a_1_6_6_0","a_2_7_7_0","a_3_8_8_0","a_4_9_9_0"
            ].into_iter())
        .set_trace_timeout(5.0)
        .set_synthesis_sketch_config(sketch_config)
        .build().ok_or("Config building failure")?;
    let mut main_loop = CEGISLoop::new(config);

    println!("{}", main_loop.run_loop()?.or(Some("Unsolvable benchmark".to_string())).unwrap());
    let (mut record_file, record_file_path) = Builder::new().prefix("closest_power_two_ilog.").suffix(".record.json").tempfile_in(".")?.keep()?;
    main_loop.get_recorder().ok_or("Recorder uninitialized")?.write_json_pretty(&mut record_file)?;
    println!("Record File: {}", record_file_path.file_name().ok_or("No record file name")?.to_str().ok_or("Record file name decode failed")?);
    Ok(())
}