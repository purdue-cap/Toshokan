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

pub fn expand_points_to_assume(h: &Helper,
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
    if !val_vec.is_empty() {
        out.write("assume ")?;
        out.write(assign_vec.join(" || ").as_str())?;
        out.write(";")?;
    }
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

pub fn expand_x_d_points_to_assume(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    let xd_points: Vec<Vec<String>> = h.param(0)
                .ok_or(RenderError::new("XD array not found"))?
                .value().as_array()
                .ok_or(RenderError::new("XD array not an array"))?
                .iter().map(|val| {
                    val.as_array().ok_or(RenderError::new("Inner element not array")).map(
                        |vec| vec.iter().map(|val| val.to_string()).collect::<Vec<String>>()
                    )
                }).collect::<Result<Vec<Vec<String>>, RenderError>>()?;
    let var_names : Vec<&str> = h.params().iter().skip(1).map(
        |val| val.value().as_str()
    ).collect::<Option<Vec<&str>>>().ok_or(RenderError::new("Variable name arguments contains non-string"))?;

    let mut point_vec: Vec<String> = Vec::new();
    let mut idx = 0;
    loop {
        if let Some(joined_predicate) =
            var_names.iter().zip(xd_points.iter()).map(|(name, values)| {
                    values.get(idx).map(|value| format!("{} == {}", name, value))
            }).collect::<Option<Vec<String>>>().map(|predicates| predicates.join(" && ")) {
            point_vec.push(format!("({})", joined_predicate));
            idx += 1;
        } else {
            break;
        }
    };
    if !point_vec.is_empty() {
        out.write("assume ")?;
        out.write(point_vec.join(" || ").as_str())?;
        out.write(";")?;
    }
    Ok(())
}

pub fn register_helpers(hb: &mut Handlebars) {
    hb.register_helper("expand-array", Box::new(expand_array));
    hb.register_helper("expand-points-to-assume", Box::new(expand_points_to_assume));
    hb.register_helper("expand-partial-array", Box::new(expand_partial_array));
    hb.register_helper("expand-x-d-points-to-assume", Box::new(expand_x_d_points_to_assume));
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

    #[derive(Serialize)]
    struct XDParam {
        array: Vec<Vec<i32>>
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
    fn expands_points_to_assume() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        hb.register_helper("expand-points-to-assume", Box::new(expand_points_to_assume));

        let template = r#"{{expand-points-to-assume array "p"}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "assume p == 1 || p == 2 || p == 3 || p == 4 || p == 5;");
        Ok(())
    }

    #[test]
    fn expands_nothing_with_empty_points() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![] };
        
        let mut hb = Handlebars::new();
        hb.register_helper("expand-points-to-assume", Box::new(expand_points_to_assume));

        let template = r#"{{expand-points-to-assume array "p"}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "");
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
    fn expands_x_d_points_to_assume() -> Result<(), Box<dyn Error>> {
        let data = XDParam { array: vec![vec![1, 2, 3], vec![4 ,5, 6]]};
        
        let mut hb = Handlebars::new();
        hb.register_helper("expand-x-d-points-to-assume", Box::new(expand_x_d_points_to_assume));

        let template = r#"{{expand-x-d-points-to-assume array "a" "b"}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "assume (a == 1 && b == 4) || (a == 2 && b == 5) || (a == 3 && b == 6);");
        Ok(())
    }

    #[test]
    fn expands_nothing_with_empty_x_d_points() -> Result<(), Box<dyn Error>> {
        let data = XDParam { array: vec![vec![], vec![]]};
        
        let mut hb = Handlebars::new();
        hb.register_helper("expand-x-d-points-to-assume", Box::new(expand_x_d_points_to_assume));

        let template = r#"{{expand-x-d-points-to-assume array "a" "b"}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "");
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