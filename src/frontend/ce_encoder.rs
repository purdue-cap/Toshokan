use handlebars::Handlebars;
use std::cell::RefCell;
use std::path::Path;
use super::{HandlebarsEncoder, EncodeError, RewriteController};

pub struct CEEncoder<'h, 'r> {
    handlebars: &'h RefCell<Handlebars<'r>>,
    name: &'static str,
}

impl<'h, 'r> CEEncoder<'h, 'r> {
    pub fn new(hb: &'h RefCell<Handlebars<'r>>) -> Self{
        CEEncoder {
            handlebars: hb,
            name: "c-e-encoder",
        }
    }

    pub fn new_with_name(hb: &'h RefCell<Handlebars<'r>>, name: &'static str) -> Self{
        CEEncoder {
            handlebars: hb,
            name: name,
        }
    }
}

impl<'h, 'r> HandlebarsEncoder<'r> for CEEncoder<'h, 'r> {
    fn name(&self) -> &'static str { self.name }
    fn handlebars(&self) -> &RefCell<Handlebars<'r>> { self.handlebars }

    fn setup_rewrite(&mut self, _controller: &RewriteController) -> Result<(), EncodeError> { Ok(()) }
    fn rewrite_template_to_str(&self) -> Result<String, EncodeError> {
        unimplemented!("Rewrite for CEEncoder not implemented yet.");
    }
    fn rewrite_template_to_file<P: AsRef<Path>>(&self, _file_name: P) -> Result<(), EncodeError> {
        unimplemented!("Rewrite for CEEncoder not implemented yet.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cegis::CEGISState;
    use crate::frontend::Encoder;
    use std::error::Error;
    
    #[test]
    fn renders_c_e_s() -> Result<(), Box<dyn Error>> {
        let mut state = CEGISState::new([("func".to_string(), 1 as usize)].iter().cloned().collect(), 1, 10, true);
        let handlebars = RefCell::new(Handlebars::new());
        let mut encoder = CEEncoder::new(&handlebars);
        encoder.load_str("c_e_2 = {{c_e_s.[0].[2]}}")?;
        for i in 1..5 {
            state.add_c_e(vec![i]).ok_or("C.E. addition failed")?;
        }
        assert_eq!(encoder.render(&state)?, "c_e_2 = 3");
        Ok(())
    }

}