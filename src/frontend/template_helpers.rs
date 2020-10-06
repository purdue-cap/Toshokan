use handlebars::{Helper, Handlebars, Context,
                RenderContext, Output,
                BlockContext, BlockParams,
                HelperResult, RenderError, Renderable};
use serde_json::value::Value;

handlebars_helper!(range: |x: u64| (0..x).collect::<Vec<u64>>());
handlebars_helper!(add: |x: u64, y: u64| x + y);
handlebars_helper!(subtree: |obj: object, key: str| obj.get(key).cloned().unwrap_or(Value::Null));

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

pub fn get_encoding_offset(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting parameters: function encoding code mapping
    let encoding_code_map = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_object()
        .ok_or(RenderError::new("First parameter not object"))?;
    out.write(format!("{}", encoding_code_map.len() + 1).as_str())?;
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
    // Expecting parameters: logs, index_of_arg, optional(fill_string), optional(n_unknown)
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
    let mut fill_string = "0".to_string();
    if let Some(param_2) = h.param(2) {
        match param_2.value() {
            Value::Number(param_number) => {
                n_unknown = param_number.as_u64()
                    .ok_or(RenderError::new("Third parameter not an unsigned int or string"))?
                    as usize;
            },
            Value::String(param_string) => {
                fill_string = param_string.clone();
                if let Some(param_3) = h.param(3) {
                    n_unknown = param_3.value().as_u64()
                        .ok_or(RenderError::new("Fourth parameter not an unsigned int"))?
                        as usize;
                }
            },
            _ => { return Err(RenderError::new("Third parameter not an unsigned int or string"));}
        }
    }
    let mut arg_array = logs_array.into_iter().map(|j|
        j.as_object()
        .and_then(|obj| obj.get("args"))
        .and_then(|args_v| args_v.as_array())
        .and_then(|args| args.get(index_of_arg))
        .and_then(|arg_v| {
            format_value(arg_v)
        })
    ).collect::<Option<Vec<_>>>()
        .ok_or(RenderError::new("Arg array parse failed"))?;
    arg_array.append(&mut std::iter::repeat(fill_string).take(n_unknown).collect());
    out.write(arg_array.join(", ").as_str())?;
    Ok(())
}

pub fn format_value(val: &Value) -> Option<String> {
    match val {
        Value::Array(ref val_vec) => {
            val_vec.iter().map(
                |v| format_value(v)
            ).collect::<Option<Vec<_>>>()
            .map(|str_vec| format!( "{{ {} }}",str_vec.join(", ")))
        },
        Value::Object(ref val_map) => {
            let full_name = val_map.get("@class_name")?.as_str()?;
            let split_name = full_name.split("::").collect::<Vec<_>>();
            let ns_name = split_name.get(0)?;
            let struct_name = split_name.get(1)?;
            let sketch_name : String;
            if *ns_name == "ANONYMOUS" {
                sketch_name = struct_name.to_string();
            } else {
                sketch_name = format!("{}@{}", struct_name, ns_name);
            }
            val_map.iter()
            .filter(|(k, _v)| !k.starts_with("@"))
            .map(|(k, v)| format_value(v)
                .map(|format_result| format!("{}={}", k, format_result))
            ).collect::<Option<Vec<_>>>()
            .map(|str_vec| format!( "new {}( {} )", sketch_name, str_vec.join(", ")))
        },
        _ => {
            Some(val.to_string())
        }
    }
}

pub fn expand_to_hist_arrays(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting parameters: logs, n_unknown, history_capacity, optional(unknown_fill_string), optional(hist_fill_string)
    let logs_array = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_array()
        .ok_or(RenderError::new("First parameter not array"))?;
    let n_unknown = h.param(1)
        .ok_or(RenderError::new("Second parameter not found"))?
        .value().as_u64()
        .ok_or(RenderError::new("Second parameter not an unsigned int"))?
        as usize;
    let hist_cap = h.param(2)
        .ok_or(RenderError::new("Third parameter not found"))?
        .value().as_u64()
        .ok_or(RenderError::new("Third parameter not an unsigned int"))?
        as usize;
    let mut unknown_fill_string = "??".to_string();
    let mut hist_fill_string = "0".to_string();
    if let Some(param_3) = h.param(3) {
        unknown_fill_string = param_3.value().as_str().ok_or(RenderError::new("Fourth parameter not a string"))?.to_string();
    }
    if let Some(param_4) = h.param(4) {
        hist_fill_string = param_4.value().as_str().ok_or(RenderError::new("Fifth parameter not a string"))?.to_string();
    }
    let mut arg_array = logs_array.into_iter().map(|j|
        j.as_object()
        .and_then(|obj| obj.get("args"))
        .and_then(|args_v| args_v.as_array())
        .and_then(|hist_v| hist_v.into_iter().map(|v| format_value(v)).collect::<Option<Vec<_>>>())
        .and_then(|mut hist_v|
            if hist_v.len() > hist_cap {
                None
            } else {
                hist_v.append(&mut vec![hist_fill_string.clone(); hist_cap - hist_v.len()]);
                Some(hist_v)
            }
        )
        .map(|hist_v| format!("{{ {} }}", hist_v.join(", ")))
    ).collect::<Option<Vec<_>>>()
        .ok_or(RenderError::new("Hist array parse failed"))?;
    arg_array.append(&mut std::iter::repeat(unknown_fill_string).take(n_unknown).collect());
    out.write(arg_array.join(", ").as_str())?;
    Ok(())
}

pub fn expand_to_hist_lens(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting parameters: logs, n_unknown, optional(unknown_fill_string)
    let logs_array = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_array()
        .ok_or(RenderError::new("First parameter not array"))?;
    let n_unknown = h.param(1)
        .ok_or(RenderError::new("Second parameter not found"))?
        .value().as_u64()
        .ok_or(RenderError::new("Second parameter not an unsigned int"))?
        as usize;
    let mut unknown_fill_string = "??".to_string();
    if let Some(param_2) = h.param(2) {
        unknown_fill_string = param_2.value().as_str().ok_or(RenderError::new("Third parameter not a string"))?.to_string();
    }
    let mut len_array = logs_array.into_iter().map(|j|
        j.as_object()
        .and_then(|obj| obj.get("args"))
        .and_then(|args_v| args_v.as_array())
        .map(|hist_v| format!("{}", hist_v.len()))
    ).collect::<Option<Vec<_>>>()
        .ok_or(RenderError::new("Hist array parse failed"))?;
    len_array.append(&mut std::iter::repeat(unknown_fill_string).take(n_unknown).collect());
    out.write(len_array.join(", ").as_str())?;
    Ok(())
}

pub fn expand_to_rtn_array(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting parameters: logs, optional(fill_string), optional(n_unknown)
    let logs_array = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_array()
        .ok_or(RenderError::new("First parameter not array"))?;
    let mut n_unknown = 0;
    let mut fill_string = "0".to_string();
    if let Some(param_1) = h.param(1) {
        match param_1.value() {
            Value::Number(param_number) => {
                n_unknown = param_number.as_u64()
                    .ok_or(RenderError::new("Second parameter not an unsigned int or string"))?
                    as usize;
            },
            Value::String(param_string) => {
                fill_string = param_string.clone();
                if let Some(param_2) = h.param(2) {
                    n_unknown = param_2.value().as_u64()
                        .ok_or(RenderError::new("Third parameter not an unsigned int"))?
                        as usize;
                }
            },
            _ => { return Err(RenderError::new("Second parameter not an unsigned int or string"));}
        }
    }
    let mut rtn_array = logs_array.into_iter().map(|j|
        j.as_object()
        .and_then(|obj| obj.get("rtn"))
        .and_then(|arg_v| {
            format_value(arg_v)
        })
    ).collect::<Option<Vec<_>>>()
        .ok_or(RenderError::new("Arg array parse failed"))?;
    rtn_array.append(&mut std::iter::repeat(fill_string).take(n_unknown).collect());
    out.write(rtn_array.join(", ").as_str())?;
    Ok(())
}

pub fn expand_to_ith_rtn_array(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting parameters: logs, index_of_rtn, optional(fill_string) optional(n_unknown)
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
    let mut fill_string = "0".to_string();
    if let Some(param_2) = h.param(2) {
        match param_2.value() {
            Value::Number(param_number) => {
                n_unknown = param_number.as_u64()
                    .ok_or(RenderError::new("Third parameter not an unsigned int or string"))?
                    as usize;
            },
            Value::String(param_string) => {
                fill_string = param_string.clone();
                if let Some(param_3) = h.param(3) {
                    n_unknown = param_3.value().as_u64()
                        .ok_or(RenderError::new("Fourth parameter not an unsigned int"))?
                        as usize;
                }
            },
            _ => { return Err(RenderError::new("Third parameter not an unsigned int or string"));}
        }
    }
    let mut rtn_array = logs_array.into_iter().map(|j|
        j.as_object()
        .and_then(|obj| obj.get("rtn"))
        .and_then(|args_v| args_v.as_array())
        .and_then(|args| args.get(index_of_arg))
        .and_then(|arg_v| {
            format_value(arg_v)
        })
    ).collect::<Option<Vec<_>>>()
        .ok_or(RenderError::new("Arg array parse failed"))?;
    rtn_array.append(&mut std::iter::repeat(fill_string).take(n_unknown).collect());
    out.write(rtn_array.join(", ").as_str())?;
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

pub fn expand_holes(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting argument: number_of_holes
    let n_unknown = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_u64()
        .ok_or(RenderError::new("First parameter not an unsigned int"))?
        as usize;
    out.write(std::iter::repeat("??").take(n_unknown).collect::<Vec<_>>().join(", ").as_str())?;
    Ok(())
}

pub fn expand_comma_list(h: &Helper,
                    _: &Handlebars,
                    _: &Context,
                    _: &mut RenderContext,
                    out: &mut dyn Output) -> HelperResult {
    // Expecting argument: list_to_expand 
    let array = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_array()
        .ok_or(RenderError::new("First parameter not array"))?
        .iter()
        .map(|e| e.to_string()).collect::<Vec<_>>();
    out.write(array.join(", ").as_str())?;
    Ok(())
}

pub fn for_cap_logs<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    r: &'reg Handlebars,
    ctx: &'rc Context,
    rc: &mut RenderContext<'reg, 'rc>,
    out: &mut dyn Output,
) -> HelperResult {
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
    let iter_count = logs_array.len() + n_unknown;
    let inner_template = h.template().ok_or(RenderError::new("Not called as block helper"))?;
    let block_context = BlockContext::new();
    rc.push_block(block_context);
    for i in 0..iter_count {
        let block_context = rc.block_mut().ok_or(RenderError::new("Block context not found"))?;
        let is_first = i == 0;
        let is_last = i == iter_count - 1;
        block_context.set_local_var("@index".to_string(), Value::from(i));
        block_context.set_local_var("@first".to_string(), Value::from(is_first));
        block_context.set_local_var("@last".to_string(), Value::from(is_last));
        if let Some(bp_val) = h.block_param() {
            let mut params = BlockParams::new();
            params.add_value(bp_val, Value::from(i))?;

            block_context.set_block_params(params);
        }
        inner_template.render(r, ctx, rc, out)?;
    }
    rc.pop_block();
    Ok(())
}

pub fn for_trans_c_e<'reg, 'rc>(
    h: &Helper<'reg, 'rc>,
    r: &'reg Handlebars,
    ctx: &'rc Context,
    rc: &mut RenderContext<'reg, 'rc>,
    out: &mut dyn Output,
) -> HelperResult {
    // Expecting parameters: c_e_s
    let c_e_s = h.param(0)
        .ok_or(RenderError::new("First parameter not found"))?
        .value().as_array()
        .ok_or(RenderError::new("First parameter not array"))?
        .iter()
        .map(|one_c_e_vec| one_c_e_vec.as_array())
        .collect::<Option<Vec<_>>>()
        .ok_or(RenderError::new("c_e_s not a 2D array"))?
        .iter()
        .map(|one_c_e_vec|
            one_c_e_vec.iter().map(
                |each_c_e| each_c_e.as_i64()
            ).collect::<Option<Vec<_>>>()
        )
        .collect::<Option<Vec<_>>>()
        .ok_or(RenderError::new("c_e_s contains non integer element"))?;
    let iter_count = c_e_s.iter().map(|one_c_e_vec| one_c_e_vec.len()).min().ok_or(RenderError::new("Error in resolving C.E. count"))?;
    let inner_template = h.template().ok_or(RenderError::new("Not called as block helper"))?;
    let block_context = BlockContext::new();
    rc.push_block(block_context);
    for i in 0..iter_count {
        let block_context = rc.block_mut().ok_or(RenderError::new("Block context not found"))?;
        let is_first = i == 0;
        let is_last = i == iter_count - 1;
        block_context.set_local_var("@index".to_string(), Value::from(i));
        block_context.set_local_var("@first".to_string(), Value::from(is_first));
        block_context.set_local_var("@last".to_string(), Value::from(is_last));
        if let Some(bp_val) = h.block_param() {
            let mut params = BlockParams::new();
            let c_e_vec = c_e_s.iter().map(
                |one_c_e_vec| one_c_e_vec.get(i).map(|n| *n)
            ).collect::<Option<Vec<_>>>()
            .ok_or(RenderError::new("Error in creating c_e_vec"))?;
            params.add_value(bp_val, Value::from(c_e_vec))?;

            block_context.set_block_params(params);
        }
        inner_template.render(r, ctx, rc, out)?;
    }
    rc.pop_block();
    Ok(())
}
pub fn register_helpers(hb: &mut Handlebars) {
    hb.register_helper("range", Box::new(range));
    hb.register_helper("add", Box::new(add));
    hb.register_helper("subtree", Box::new(subtree));
    hb.register_helper("get-n-logs", Box::new(get_n_logs));
    hb.register_helper("get-encoding-offset", Box::new(get_encoding_offset));
    hb.register_helper("get-cap-logs", Box::new(get_cap_logs));
    hb.register_helper("expand-to-arg-array", Box::new(expand_to_arg_array));
    hb.register_helper("expand-to-hist-arrays", Box::new(expand_to_hist_arrays));
    hb.register_helper("expand-to-hist-lens", Box::new(expand_to_hist_lens));
    hb.register_helper("expand-to-rtn-array", Box::new(expand_to_rtn_array));
    hb.register_helper("expand-to-ith-rtn-array", Box::new(expand_to_ith_rtn_array));
    hb.register_helper("expand-points-to-assume", Box::new(expand_points_to_assume));
    hb.register_helper("expand-x-d-points-to-assume", Box::new(expand_x_d_points_to_assume));
    hb.register_helper("expand-holes", Box::new(expand_holes));
    hb.register_helper("expand-comma-list", Box::new(expand_comma_list));
    hb.register_helper("for-cap-logs", Box::new(for_cap_logs));
    hb.register_helper("for-trans-c-e", Box::new(for_trans_c_e));
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use serde::Serialize;
    use super::*;
    use crate::cegis::{TraceLog, FuncLog};
    use std::collections::HashMap;
    
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

    #[derive(Serialize)]
    struct HistLogParam {
        encoding: HashMap<String, usize>,
        logs: Vec<TraceLog>
    }

    #[test]
    fn gets_subtree() -> Result<(), Box<dyn Error>> {
        let data = json!(
            {
                "logs": {
                    "array": [0, 1, 2]
                }
            }
        );
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"{{subtree logs "array"}}"#;
        assert_eq!(hb.render_template(template, &data)?, "[0, 1, 2, ]");
        Ok(())
    }

    #[test]
    fn gets_n_logs() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = "n_logs: {{get-n-logs array}}";
        assert_eq!(hb.render_template(template, &data)?, "n_logs: 5");
        Ok(())
    }

    #[test]
    fn gets_encoding_offset() -> Result<(), Box<dyn Error>> {
        let data = HistLogParam {
            encoding: vec![
                ("push".to_string(), 1usize), 
                ("pop".to_string(), 2usize)].into_iter().collect(),
            logs: vec![]
        };

        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"offset: {{get-encoding-offset encoding}}"#;
        assert_eq!(hb.render_template(template, &data)?, "offset: 3");

        Ok(())
    }

    #[test]
    fn gets_cap_logs() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = "n_logs: {{get-cap-logs array 10}}";
        assert_eq!(hb.render_template(template, &data)?, "n_logs: 15");
        Ok(())
    }

    #[test]
    fn expands_to_arg_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(1)],
                    rtn: json!(1),
                    func: "sqrt".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(5)],
                    rtn: json!(2),
                    func: "sqrt".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(25)],
                    rtn: json!(5),
                    func: "sqrt".to_string()
                })
            ]
        };
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = "args: {{expand-to-arg-array logs 0 2}}";
        assert_eq!(hb.render_template(template, &data)?, "args: 1, 5, 25, 0, 0");

        Ok(())
    }

    #[test]
    fn expands_objects_to_arg_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
                TraceLog::FuncCall(FuncLog {
                    args: vec![
                        json!({
                            "@class_name": "ANONYMOUS::point",
                            "a": 1,
                            "b": 2
                        })
                    ],
                    rtn: json!({
                        "@class_name": "std::vector",
                        "a": 2,
                        "b": 1
                    }),
                    func: "inv".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![
                        json!({
                            "@class_name": "ANONYMOUS::point",
                            "a": 3,
                            "b": 4
                        })
                    ],
                    rtn: json!({
                        "@class_name": "std::vector",
                        "a": 4,
                        "b": 3
                    }),
                    func: "inv".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![
                        json!({
                            "@class_name": "ANONYMOUS::point",
                            "a": 5,
                            "b": 4
                        })
                    ],
                    rtn: json!({
                        "@class_name": "std::vector",
                        "a": 4,
                        "b": 5
                    }),
                    func: "inv".to_string()
                }),
            ]
        };
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"args: {{expand-to-arg-array logs 0 "null" 2}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "args: new point( a=1, b=2 ), new point( a=3, b=4 ), new point( a=5, b=4 ), null, null");
        Ok(())
    }

    #[test]
    fn expands_empty_objects_to_arg_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
            ]
        };
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"args: {{expand-to-arg-array logs 0 "null" 2}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "args: null, null");
        Ok(())
    }

    #[test]
    fn expands_to_hist_arrays() -> Result<(), Box<dyn Error>> {
        let data = HistLogParam {
            encoding: vec![
                ("push".to_string(), 1usize), 
                ("pop".to_string(), 2usize)].into_iter().collect(),
            logs: vec![
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(2), json!(1), json!(6)],
                    rtn: json!(3),
                    func: "pop".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(2), json!(1), json!(1), json!(3), json!(8)],
                    rtn: json!(5),
                    func: "pop".to_string()
                })
            ]
        };

        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"hist: {{expand-to-hist-arrays logs 2 8}}"#;
        assert_eq!(hb.render_template(template, &data)?, "hist: { 2, 1, 6, 0, 0, 0, 0, 0 }, { 2, 1, 1, 3, 8, 0, 0, 0 }, ??, ??");

        Ok(())
    }

    #[test]
    fn expands_to_hist_lens() -> Result<(), Box<dyn Error>> {
        let data = HistLogParam {
            encoding: vec![
                ("push".to_string(), 1usize), 
                ("pop".to_string(), 2usize)].into_iter().collect(),
            logs: vec![
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(2), json!(1), json!(6)],
                    rtn: json!(3),
                    func: "pop".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(2), json!(1), json!(1), json!(3), json!(8)],
                    rtn: json!(5),
                    func: "pop".to_string()
                })
            ]
        };

        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"hist_len: {{expand-to-hist-lens logs 2}}"#;
        assert_eq!(hb.render_template(template, &data)?, "hist_len: 3, 5, ??, ??");

        Ok(())
    }

    #[test]
    fn expands_to_rtn_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(1)],
                    rtn: json!(1),
                    func: "sqrt".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(5)],
                    rtn: json!(2),
                    func: "sqrt".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(25)],
                    rtn: json!(5),
                    func: "sqrt".to_string()
                })
            ]
        };
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = "rtn: {{expand-to-rtn-array logs 2}}";
        assert_eq!(hb.render_template(template, &data)?, "rtn: 1, 2, 5, 0, 0");

        Ok(())
    }

    #[test]
    fn expands_objects_to_rtn_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
                TraceLog::FuncCall(FuncLog {
                    args: vec![
                        json!({
                            "@class_name": "ANONYMOUS::point",
                            "a": 1,
                            "b": 2
                        })
                    ],
                    rtn: json!({
                        "@class_name": "std::vector",
                        "a": 2,
                        "b": 1
                    }),
                    func: "inv".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![
                        json!({
                            "@class_name": "ANONYMOUS::point",
                            "a": 3,
                            "b": 4
                        })
                    ],
                    rtn: json!({
                        "@class_name": "std::vector",
                        "a": 4,
                        "b": 3
                    }),
                    func: "inv".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![
                        json!({
                            "@class_name": "ANONYMOUS::point",
                            "a": 5,
                            "b": 4
                        })
                    ],
                    rtn: json!({
                        "@class_name": "std::vector",
                        "a": 4,
                        "b": 5
                    }),
                    func: "inv".to_string()
                })
            ]
        };
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"rtn: {{expand-to-rtn-array logs "null" 2 }}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "rtn: new vector@std( a=2, b=1 ), new vector@std( a=4, b=3 ), new vector@std( a=4, b=5 ), null, null");
        Ok(())
    }

    #[test]
    fn expands_empty_objects_to_rtn_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
            ]
        };
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"rtn: {{expand-to-rtn-array logs "null" 2}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "rtn: null, null");
        Ok(())
    }

    #[test]
    fn expands_to_ith_rtn_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(1), json!(2)],
                    rtn: json!(vec![1, 2]),
                    func: "vec".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(5), json!(2)],
                    rtn: json!(vec![2, 5]),
                    func: "vec".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(25), json!(26)],
                    rtn: json!(vec![25, 26]),
                    func: "vec".to_string()
                })
            ]
        };
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = "rtn_1: {{expand-to-ith-rtn-array logs 1 2}}";
        assert_eq!(hb.render_template(template, &data)?, "rtn_1: 2, 5, 26, 0, 0");

        Ok(())
    }

    #[test]
    fn expands_objects_to_ith_rtn_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(1)],
                    rtn: json!([{
                        "@class_name": "std::vector",
                        "a": 1,
                        "b": 0
                    }, {
                        "@class_name": "std::vector",
                        "a": 0,
                        "b": 1
                    }
                    ]),
                    func:"inv".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(2)],
                    rtn: json!([{
                        "@class_name": "std::vector",
                        "a": 2,
                        "b": 0
                    }, {
                        "@class_name": "std::vector",
                        "a": 0,
                        "b": 2
                    }
                    ]),
                    func:"inv".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(3)],
                    rtn: json!([{
                        "@class_name": "std::vector",
                        "a": 3,
                        "b": 0
                    }, {
                        "@class_name": "std::vector",
                        "a": 0,
                        "b": 3
                    }
                    ]),
                    func:"inv".to_string()
                })
            ]
        };
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"rtn: {{expand-to-ith-rtn-array logs 1 "null" 2 }}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "rtn: new vector@std( a=0, b=1 ), new vector@std( a=0, b=2 ), new vector@std( a=0, b=3 ), null, null");
        Ok(())
    }

    #[test]
    fn expands_empty_objects_to_ith_rtn_array() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
            ]
        };
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"rtn: {{expand-to-ith-rtn-array logs 0 "null" 2}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "rtn: null, null");
        Ok(())
    }

    #[test]
    fn expands_points_to_assume() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"{{expand-points-to-assume array "p"}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "assume p == 1 || p == 2 || p == 3 || p == 4 || p == 5;");
        Ok(())
    }

    #[test]
    fn expands_nothing_with_empty_points() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![] };
        
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"{{expand-points-to-assume array "p"}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "");
        Ok(())
    }

    #[test]
    fn expands_x_d_points_to_assume() -> Result<(), Box<dyn Error>> {
        let data = XDParam { array: vec![vec![1, 2, 3], vec![4 ,5, 6]]};
        
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"{{expand-x-d-points-to-assume array "a" "b"}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "assume (a == 1 && b == 4) || (a == 2 && b == 5) || (a == 3 && b == 6);");
        Ok(())
    }

    #[test]
    fn expands_nothing_with_empty_x_d_points() -> Result<(), Box<dyn Error>> {
        let data = XDParam { array: vec![vec![], vec![]]};
        
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"{{expand-x-d-points-to-assume array "a" "b"}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "");
        Ok(())
    }

    #[test]
    fn expands_holes() -> Result<(), Box<dyn Error>> {
        let data = json!({"hole_count": 5});

        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = r#"holes: { {{expand-holes hole_count}} }"#;
        assert_eq!(hb.render_template(template, &data)?,
            "holes: { ??, ??, ??, ??, ?? }");
        Ok(())
    }

    #[test]
    fn iters_over_cap_logs() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = "indexes: {{#for-cap-logs array 3}}{{@index}}|{{@first}}|{{@last}} {{/for-cap-logs}}";
        assert_eq!(hb.render_template(template, &data)?,
            "indexes: 0|true|false 1|false|false 2|false|false 3|false|false 4|false|false 5|false|false 6|false|false 7|false|true ");
        Ok(())
    }

    #[test]
    fn iters_over_cap_logs_with_nested_iteration() -> Result<(), Box<dyn Error>> {
        let data = Param { array: vec![1, 2, 3, 4 ,5] };
        
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        let template = "indexes: {{#for-cap-logs array 3 as |i|}}{{#each (range 3)}}{{add @index i}}{{/each}} {{/for-cap-logs}}";
        assert_eq!(hb.render_template(template, &data)?,
            "indexes: 012 123 234 345 456 567 678 789 ");
        Ok(())
    }

    #[test]
    fn iters_over_c_e_s() -> Result<(), Box<dyn Error>> {
        let data = json!( {
            "c_e_s": [[1,2,3], [4,5,6]]
        }
        );

        let mut hb = Handlebars::new();
        register_helpers(&mut hb);
        
        let template = r#"{{#for-trans-c-e c_e_s as |l|}}{{expand-comma-list l}}{{#unless @last}}|{{/unless}}{{/for-trans-c-e}}"#;
        assert_eq!(hb.render_template(template, &data)?,
            "1, 4|2, 5|3, 6");
        Ok(())
    }

    #[test]
    fn expands_nested_templates() -> Result<(), Box<dyn Error>> {
        let data = TraceLogParam {
            logs: vec![
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(1)],
                    rtn: json!(1),
                    func: "sqrt".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(5)],
                    rtn: json!(2),
                    func: "sqrt".to_string()
                }),
                TraceLog::FuncCall(FuncLog {
                    args: vec![json!(25)],
                    rtn: json!(5),
                    func: "sqrt".to_string()
                })
            ]
        };
        let mut hb = Handlebars::new();
        register_helpers(&mut hb);

        // TODO: Currently there's a bug with handlebars-rust (reported: sunng87/handlebars-rust#343) forbids the correct use of 
        // "each" built-helpers. Only @index are correct within "each" contexts, use that as a workaround for the moment.
        let template =
r#"{{#each (range 3)}}{{expand-to-arg-array logs 0 (add @index 1)}}
{{/each}}"#;
        assert_eq!(hb.render_template(template, &data)?, 
r#"1, 5, 25, 0
1, 5, 25, 0, 0
1, 5, 25, 0, 0, 0
"#);

        Ok(())
    }
}