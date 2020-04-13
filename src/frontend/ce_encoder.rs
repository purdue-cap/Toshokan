use handlebars::Handlebars;
use std::cell::RefCell;
use super::Encoder;

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

impl<'h, 'r> Encoder<'r> for CEEncoder<'h, 'r> {
    fn name(&self) -> &'static str { self.name }
    fn handlebars(&self) -> &RefCell<Handlebars<'r>> { self.handlebars }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cegis::CEGISState;
    use std::error::Error;
    
    #[test]
    fn renders_c_e_s() -> Result<(), Box<dyn Error>> {
        let mut state = CEGISState::new(1, 1, 10, 5, true);
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