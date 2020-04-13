use crate::frontend::EncoderSource;
use std::path::{Path, PathBuf};

pub struct CEGISConfig {
    pub sketch_bin: PathBuf,
    pub sketch_home: PathBuf,
    pub impl_file: PathBuf,
    pub lib_func_name: String,
    pub n_f_args: usize,
    pub n_inputs: usize,
    pub init_n_unknowns: usize,
    pub n_holes: usize,
    pub hole_offset: usize,
    pub pure_function: bool,
    pub cand_encoder_src: EncoderSource,
    pub c_e_encoder_src: EncoderSource,
    pub generation_encoder_src: EncoderSource,
    pub c_e_names: Vec<String>
}

impl CEGISConfig {
    pub fn new<P: AsRef<Path>, S: AsRef<str>>(
            sketch_bin: P, sketch_home: P, impl_file: P, lib_func_name: S,
            n_f_args: usize, n_inputs: usize, init_n_unknowns: usize,
            n_holes: usize, hole_offset: usize, pure_function: bool,
            cand: P, c_e: P, generation: P, c_e_names: &[&str]) -> Self {
        CEGISConfig {
            sketch_bin: sketch_bin.as_ref().to_path_buf(),
            sketch_home: sketch_home.as_ref().to_path_buf(),
            impl_file: impl_file.as_ref().to_path_buf(),
            lib_func_name: lib_func_name.as_ref().to_string(),
            n_f_args: n_f_args,
            n_inputs: n_inputs,
            init_n_unknowns: init_n_unknowns,
            n_holes: n_holes,
            hole_offset: hole_offset,
            pure_function: pure_function,
            cand_encoder_src: EncoderSource::LoadFromFile(cand.as_ref().to_path_buf()),
            c_e_encoder_src: EncoderSource::LoadFromFile(c_e.as_ref().to_path_buf()),
            generation_encoder_src: EncoderSource::LoadFromFile(generation.as_ref().to_path_buf()),
            c_e_names: c_e_names.iter().map(|s| s.to_string()).collect()
        }
    }
}