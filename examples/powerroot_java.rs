use libpartlibspec::cegis::java::{CEGISConfig, CEGISLoop, CEGISConfigParamsBuilder};
use libpartlibspec::frontend::java::{JBMCConfigBuilder, JSketchConfigBuilder};
use std::path::PathBuf;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_data_dir = std::env::current_dir()?.join(file!()).parent().ok_or("Get parent failed")?.join("data/powerroot_java");
    let synthesis_template = base_data_dir.join("Synthesis.java");
    let lib_src_file = base_data_dir.join("Library.java");
    let verif_src_file = base_data_dir.join("Main.java");
    let api_src_file = base_data_dir.join("PowerRoot.java");
    let synthesis_common_file = base_data_dir.join("Synthesis_common.java");
    let jsketch_dir = PathBuf::from(std::env::var("JSKETCH_DIR")?);
    let mut jbmc_bin = PathBuf::from("jbmc");
    if let Ok(env_path) = std::env::var("JBMC_BIN") {
        jbmc_bin = PathBuf::from(env_path);
    }
    let mut javac_bin = PathBuf::from("javac");
    if let Ok(env_path) = std::env::var("JAVAC_BIN") {
        javac_bin = PathBuf::from(env_path);
    } 
    let jbmc_config = JBMCConfigBuilder::default()
        .bin_path(jbmc_bin)
        .unwind(8)
        .primitive_input_bound(Some((0, 15)))
        .build()?;
    let jsketch_config = JSketchConfigBuilder::default()
        .dir_path(jsketch_dir)
        .build()?;
    let config_params = CEGISConfigParamsBuilder::default()
        .jbmc_config(jbmc_config)
        .javac_bin(javac_bin)
        .jsketch_config(jsketch_config)
        .lib_func("Library.sqrt(int)".into())
        .c_e_encoder_src(synthesis_template)
        .verif_src_file(lib_src_file.into())
        .verif_src_file(verif_src_file.into())
        .verif_src_file(api_src_file.into())
        .verif_entrance("Main.main")
        .synth_file(synthesis_common_file)
        .output_class("PowerRoot".into())
        .n_inputs(1 as usize)
        .output_dir("/usr/tmp")
        .keep_tmp(true)
        .build()?;
    let config = CEGISConfig::new(config_params);
    let mut cegis_loop = CEGISLoop::new(config);
    if let Err(err) = cegis_loop.run_loop() {
        println!("WorkDir: {:?}", cegis_loop.get_work_dir());
        return Err(err);
    }
    Ok(())
}