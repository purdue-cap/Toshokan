use handlebars::{Helper, Handlebars, Context,
                RenderContext, Output,
                HelperResult, RenderError};

pub fn get_n_logs(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting parameters: logs
    let logs_array = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_array()
        .ok_or(RenderError::new("First parameter not array"))?;
    out.write(format!("{}", logs_array.len()).as_str())?;
    Ok(())
}

pub fn get_cap_logs(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting parameters: logs, n_unknown
    let logs_array = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_array()
        .ok_or(RenderError::new("First parameter not array"))?;
    let n_unknown = h.param(1)
        .ok_or(RenderError::new("Second parameter not found"))?
        .value().as_u64()
        .ok_or(RenderError::new("Second parameter not an unsigned int"))?
        as usize;
    out.write(format!("{}", logs_array.len() + n_unknown).as_str())?;
    Ok(())
}

pub fn expand_to_arg_array(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting parameters: logs, index_of_arg, optional(n_unknown)
    let logs_array = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_array()
        .ok_or(RenderError::new("First parameter not array"))?;
    let index_of_arg = h.param(1)
        .ok_or(RenderError::new("Second parameter not found"))?
        .value().as_u64()
        .ok_or(RenderError::new("Second parameter not an unsigned int"))?
        as usize;
    let mut n_unknown = 0;
    if let Some(param_2) = h.param(2) {
        n_unknown = param_2.value().as_u64()
            .ok_or(RenderError::new("Third parameter not an unsigned int"))?
            as usize;
    }
    let mut arg_array = logs_array.into_iter().map(|j|
        j.as_object()
        .and_then(|obj| obj.get("args"))
        .and_then(|args_v| args_v.as_array())
        .and_then(|args| args.get(index_of_arg))
        .map(|arg_v| arg_v.to_string())
    ).collect::<Option<Vec<_>>>()
        .ok_or(RenderError::new("Arg array parse failed"))?;
    arg_array.append(&mut std::iter::repeat("0".to_string()).take(n_unknown).collect());
    out.write(arg_array.join(", ").as_str())?;
    Ok(())
}

pub fn expand_to_rtn_array(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting parameters: logs, optional(n_unknown)
    let logs_array = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_array()
        .ok_or(RenderError::new("First parameter not array"))?;
    let mut n_unknown = 0;
    if let Some(param_1) = h.param(1) {
        n_unknown = param_1.value().as_u64()
            .ok_or(RenderError::new("Second parameter not an unsigned int"))?
            as usize;
    }
    let mut arg_array = logs_array.into_iter().map(|j|
        j.as_object()
        .and_then(|obj| obj.get("rtn"))
        .map(|arg_v| arg_v.to_string())
    ).collect::<Option<Vec<_>>>()
        .ok_or(RenderError::new("Arg array parse failed"))?;
    arg_array.append(&mut std::iter::repeat("0".to_string()).take(n_unknown).collect());
    out.write(arg_array.join(", ").as_str())?;
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
    hb.register_helper("get-n-logs", Box::new(get_n_logs));
    hb.register_helper("get-cap-logs", Box::new(get_cap_logs));
    hb.register_helper("expand-to-arg-array", Box::new(expand_to_arg_array));
    hb.register_helper("expand-to-rtn-array", Box::new(expand_to_rtn_array));
    hb.register_helper("expand-points-to-assume", Box::new(expand_points_to_assume));
    hb.register_helper("expand-x-d-points-to-assume", Box::new(expand_x_d_points_to_assume));
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use serde::Serialize;
    use super::*;
    use crate::cegis::TraceLog;
    
    #[derive(Serialize)]
    struct Param {
        array: Vec<i32>
    }

    #[derive(Serialize)]
    struct XDParam {
        array: Vec<Vec<i32>>
    }

    #[derive(Serialize)]
    struct TraceLogParam {
        logs: Vec<TraceLog>
    }

    #[test]
    fn gets_n_logs() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        hb.register_helper("get-n-logs", Box::new(get_n_logs));

        let template = "n_logs: {{get-n-logs array}}";
        assert_eq!(hb.render_template(template, &data)?, "n_logs: 5");
        Ok(())
    }

    #[test]
    fn gets_cap_logs() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        hb.register_helper("get-cap-logs", Box::new(get_cap_logs));

        let template = "n_logs: {{get-cap-logs array 10}}";
        assert_eq!(hb.render_template(template, &data)?, "n_logs: 15");
        Ok(())
    }

    #[test]
    fn expands_to_arg_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
                TraceLog {
                    args: vec![json!(1)],
                    rtn: json!(1)
                },
                TraceLog {
                    args: vec![json!(5)],
                    rtn: json!(2)
                },
                TraceLog {
                    args: vec![json!(25)],
                    rtn: json!(5)
                }
            ]
        };
        let mut hb = Handlebars::new();
        hb.register_helper("expand-to-arg-array", Box::new(expand_to_arg_array));

        let template = "args: {{expand-to-arg-array logs 0 2}}";
        assert_eq!(hb.render_template(template, &data)?, "args: 1, 5, 25, 0, 0");

        Ok(())
    }

    #[test]
    fn expands_to_rtn_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
                TraceLog {
                    args: vec![json!(1)],
                    rtn: json!(1)
                },
                TraceLog {
                    args: vec![json!(5)],
                    rtn: json!(2)
                },
                TraceLog {
                    args: vec![json!(25)],
                    rtn: json!(5)
                }
            ]
        };
        let mut hb = Handlebars::new();
        hb.register_helper("expand-to-rtn-array", Box::new(expand_to_rtn_array));

        let template = "rtn: {{expand-to-rtn-array logs 2}}";
        assert_eq!(hb.render_template(template, &data)?, "rtn: 1, 2, 5, 0, 0");

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