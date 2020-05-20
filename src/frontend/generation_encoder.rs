use handlebars::Handlebars;
use std::cell::RefCell;
use super::Encoder;
use super::EncodeError;
use std::path::Path;

pub struct GenerationEncoder<'h, 'r> {
    handlebars: &'h RefCell<Handlebars<'r>>,
    name: &'static str,
}

impl<'h, 'r> GenerationEncoder<'h, 'r> {
    pub fn new(hb: &'h RefCell<Handlebars<'r>>) -> Self{
        GenerationEncoder {
            handlebars: hb,
            name: "generation-encoder",
        }
    }

    pub fn new_with_name(hb: &'h RefCell<Handlebars<'r>>, name: &'static str) -> Self{
        GenerationEncoder {
            handlebars: hb,
            name: name,
        }
    }
}

impl<'h, 'r> Encoder<'r> for GenerationEncoder<'h, 'r> {
    fn name(&self) -> &'static str { self.name }
    fn handlebars(&self) -> &RefCell<Handlebars<'r>> { self.handlebars }

    fn rewrite_template_to_str(&self) -> Result<String, EncodeError> {
        unimplemented!("Rewrite for GenerationEncoder not implemented yet.");
    }
    fn rewrite_template_to_file<P: AsRef<Path>>(&self, _file_name: P) -> Result<(), EncodeError> {
        unimplemented!("Rewrite for GenerationEncoder not implemented yet.");
    }
}

