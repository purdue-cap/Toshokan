use libpartlibspec::cegis::java::{CEGISConfig, CEGISLoop, CEGISConfigParamsBuilder};
use libpartlibspec::frontend::java::{JBMCConfigBuilder, JSketchConfigBuilder};
use std::path::PathBuf;
use tempfile::Builder as TempFileBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_data_dir = std::env::current_dir()?.join(file!()).parent().ok_or("Get parent failed")?.join("data/lcm_java");
    let synthesis_template = base_data_dir.join("Synthesis.java");
    let lib_src_file = base_data_dir.join("Library.java");
    let verif_src_file = base_data_dir.join("Main.java");
    let api_src_file = base_data_dir.join("LCM.java");
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
        .unwind(6)
        .unwind_growth_step(2)
        .unwind_maximum(16)
        .primitive_input_bound(Some((0, 511)))
        .build()?;
    let jsketch_config = JSketchConfigBuilder::default()
        .dir_path(jsketch_dir)
        .inline(None)
        .unroll(16)
        .inbits(9)
        .cbits(6)
        .build()?;
    let mut config_builder = CEGISConfigParamsBuilder::default()
        .jbmc_config(jbmc_config)
        .javac_bin(javac_bin)
        .jsketch_config(jsketch_config)
        .lib_func("Library.lcm(int, int)".into())
        .c_e_encoder_src(synthesis_template)
        .verif_src_file(lib_src_file.into())
        .verif_src_file(verif_src_file.into())
        .verif_src_file(api_src_file.into())
        .verif_entrance("Main.main")
        .synth_file(synthesis_common_file)
        .output_class("LCM".into())
        .n_inputs(5 as usize)
        .output_dir("results/")
        .keep_tmp(true)
        .enable_record(true);
    if let Ok(env_path) = std::env::var("CPROVER_JAR") {
        config_builder = config_builder.verif_classpath(env_path.into());
    }
    let config_params = config_builder.build()?;
    let config = CEGISConfig::new(config_params);
    let mut cegis_loop = CEGISLoop::new(config);
    if let Err(err) = cegis_loop.run_loop() {
        println!("WorkDir: {:?}", cegis_loop.get_work_dir());
        return Err(err);
    }

    let (mut record_file, record_file_path) = TempFileBuilder::new().prefix("lcm_java.").suffix(".record.json").tempfile_in(".")?.keep()?;
    cegis_loop.get_recorder().ok_or("Recorder uninitialized")?.write_json_pretty(&mut record_file)?;
    println!("Record File: {}", record_file_path.file_name().ok_or("No record file name")?.to_str().ok_or("Record file name decode failed")?);
    Ok(())
}