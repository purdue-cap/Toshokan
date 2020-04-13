use handlebars::Handlebars;
use std::cell::RefCell;
use super::Encoder;

pub struct CandEncoder<'h, 'r> {
    handlebars: &'h RefCell<Handlebars<'r>>,
    name: &'static str,
}

impl<'h, 'r> CandEncoder<'h, 'r> {
    pub fn new(hb: &'h RefCell<Handlebars<'r>>) -> Self{
        CandEncoder {
            handlebars: hb,
            name: "cand-encoder",
        }
    }

    pub fn new_with_name(hb: &'h RefCell<Handlebars<'r>>, name: &'static str) -> Self{
        CandEncoder {
            handlebars: hb,
            name: name,
        }
    }
}

impl<'h, 'r> Encoder<'r> for CandEncoder<'h, 'r> {
    fn name(&self) -> &'static str { self.name }
    fn handlebars(&self) -> &RefCell<Handlebars<'r>> { self.handlebars }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cegis::CEGISState;
    use std::error::Error;
    
    #[test]
    fn renders_holes() -> Result<(), Box<dyn Error>> {
        let mut state = CEGISState::new(1, 1, 10, 5, true);
        let handlebars = RefCell::new(Handlebars::new());
        let mut encoder = CandEncoder::new(&handlebars);
        encoder.load_str("holes_2 = {{holes.[2]}}")?;
        state.update_hole(2, 3).ok_or("Hole update failed.")?;
        assert_eq!(encoder.render(&state)?, "holes_2 = 3");
        Ok(())
    }

}
