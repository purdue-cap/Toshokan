use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::cegis::FuncLog;
use super::super::TraceError;
use std::io::BufRead;
use quick_error::ResultExt;
use std::iter::repeat;
use super::super::traits::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct JavaTracerLog {
    #[serde(rename = "className")]
    class_name: String,
    #[serde(rename = "methodName")]
    method_name: String,
    #[serde(rename = "methodSig")]
    method_sig: String,
    args: Vec<Value>,
    #[serde(rename = "ret")]
    rtn: Option<Value>,
    #[serde(rename = "thisObj")]
    this_obj: Option<Value>,
    #[serde(rename = "thisId")]
    this_id: Option<i32>,
}

pub struct LogAnalyzer {
    traces: Vec<Vec<FuncLog>>
}

static EMPTY_C_E: Vec<Vec<i32>> = Vec::new();
static EMPTY_UNWIND_ERR_LOOPS: Vec<String> = Vec::new();

// reads JVM signature strings and convert to parameter type strings
// e.g. (IZ)I -> int, boolean
fn parse_param_types_from_sig(sig_str: &str) -> Option<Vec<String>> {
    let mut class_name = String::new();

    enum Status {PrePar, InPar, PostPar, InClassName}
    let mut status = Status::PrePar;
    let mut array_layers: usize = 0;
    let mut param_types: Vec<String> = Vec::new();
    for c in sig_str.chars() {
        match c {
            '(' if matches!(status, Status::PrePar)
                => {status = Status::InPar;},
            'Z' if matches!(status, Status::InPar)
                => {
                    param_types.push(
                        format!("{}{}", "boolean", repeat("[]").take(array_layers).collect::<String>())
                    );
                    array_layers = 0;
                },
            'B' if matches!(status, Status::InPar)
                => {
                    param_types.push(
                        format!("{}{}", "byte", repeat("[]").take(array_layers).collect::<String>())
                    );
                    array_layers = 0;
                },
            'C' if matches!(status, Status::InPar)
                => {
                    param_types.push(
                        format!("{}{}", "char", repeat("[]").take(array_layers).collect::<String>())
                    );
                    array_layers = 0;
                },
            'S' if matches!(status, Status::InPar)
                => {
                    param_types.push(
                        format!("{}{}", "short", repeat("[]").take(array_layers).collect::<String>())
                    );
                    array_layers = 0;
                },
            'I' if matches!(status, Status::InPar)
                => {
                    param_types.push(
                        format!("{}{}", "int", repeat("[]").take(array_layers).collect::<String>())
                    );
                    array_layers = 0;
                },
            'J' if matches!(status, Status::InPar)
                => {
                    param_types.push(
                        format!("{}{}", "long", repeat("[]").take(array_layers).collect::<String>())
                    );
                    array_layers = 0;
                },
            'F' if matches!(status, Status::InPar)
                => {
                    param_types.push(
                        format!("{}{}", "float", repeat("[]").take(array_layers).collect::<String>())
                    );
                    array_layers = 0;
                },
            'D' if matches!(status, Status::InPar)
                => {
                    param_types.push(
                        format!("{}{}", "double", repeat("[]").take(array_layers).collect::<String>())
                    );
                    array_layers = 0;
                },
            '[' if matches!(status, Status::InPar)
                => {array_layers += 1;}
            ')' if matches!(status, Status::InPar)
                => {status = Status::PostPar;},
            'L' if matches!(status, Status::InPar)
                => {status = Status::InClassName;}
            ';' if matches!(status, Status::InClassName)
                => {
                    param_types.push(
                        format!("{}{}", class_name.replace("/", "."), repeat("[]").take(array_layers).collect::<String>())
                    );
                    class_name.clear();
                    array_layers = 0;
                    status = Status::InPar;
                },
            // FIXME: does not check if c is valid character in class name
            _ if matches!(status, Status::InClassName)
                => {
                    class_name.push(c);
                },
            _ if matches!(status, Status::PostPar) => {},
            _ => {return None;},
        }
    }
    if let Status::PostPar = status {
        Some(param_types)
    } else {None}
}

impl AnalyzeTracingVerifierLog for LogAnalyzer {
    type Error = TraceError;
    // FIXME: For now, javaTracer verifier is meant for fixed tests benches only
    // and is guaranteed to have no c.e.s, but this can be changed to enable random testing
    fn get_c_e_s(&self) -> &Vec<Vec<i32>> {&EMPTY_C_E}
    // Testing does not have unwind errors
    fn get_unwind_err_loops(&self) -> &Vec<String> {&EMPTY_UNWIND_ERR_LOOPS}
    fn get_traces(&self) -> &Vec<Vec<FuncLog>> {&self.traces}
    fn analyze_logs(&mut self, mut logs_reader: &[u8]) -> Result<bool, TraceError> {
        let mut buffer = String::new();
        let mut no_exceptions = true;
        // FIXME: there is only one sequence of trace here, corresponding to only one run traced
        // this is always true for fixed-test benches, but can be extended to support random testing
        let mut current_traces = vec![];
        loop {
            if logs_reader.read_line(&mut buffer)? == 0 {
                break;
            }
            if buffer.starts_with("[javaTracer] log:") {
                let log_content = &buffer["[javaTracer] log:".len()..];
                let java_tracer_log: JavaTracerLog = serde_json::from_str(log_content)
                    .context(log_content.as_bytes().to_vec())?;
                let func_name = if java_tracer_log.method_name == "<init>" {
                    format!("{}({})", java_tracer_log.class_name,
                        parse_param_types_from_sig(&java_tracer_log.method_sig)
                        .ok_or(TraceError::JavaTracerLogError("signature str parsing failure"))?
                        .join(", "))
                } else {
                    format!("{}.{}({})",
                        java_tracer_log.class_name,
                        java_tracer_log.method_name,
                        parse_param_types_from_sig(&java_tracer_log.method_sig)
                        .ok_or(TraceError::JavaTracerLogError("signature str parse failure"))?
                        .join(", "))
                };
                current_traces.push(FuncLog {
                    args: java_tracer_log.args,
                    this: java_tracer_log.this_id.map(|i| i.to_string()),
                    rtn: java_tracer_log.rtn,
                    func: func_name
                });
            } else if buffer.starts_with("Exception in thread") {
                no_exceptions = false;
            }
            buffer.clear();
        };
        self.traces.push(current_traces);
        Ok(no_exceptions)
    }
}

impl LogAnalyzer {
    pub fn new() -> Self {
        Self {
            traces: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    macro_rules! string_vec {
        [$($t:tt),*] => {
           vec![$($t.to_string()),*] 
        };
    }
    #[test]
    fn parses_jvm_sig_strings() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(parse_param_types_from_sig("(II)I").ok_or("parse error")?,
            string_vec!["int", "int"]);
        assert_eq!(parse_param_types_from_sig("(I[Z[[Ljava/lang/String;)I").ok_or("parse error")?,
            string_vec!["int", "boolean[]", "java.lang.String[][]"]);
        Ok(())
    }

    static SAMPLE_STDERR: &'static [u8] = include_bytes!("../../../tests/data/java_tracer_sample.stderr");
    #[test]
    fn parses_sample_stderr() -> Result<(), Box<dyn std::error::Error>> {
        let mut analyzer = LogAnalyzer::new();
        let result = analyzer.analyze_logs(SAMPLE_STDERR)?;
        assert_eq!(result, false);
        assert_eq!((&analyzer as &dyn AnalyzeVerifierLog<Error=_>).get_c_e_s(), &Vec::<Vec<i32>>::new());
        assert_eq!((&analyzer as &dyn AnalyzeTracerLog<Error=_>).get_traces(), &vec![vec![
            FuncLog {
                args: vec![json!(1i32)],
                this: None,
                rtn: None,
                func: "Test.test0(int)".into()
            },
            FuncLog {
                args: vec![json!(2i32)],
                this: None,
                rtn: None,
                func: "Test.test0(int)".into()
            },
            FuncLog {
                args: vec![json!(3i32)],
                this: None,
                rtn: Some(json!(0i32)),
                func: "Test.test1(int)".into()
            },
            FuncLog {
                args: vec![json!(4i32)],
                this: None,
                rtn: Some(json!(0i32)),
                func: "Test.test1(int)".into()
            },
            FuncLog {
                args: vec![json!(4i32)],
                this: Some("97730845".into()),
                rtn: Some(json!(7i32)),
                func: "Adder.doAdd(int)".into()
            },
        ]]);
        Ok(())
    }

    static SAMPLE_STDERR_WITH_INIT: &'static [u8] = include_bytes!("../../../tests/data/java_tracer_sample_init.stderr");
    #[test]
    fn parses_sample_stderr_with_init() -> Result<(), Box<dyn std::error::Error>> {
        let mut analyzer = LogAnalyzer::new();
        let result = analyzer.analyze_logs(SAMPLE_STDERR_WITH_INIT)?;
        assert_eq!(result, false);
        assert_eq!((&analyzer as &dyn AnalyzeVerifierLog<Error=_>).get_c_e_s(), &Vec::<Vec<i32>>::new());
        assert_eq!((&analyzer as &dyn AnalyzeTracerLog<Error=_>).get_traces(), &vec![vec![
            FuncLog {
                args: vec![json!(1i32)],
                this: None,
                rtn: None,
                func: "Test.test0(int)".into()
            },
            FuncLog {
                args: vec![json!(2i32)],
                this: None,
                rtn: None,
                func: "Test.test0(int)".into()
            },
            FuncLog {
                args: vec![json!(3i32)],
                this: None,
                rtn: Some(json!(0i32)),
                func: "Test.test1(int)".into()
            },
            FuncLog {
                args: vec![json!(4i32)],
                this: None,
                rtn: Some(json!(0i32)),
                func: "Test.test1(int)".into()
            },
            FuncLog {
                args: vec![json!(5i32), json!(6i32)],
                this: None,
                rtn: Some(json!(0i32)),
                func: "Test.test1(int, int)".into()
            },
            FuncLog {
                args: vec![json!(3i32)],
                this: Some("100555887".into()),
                rtn: None,
                func: "Adder(int)".into()
            },
            FuncLog {
                args: vec![json!(4i32)],
                this: Some("100555887".into()),
                rtn: Some(json!(7i32)),
                func: "Adder.doAdd(int)".into()
            },
            FuncLog {
                args: vec![json!(3i32), json!(4i32)],
                this: Some("1769597131".into()),
                rtn: None,
                func: "Adder(int, int)".into()
            },
            FuncLog {
                args: vec![json!(9i32)],
                this: Some("1769597131".into()),
                rtn: Some(json!(16i32)),
                func: "Adder.doAdd(int)".into()
            },
        ]]);
        Ok(())
    }


}