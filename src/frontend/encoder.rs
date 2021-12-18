use handlebars::Handlebars;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::fs::File;
use std::cell::RefCell;

use super::{EncodeError, RewriteController};
use serde::Serialize;

pub enum EncoderSource {
    LoadFromFile(PathBuf),
    LoadFromStr(String),
    Rewrite
}

test_fixture!(EncoderSource, dummy, EncoderSource::LoadFromStr("".into()));

impl From<PathBuf> for EncoderSource {
    fn from(p: PathBuf) -> Self {EncoderSource::LoadFromFile(p)}
}

impl From<String> for EncoderSource {
    fn from(s: String) -> Self {EncoderSource::LoadFromStr(s)}
}

pub trait Encoder {
    fn setup_rewrite(&mut self, controller: &RewriteController) -> Result<(), EncodeError>;
    fn rewrite_template_to_str(&self) -> Result<String, EncodeError>;
    fn rewrite_template_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<(), EncodeError> {
        write!(File::create(file_path)?, "{}", self.rewrite_template_to_str()?)?;
        Ok(())
    }

    fn load(&mut self, src: &EncoderSource) -> Result<(), EncodeError> {
        match src {
            EncoderSource::LoadFromFile(p) => self.load_file(p.as_path()),
            EncoderSource::LoadFromStr(s) => self.load_str(s.as_str()),
            EncoderSource::Rewrite => self.load_from_rewrite(),
        }
    }
    fn load_str<S: AsRef<str>>(&mut self, template: S) -> Result<(), EncodeError>;
    fn load_file<P: AsRef<Path>>(&mut self, template_file: P) -> Result<(), EncodeError> {
        self.load_str(std::fs::read_to_string(template_file)?)
    }
    fn load_from_rewrite(&mut self) -> Result<(), EncodeError> {
        self.load_str(self.rewrite_template_to_str()?.as_str())
    }
}

pub trait Renderer<Data>
    where Data: Serialize {
    fn render(&self, params: &Data) -> Result<String, EncodeError>;
    fn render_to_write<W: Write>(&self, params: &Data, mut writer: W) -> Result<(), EncodeError> {
        write!(writer, "{}", self.render(params)?)?;
        Ok(())
    }
    fn render_to_file<P: AsRef<Path>>(&self, params: &Data, file_path: P) -> Result<(), EncodeError> {
        self.render_to_write(params, File::create(file_path)?)
    }
}

pub trait HandlebarsEncoder<'r> {
    fn name(&self) -> &'static str;
    fn handlebars(&self) -> &RefCell<Handlebars<'r>>;

    fn setup_rewrite(&mut self, controller: &RewriteController) -> Result<(), EncodeError>;
    fn rewrite_template_to_str(&self) -> Result<String, EncodeError>;
    fn rewrite_template_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<(), EncodeError> {
        write!(File::create(file_path)?, "{}", self.rewrite_template_to_str()?)?;
        Ok(())
    }
}

impl<'r, T> Encoder for T
    where T: HandlebarsEncoder<'r> {

    fn setup_rewrite(&mut self, controller: &RewriteController) -> Result<(), EncodeError> {
        HandlebarsEncoder::setup_rewrite(self, controller)
    }
    fn rewrite_template_to_str(&self) -> Result<String, EncodeError> {
        HandlebarsEncoder::rewrite_template_to_str(self)
    }
    fn rewrite_template_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<(), EncodeError> {
        HandlebarsEncoder::rewrite_template_to_file(self, file_path)
    }

    fn load_str<S: AsRef<str>>(&mut self, template: S) -> Result<(), EncodeError>{
        Ok(self.handlebars().try_borrow_mut()?
            .register_template_string(self.name(), template)?)
    }

    fn load_file<P: AsRef<Path>>(&mut self, template_file: P) -> Result<(), EncodeError>{
        Ok(self.handlebars().try_borrow_mut()?
            .register_template_file(self.name(), template_file)?)
    }
}

impl<'r, T, Data> Renderer<Data> for T  
    where T: HandlebarsEncoder<'r>, Data: Serialize{
    fn render(&self, params: &Data) -> Result<String, EncodeError> {
        Ok(self.handlebars().try_borrow()?
            .render(self.name(), params)?)
    }

    fn render_to_write<W: Write>(&self, params: &Data, writer: W) -> Result<(), EncodeError> {
        Ok(self.handlebars().try_borrow()?
            .render_to_write(self.name(), params, writer)?)
    }

}