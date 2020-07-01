use handlebars::Handlebars;
use std::cell::RefCell;
use std::path::Path;
use std::fs;
use std::io::Write;
use std::collections::HashSet;
use regex::Regex;
use super::{Encoder, EncodeError, RewriteController};

pub static HOLE_REGEX: &'static str = r#"(?x)
(?:
    SPVAR \s+\d+\s* \$ .*? \$ \s*|
    MINVAR \s*
)?
<
    \s*
    (?P<name>H__\d+[_\d]*)
    \s*
    (?:
        \s\d+\s*
        (?:\*|\+)?|
        \+|
        \$
    )?
    \s*
>"#;


pub struct CandEncoder<'h, 'r> {
    handlebars: &'h RefCell<Handlebars<'r>>,
    name: &'static str,
    input_tmp: Option<String>
}

impl<'h, 'r> CandEncoder<'h, 'r> {
    pub fn new(hb: &'h RefCell<Handlebars<'r>>) -> Self{
        CandEncoder {
            handlebars: hb,
            name: "cand-encoder",
            input_tmp: None
        }
    }

    pub fn new_with_name(hb: &'h RefCell<Handlebars<'r>>, name: &'static str) -> Self{
        CandEncoder {
            handlebars: hb,
            name: name,
            input_tmp: None
        }
    }

    pub fn load_input_tmp_from_str<S: AsRef<str>>(&mut self, content: S) {
        self.input_tmp = Some(content.as_ref().to_string());
    }

    pub fn load_input_tmp_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), EncodeError> {
        self.input_tmp = Some(fs::read_to_string(path)?);
        Ok(())
    }

    pub fn get_hole_names(&self) -> Option<HashSet<String>> {
        let regex = Regex::new(HOLE_REGEX).expect("Hard coded regex should not fail");
        regex.captures_iter(self.input_tmp.as_ref()?.as_str()).map(|cap|
            cap.name("name").map(|m| m.as_str().to_string())).collect()
    }
}

impl<'h, 'r> Encoder<'r> for CandEncoder<'h, 'r> {
    fn name(&self) -> &'static str { self.name }
    fn handlebars(&self) -> &RefCell<Handlebars<'r>> { self.handlebars }

    fn setup_rewrite(&mut self, controller: &RewriteController) -> Result<(), EncodeError> {
        if controller.enable_rewrite_cand_encoder() {
            self.load_input_tmp_from_file(controller.get_input_tmp_path().ok_or(
                EncodeError::RewriteError("No input tmp file path present in RewriteController")
            )?)?;
        }
        Ok(())
    }
    fn rewrite_template_to_str(&self) -> Result<String, EncodeError> {
        let regex = Regex::new(HOLE_REGEX).expect("Hard coded regex should not fail");
        Ok(regex.replace_all(self.input_tmp.as_ref()
                            .ok_or(EncodeError::RewriteError("No input tmp file loaded"))?
                            .as_str(), "{{holes.${name}}}").into_owned())
    }
    fn rewrite_template_to_file<P: AsRef<Path>>(&self, file_name: P) -> Result<(), EncodeError> {
        let mut output_file = fs::File::create(file_name)?;
        write!(output_file, "{}", self.rewrite_template_to_str()?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cegis::CEGISState;
    use std::error::Error;
    
    #[test]
    fn renders_holes() -> Result<(), Box<dyn Error>> {
        let mut state = CEGISState::new([("func".to_string(), 1 as usize)].iter().cloned().collect(), 1, 10, true);
        let handlebars = RefCell::new(Handlebars::new());
        let mut encoder = CandEncoder::new(&handlebars);
        encoder.load_str(r#"holes_2 = {{holes.H__2}}"#)?;
        state.update_hole("H__2", 3);
        assert_eq!(encoder.render(&state)?, "holes_2 = 3");
        Ok(())
    }

    #[test]
    fn renders_from_tmp_file() -> Result<(), Box<dyn Error>> {
        let mut state = CEGISState::new([("func".to_string(), 1 as usize)].iter().cloned().collect(), 1, 10, true);
        let handlebars = RefCell::new(Handlebars::new());
        let mut encoder = CandEncoder::new(&handlebars);
        encoder.load_input_tmp_from_str(
            "<H__0  1> + <H__1> + <H__2 1 *> + <H__3 +> + MINVAR <H__4> + SPVAR 3 $ H__3 $ < H__5 $>");
        encoder.load_from_rewrite()?;
        state.update_hole("H__0", 1);
        state.update_hole("H__1", 2);
        state.update_hole("H__2", 3);
        state.update_hole("H__3", 4);
        state.update_hole("H__4", 5);
        state.update_hole("H__5", 6);
        assert_eq!(encoder.render(&state)?, "1 + 2 + 3 + 4 + 5 + 6");
        Ok(())
    }

}
