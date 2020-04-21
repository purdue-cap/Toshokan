use handlebars::{Helper, Handlebars, Context,
                RenderContext, Output,
                HelperResult, RenderError};

pub fn expand_array(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    let val_vec: Vec<String> = h.param(0)
                .ok_or(RenderError::new("Parameter not found"))?
                .value().as_array()
                .ok_or(RenderError::new("Parameter not an array"))?
                .iter().map(|val| val.to_string()).collect();
    out.write(val_vec.join(", ").as_str())?;
    Ok(())
}

pub fn expand_points(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    let val_vec: Vec<String> = h.param(0)
                .ok_or(RenderError::new("Points not found"))?
                .value().as_array()
                .ok_or(RenderError::new("Points not an array"))?
                .iter().map(|val| val.to_string()).collect();
    let var_name = h.param(1).ok_or(RenderError::new("Variable not found"))?
                .value().as_str()
                .ok_or(RenderError::new("Variable not a string"))?;
    let assign_vec : Vec<String> = val_vec.iter()
                .map(|val| format!("{} == {}", var_name, val))
                .collect();
    out.write(assign_vec.join(" || ").as_str())?;
    Ok(())
}

pub fn expand_partial_array(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    let mut val_vec: Vec<String> = h.param(0)
                .ok_or(RenderError::new("Array input not found"))?
                .value().as_array()
                .ok_or(RenderError::new("Array input not an array"))?
                .iter().map(|val| val.to_string()).collect();
    let extra_elements = h.param(1).ok_or(RenderError::new("Extra elements argument not found"))?
                .value().as_u64()
                .ok_or(RenderError::new("Extra elements not an unsigned int"))?;
    val_vec.append(&mut std::iter::repeat("0".to_string()).take(extra_elements as usize).collect());
    out.write(val_vec.join(", ").as_str())?;
    Ok(())
    }

pub fn register_helpers(hb: &mut Handlebars) {
    hb.register_helper("expand-array", Box::new(expand_array));
    hb.register_helper("expand-points", Box::new(expand_points));
    hb.register_helper("expand-partial-array", Box::new(expand_partial_array));
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use serde::Serialize;
    use super::*;
    
    #[derive(Serialize)]
    struct Param {
        array: Vec<i32>
    }

    #[test]
    fn expands_arrays() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        hb.register_helper("expand-array", Box::new(expand_array));

        let template = "Array: { {{expand-array array}} }";
        assert_eq!(hb.render_template(template, &data)?, "Array: { 1, 2, 3, 4, 5 }");
        Ok(())
    }

    #[test]
    fn expands_points() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        hb.register_helper("expand-points", Box::new(expand_points));

        let template = r#"assume {{expand-points array "p"}};"#;
        assert_eq!(hb.render_template(template, &data)?,
            "assume p == 1 || p == 2 || p == 3 || p == 4 || p == 5;");
        Ok(())
    }

    #[test]
    fn expands_partial_arrays() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        hb.register_helper("expand-partial-array", Box::new(expand_partial_array));

        let template = "Array: { {{expand-partial-array array 3}} }";
        assert_eq!(hb.render_template(template, &data)?, "Array: { 1, 2, 3, 4, 5, 0, 0, 0 }");
        Ok(())
    }

    #[test]
    fn registers_helpers() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"Array: { {{expand-array array}} }, assume {{expand-points array "p"}};"#;
        assert_eq!(hb.render_template(template, &data)?,
            "Array: { 1, 2, 3, 4, 5 }, assume p == 1 || p == 2 || p == 3 || p == 4 || p == 5;");
        Ok(())
    }

}