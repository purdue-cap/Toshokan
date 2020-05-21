use super::EncoderSource;
use crate::cegis::CEGISConfig;
use std::path::Path;

pub struct RewriteController<'itp> {
    rewrite_cand_encoder_enabled: bool,
    rewrite_c_e_encoder_enabled: bool,
    rewrite_generation_encoder_enabled: bool,
    input_tmp_path: Option<&'itp Path> 
}

impl<'itp> RewriteController<'itp> {
    pub fn new(config: &CEGISConfig) -> Self {
        RewriteController{
            rewrite_cand_encoder_enabled:
                if let EncoderSource::Rewrite = config.get_params().cand_encoder_src {true} else {false},
            rewrite_c_e_encoder_enabled:
                if let EncoderSource::Rewrite = config.get_params().c_e_encoder_src {true} else {false},
            rewrite_generation_encoder_enabled:
                if let EncoderSource::Rewrite = config.get_params().generation_encoder_src {true} else {false},
            input_tmp_path: None
        }
    }

    pub fn update_with_config(&mut self, config: &'itp CEGISConfig) {
        self.rewrite_cand_encoder_enabled = 
            if let EncoderSource::Rewrite = config.get_params().cand_encoder_src {true} else {false};
        self.rewrite_c_e_encoder_enabled = 
            if let EncoderSource::Rewrite = config.get_params().c_e_encoder_src {true} else {false};
        self.rewrite_generation_encoder_enabled = 
            if let EncoderSource::Rewrite = config.get_params().generation_encoder_src {true} else {false};
        self.input_tmp_path = config.get_input_tmp_path();
    }

    pub fn enable_rewrite_cand_encoder(&self) -> bool {
        self.rewrite_cand_encoder_enabled
    }
    pub fn enable_rewrite_c_e_encoder(&self) -> bool {
        self.rewrite_c_e_encoder_enabled
    }
    pub fn enable_rewrite_generation_encoder(&self) -> bool {
        self.rewrite_generation_encoder_enabled
    }

    pub fn get_input_tmp_path(&self) -> Option<&Path> {self.input_tmp_path.clone()}
}