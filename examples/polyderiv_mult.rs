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
    let base_data_dir = PathBuf::from(file!()).parent().ok_or("Get parent failed")?.join("data/polyderiv_mult");
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
        &[("ANONYMOUS::mult_proxy", 40)],
        "main",
        60,
        VerifyPointsConfig::NoSpec,
        30,
        vec![
            ExcludedHole::Name("H__0".to_string()),
            ExcludedHole::Name("H__1".to_string()),
            ExcludedHole::Name("H__2".to_string()),
            ExcludedHole::Name("H__3".to_string()),
            ExcludedHole::Name("H__4".to_string()),
            ExcludedHole::Name("H__5".to_string()),
            ExcludedHole::Name("H__6".to_string()),
            ExcludedHole::Name("H__7".to_string()),
            ExcludedHole::Name("H__8".to_string()),
            ExcludedHole::Name("H__9".to_string()),
            ExcludedHole::Name("H__10".to_string()),
            ExcludedHole::Name("H__11".to_string()),
            ExcludedHole::Name("H__12".to_string()),
            ExcludedHole::Name("H__13".to_string()),
            ExcludedHole::Name("H__14".to_string()),
            ExcludedHole::Name("H__15".to_string()),
            ExcludedHole::Name("H__16".to_string()),
            ExcludedHole::Name("H__17".to_string()),
            ExcludedHole::Name("H__18".to_string()),
            ExcludedHole::Name("H__19".to_string()),
            ExcludedHole::Name("H__20".to_string()),
            ExcludedHole::Name("H__21".to_string()),
            ExcludedHole::Name("H__22".to_string()),
            ExcludedHole::Name("H__23".to_string()),
            ExcludedHole::Name("H__24".to_string()),
            ExcludedHole::Name("H__25".to_string()),
            ExcludedHole::Name("H__26".to_string()),
            ExcludedHole::Name("H__27".to_string()),
            ExcludedHole::Name("H__28".to_string()),
            ExcludedHole::Name("H__29".to_string()),
            ExcludedHole::Name("H__30".to_string()),
            ExcludedHole::Name("H__31".to_string()),
            ExcludedHole::Name("H__32".to_string()),
            ExcludedHole::Name("H__33".to_string()),
            ExcludedHole::Name("H__34".to_string()),
            ExcludedHole::Name("H__35".to_string()),
            ExcludedHole::Name("H__36".to_string()),
            ExcludedHole::Name("H__37".to_string()),
            ExcludedHole::Name("H__38".to_string()),
            ExcludedHole::Name("H__39".to_string()),
        ].into_iter(),
        true,
        true,
        log_level == LevelFilter::Trace,
        synthesis.as_path(),
        verification.as_path(),
        &[  "p_0_0_3c_3c_0",
            "p_0_1_3f_3f_0",
            "p_0_2_42_42_0",
            "p_0_3_45_45_0",
            "p_0_4_48_48_0",
            "p_0_5_4b_4b_0",
            "p_0_6_4e_4e_0",
            "p_0_7_51_51_0",
            "p_0_8_54_54_0",
            "p_0_9_57_57_0",
            "p_0_10_5a_5a_0",
            "p_0_11_5d_5d_0",
            "p_0_12_60_60_0",
            "p_0_13_63_63_0",
            "p_0_14_66_66_0",
            "p_0_15_69_69_0",
            "p_0_16_6c_6c_0",
            "p_0_17_6f_6f_0",
            "p_0_18_72_72_0",
            "p_0_19_75_75_0",
            "p_1_0_3d_3d_0",
            "p_1_1_40_40_0",
            "p_1_2_43_43_0",
            "p_1_3_46_46_0",
            "p_1_4_49_49_0",
            "p_1_5_4c_4c_0",
            "p_1_6_4f_4f_0",
            "p_1_7_52_52_0",
            "p_1_8_55_55_0",
            "p_1_9_58_58_0",
            "p_1_10_5b_5b_0",
            "p_1_11_5e_5e_0",
            "p_1_12_61_61_0",
            "p_1_13_64_64_0",
            "p_1_14_67_67_0",
            "p_1_15_6a_6a_0",
            "p_1_16_6d_6d_0",
            "p_1_17_70_70_0",
            "p_1_18_73_73_0",
            "p_1_19_76_76_0",
            "p_2_0_3e_3e_0",
            "p_2_1_41_41_0",
            "p_2_2_44_44_0",
            "p_2_3_47_47_0",
            "p_2_4_4a_4a_0",
            "p_2_5_4d_4d_0",
            "p_2_6_50_50_0",
            "p_2_7_53_53_0",
            "p_2_8_56_56_0",
            "p_2_9_59_59_0",
            "p_2_10_5c_5c_0",
            "p_2_11_5f_5f_0",
            "p_2_12_62_62_0",
            "p_2_13_65_65_0",
            "p_2_14_68_68_0",
            "p_2_15_6b_6b_0",
            "p_2_16_6e_6e_0",
            "p_2_17_71_71_0",
            "p_2_18_74_74_0",
            "p_2_19_77_77_0"
        ], Some(1.0));
    let mut main_loop = CEGISLoop::new(config);

    println!("{}", main_loop.run_loop()?.or(Some("Unsolvable benchmark".to_string())).unwrap());
    let (mut record_file, record_file_path) = Builder::new().prefix("polyderiv_mult.").suffix(".record.json").tempfile_in(".")?.keep()?;
    main_loop.get_recorder().ok_or("Recorder uninitialized")?.write_json_pretty(&mut record_file)?;
    println!("Record File: {}", record_file_path.file_name().ok_or("No record file name")?.to_str().ok_or("Record file name decode failed")?);
    Ok(())
}