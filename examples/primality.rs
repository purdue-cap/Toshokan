extern crate libpartlibspec;
use libpartlibspec::cegis::{CEGISConfig, CEGISLoop};
use std::path::PathBuf;
use simplelog::{TermLogger, LevelFilter, Config, TerminalMode};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    TermLogger::init(LevelFilter::Debug, Config::default(), TerminalMode::Mixed)?;
    let base_data_dir = PathBuf::from(file!()).parent().ok_or("Get parent failed")?.join("data/primality");
    let verification = base_data_dir.join("verificationMain.sk");
    let synthesis = base_data_dir.join("synthesisMain.sk");
    let generation = base_data_dir.join("generationMain.sk");
    let impl_file = base_data_dir.join("impl.cpp");
    let sketch_bin = PathBuf::from("sketchsynth");
    let sketch_home = PathBuf::from("/usr/share/sketchsynth/runtime/");
    let config = CEGISConfig::new(
        sketch_bin.as_path(),
        sketch_home.as_path(),
        impl_file.as_path(),
        "sqrt",
        1,
        1,
        10,
        7,
        1,
        true,
        verification.as_path(),
        synthesis.as_path(),
        generation.as_path(),
        &["p_3_6_0"]);
    let mut main_loop = CEGISLoop::new(config);

    for v_p in [1, 2, 6, 23, 29, 7, 16, 148].iter() {
        main_loop.get_state_mut().add_verify_point(vec![*v_p]);
    }
    println!("{}", main_loop.run_loop()?.or(Some("Unsolvable benchmark".to_string())).unwrap());
    Ok(())
}