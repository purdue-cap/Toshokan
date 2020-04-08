use crate::cegis::{CEGISState, CEGISStateParams};
use handlebars::Handlebars;
use std::cell::RefCell;
use std::path::Path;
use std::io::Write;

use super::EncodeError;

pub struct CandEncoder<'h, 'r> {
    handlebars: &'h RefCell<Handlebars<'r>>,
    name: &'static str,
}

impl<'h, 'r> CandEncoder<'h, 'r> {
    pub fn new(hb: &'h RefCell<Handlebars<'r>>) -> Self{
        CandEncoder {
            handlebars: hb,
            name: "c-e-encoder",
        }
    }

    pub fn new_with_name(hb: &'h RefCell<Handlebars<'r>>, name: &'static str) -> Self{
        CandEncoder {
            handlebars: hb,
            name: name,
        }
    }

    pub fn load_str<S: AsRef<str>>(&mut self, template: S) -> Result<(), EncodeError>{
        Ok(self.handlebars.try_borrow_mut()?
            .register_template_string(self.name, template)?)
    }

    pub fn load_file<P: AsRef<Path>>(&mut self, template_file: P) -> Result<(), EncodeError>{
        Ok(self.handlebars.try_borrow_mut()?
            .register_template_file(self.name, template_file)?)
    }

    pub fn render(&self, state: &CEGISState) -> Result<String, EncodeError> {
        Ok(self.handlebars.try_borrow()?
            .render(self.name, state.get_params())?)
    }

    pub fn render_params(&self, params: &CEGISStateParams) -> Result<String, EncodeError> {
        Ok(self.handlebars.try_borrow()?
            .render(self.name, params)?)
    }

    pub fn render_to_write<W: Write>(&self, state:&CEGISState, writer: W) -> Result<(), EncodeError> {
        Ok(self.handlebars.try_borrow()?
            .render_to_write(self.name, state.get_params(), writer)?)
    }

    pub fn render_params_to_write<W: Write>(&self, params: &CEGISStateParams, writer: W) -> Result<(), EncodeError> {
        Ok(self.handlebars.try_borrow()?
            .render_to_write(self.name, params, writer)?)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    
    #[test]
    fn renders_holes() -> Result<(), Box<dyn Error>> {
        let mut state = CEGISState::new(1, 1, 10, 5);
        let handlebars = RefCell::new(Handlebars::new());
        let mut encoder = CandEncoder::new(&handlebars);
        encoder.load_str("holes_2 = {{holes.[2]}}")?;
        state.update_hole(2, 3).ok_or("Hole update failed.")?;
        assert_eq!(encoder.render(&state)?, "holes_2 = 3");
        Ok(())
    }

}
