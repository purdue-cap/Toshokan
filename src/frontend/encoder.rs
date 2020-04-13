use handlebars::Handlebars;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::fs::File;
use std::cell::RefCell;

use super::EncodeError;
use crate::cegis::{CEGISState, CEGISStateParams};

pub enum EncoderSource {
    LoadFromFile(PathBuf),
    LoadFromStr(String),
    Rewrite
}

pub trait Encoder<'r> {
    fn name(&self) -> &'static str;
    fn handlebars(&self) -> &RefCell<Handlebars<'r>>;

    fn load(&mut self, src: &EncoderSource) -> Result<(), EncodeError> {
        match src {
            EncoderSource::LoadFromFile(p) => self.load_file(p.as_path()),
            EncoderSource::LoadFromStr(s) => self.load_str(s.as_str()),
            _ => Err(EncodeError::SourceNotSupported)
        }
    }

    fn load_str<S: AsRef<str>>(&mut self, template: S) -> Result<(), EncodeError>{
        Ok(self.handlebars().try_borrow_mut()?
            .register_template_string(self.name(), template)?)
    }

    fn load_file<P: AsRef<Path>>(&mut self, template_file: P) -> Result<(), EncodeError>{
        Ok(self.handlebars().try_borrow_mut()?
            .register_template_file(self.name(), template_file)?)
    }

    fn render(&self, state: &CEGISState) -> Result<String, EncodeError> {
        Ok(self.handlebars().try_borrow()?
            .render(self.name(), state.get_params())?)
    }

    fn render_params(&self, params: &CEGISStateParams) -> Result<String, EncodeError> {
        Ok(self.handlebars().try_borrow()?
            .render(self.name(), params)?)
    }

    fn render_to_write<W: Write>(&self, state:&CEGISState, writer: W) -> Result<(), EncodeError> {
        Ok(self.handlebars().try_borrow()?
            .render_to_write(self.name(), state.get_params(), writer)?)
    }

    fn render_params_to_write<W: Write>(&self, params: &CEGISStateParams, writer: W) -> Result<(), EncodeError> {
        Ok(self.handlebars().try_borrow()?
            .render_to_write(self.name(), params, writer)?)
    }

    fn render_to_file<P: AsRef<Path>>(&self, state:&CEGISState, file_path: P) -> Result<(), EncodeError> {
        let file = File::create(file_path)?;
        self.render_to_write(state, file)
    }

    fn render_params_to_file<P: AsRef<Path>>(&self, params: &CEGISStateParams, file_path: P) -> Result<(), EncodeError> {
        let file = File::create(file_path)?;
        self.render_params_to_write(params, file)
    }
}