use serde_json::Value;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use crate::cegis::FuncLog;
use regex::Regex;
use super::super::TraceError::{self, JBMCLogError};

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum LogItem {
    Program {
        program: String
    },
    Message {
        #[serde(rename = "messageText")]
        message_text: String,
        #[serde(rename = "messageType")]
        message_type: String
    },
    Result {
        result: Vec<VerifyResult>
    },
    CProverStatus {
        #[serde(rename = "cProverStatus")]
        status: StatusInfo
    },
}

pub type VerifyLogs = Vec<LogItem>;

#[derive(Deserialize, Serialize, Debug)]
pub struct VerifyResult {
    description: String,
    property: String,
    status: StatusInfo,
    #[serde(default)]
    trace: Vec<VerifyTrace>,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum StatusInfo {
    #[serde(rename = "SUCCESS")]
    #[serde(alias = "success")]
    Success,
    #[serde(rename = "FAILURE")]
    #[serde(alias = "failure")]
    Failure
}


#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "stepType")]
pub enum VerifyTrace {
    #[serde(rename = "assignment")]
    Assignment {
        #[serde(rename = "assignmentType")]
        assignment_type: AssignmentTypeInfo,
        lhs: String,
        #[serde(rename = "rawLhs")]
        raw_lhs: HashMap<String, Value>,
        mode: String,
        value: ValueInfo,

        #[serde(flatten)]
        info: TraceInfo,
        #[serde(flatten)]
        others: HashMap<String, Value>
    },
    #[serde(rename = "function-call")]
    FunctionCall {
        function: FunctionInfo,

        #[serde(flatten)]
        info: TraceInfo,
        #[serde(flatten)]
        others: HashMap<String, Value>
    },
    #[serde(rename = "function-return")]
    FunctionReturn {
        function: FunctionInfo,

        #[serde(flatten)]
        info: TraceInfo,
        #[serde(flatten)]
        others: HashMap<String, Value>
    },
    #[serde(rename = "input")]
    Input {
        #[serde(rename = "inputID")]
        input_id: String,
        mode: String,
        values: Vec<ValueInfo>,

        #[serde(flatten)]
        info: TraceInfo,
        #[serde(flatten)]
        others: HashMap<String, Value>
    },
    #[serde(rename = "location-only")]
    LocationOnly {
        hidden: bool,
        #[serde(rename = "sourceLocation")]
        source_location: SourceLocationInfo,
        thread: usize,

        #[serde(flatten)]
        others: HashMap<String, Value>
    },
    #[serde(rename = "failure")]
    Failure {
        property: String,
        reason: String,

        #[serde(flatten)]
        info: TraceInfo,
        #[serde(flatten)]
        others: HashMap<String, Value>
    },
    #[serde(other)]
    Other
}

impl VerifyTrace {
    // Returns: (index, value)
    fn parse_as_input(&self) -> Result<(usize, i32), TraceError> {
        if let VerifyTrace::Input{input_id, values, ..} = self {
            let id_regex = Regex::new(r"arg(\d+)i").expect("Hardcoded regex");
            // Parse the index out of input_id
            if let Some(index) = 
                id_regex.captures(input_id)
                .and_then(|cap| cap.get(1))
                .and_then(|m| m.as_str().parse::<usize>().ok()) {
                // Currently, only work with single integer values
                if values.len() != 1 {
                    return Err(JBMCLogError("Unsupported values length"));
                }
                let value = values.get(0).expect("Length checked");
                Ok((index, value.parse_as_int()?))
            } else {
                Err(JBMCLogError("Unsupported input var name"))
            }
        } else {
            unreachable!("VerifyTrace variant mismatch");
        }
    }
    // Returns: (index, value)
    fn parse_as_param_assign(&self) -> Result<(usize, i32), TraceError> {
        if let VerifyTrace::Assignment {
            assignment_type: AssignmentTypeInfo::ActualParameter,
            lhs, value, ..
            } = self {
            let id_regex = Regex::new(r"arg(\d+)i").expect("Hardcoded regex");
            // Parse the index out of lhs
            if let Some(index) = 
                id_regex.captures(lhs)
                .and_then(|cap| cap.get(1))
                .and_then(|m| m.as_str().parse::<usize>().ok()) {
                Ok((index, value.parse_as_int()?))
            } else {
                Err(JBMCLogError("unsupported input var name"))
            }

        } else {
            unreachable!("VerifyTrace variant mismatch");
        }
    }
    // Returns (var_name, value)
    fn parse_as_var_assign(&self) -> Result<(&str, i32), TraceError> {
        if let VerifyTrace::Assignment {
            assignment_type: AssignmentTypeInfo::Variable,
            lhs, value, ..
            } = self {
            Ok((lhs.as_str(), value.parse_as_int()?))
        } else {
            unreachable!("VerifyTrace variant mismatch");
        }

    }
}

#[derive(Deserialize, Serialize, Debug)]
pub enum AssignmentTypeInfo {
    #[serde(rename = "variable")]
    Variable,
    #[serde(rename = "actual-parameter")]
    ActualParameter
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TraceInfo {
    hidden: bool,
    internal: bool,
    thread: usize,
    #[serde(rename = "sourceLocation")]
    #[serde(default)]
    source_location: SourceLocationInfo
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FunctionInfo {
    #[serde(rename = "displayName")]
    display_name: String,
    identifier: String,
    #[serde(rename = "sourceLocation")]
    source_location: SourceLocationInfo
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "name")]
pub enum ValueInfo {
    #[serde(rename = "integer")]
    Integer {
        binary: String,
        data: String,
        #[serde(rename = "type")]
        data_type: String,
        width: usize
    },
    #[serde(rename = "boolean")]
    Boolean {
        binary: String,
        data: bool
    },
    #[serde(rename = "pointer")]
    Pointer {
        data: String,
        #[serde(rename = "type")]
        data_type: String,
    },
    #[serde(rename = "string")]
    String {
        data: String
    },
    #[serde(rename = "struct")]
    Struct {
        // TODO: Struct parsing
        members: Vec<Value>
    },
    #[serde(rename = "array")]
    Array {
        // TODO: Array parsing
        elements: Vec<Value>
    },
    #[serde(rename = "float")]
    Float {
        binary: String,
        data: String,
        width: usize
    }
}

impl ValueInfo {
    fn parse_as_int(&self) -> Result<i32, TraceError> {
        if let ValueInfo::Integer{data_type, data, ..} = self {
            if data_type != "int" {
                return Err(JBMCLogError("Unsupported data type"));
            }
            let int_value = data.parse::<i32>()
                .or(Err(JBMCLogError("Unable to parse int data")))?;
            Ok(int_value)
        } else {
            Err(JBMCLogError("Unsupported value type"))
        }
    }

}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum SourceLocationInfo {
    Empty {},
    Location {
        file: String,
        function: String,
        line: String,
        #[serde(rename = "bytecodeIndex")]
        bytecode_index: Option<String>
    }
}

impl Default for SourceLocationInfo {
    fn default() -> Self {
        SourceLocationInfo::Empty {}
    }
}

pub struct LogAnalyzer {
    c_e_s: Vec<Vec<i32>>,
    traced_functions: HashSet<String>,
    traces: Vec<FuncLog>
}

// For storing a function call record that has not returned yet
struct FuncCallRecord<'s> {
    args: Vec<Value>,
    func: &'s str,
}

impl<'s> FuncCallRecord<'s>  {
    fn new(name: &'s str) -> Self {
        Self {
            args: vec![],
            func: name.as_ref()
        }
    }
    
    fn into_func_log(self, ret_val: i32) -> FuncLog {
        FuncLog {
            func: self.func.to_string(),
            args: self.args.into_iter().map(|v| Value::from(v)).collect(),
            rtn: Value::from(ret_val)
        }
    }
}

impl LogAnalyzer {
    pub fn new<I, S>(func_names: I) -> Self 
        where I: IntoIterator<Item=S>, S: AsRef<str>{
        LogAnalyzer {
            c_e_s: vec![],
            traced_functions: func_names.into_iter().map(|s| s.as_ref().to_string()).collect(),
            traces: vec![]
        }
    }

    pub fn get_c_e_s(&self) -> &Vec<Vec<i32>> {&self.c_e_s}
    pub fn get_traces(&self) -> &Vec<FuncLog> {&self.traces}

    fn analyze_traces<'l>(&mut self, traces: &'l Vec<VerifyTrace>) -> Result<(), TraceError> {
        let mut it = traces.iter();
        // For parsing input (C.E.s)
        let mut current_input: Vec<i32> = vec![];
        // Keep track of whether input points are already written
        // to avoid overriding
        let mut filled_input: Vec<bool> = vec![];
        // For parsing function trace logs
        let mut func_call_stack: Vec<FuncCallRecord> = vec![];
        // Last retrieved return value
        let mut last_return: Option<i32> = None;
        while let Some(trace) = it.next() {
            match trace {
                input @ VerifyTrace::Input{..} => {
                    let (index, value) = input.parse_as_input()?;
                    if current_input.len() < index + 1 {
                        current_input.extend(vec![0 ; index + 1 - current_input.len()]);
                        filled_input.extend(vec![false ; index + 1 - filled_input.len()]);
                    }
                    if *filled_input.get(index).expect("Length checked") {
                        return Err(JBMCLogError("Input overridden"));
                    }
                    *current_input.get_mut(index).expect("Length checked") = value;
                }
                VerifyTrace::FunctionCall{function, ..} => {
                    func_call_stack.push(FuncCallRecord::new(function.display_name.as_str()));
                }
                VerifyTrace::FunctionReturn{function, ..} => {
                    let record = func_call_stack.pop()
                        .ok_or(JBMCLogError("func_call_stack empty"))?;
                    if record.func != &function.display_name {
                        return Err(JBMCLogError("func_call_stack mismatch"));
                    }
                    // Currently we assume that traced functions are not void
                    // If we do not see a ret_val which means returning from a void function
                    // We just assume it is from an non-traced function and ignore the trace
                    if let Some(ret_val) = last_return.take() {
                        if self.traced_functions.contains(&function.display_name) {
                            self.traces.push(record.into_func_log(ret_val));
                        }
                    }
                }
                trace @ VerifyTrace::Assignment{
                    assignment_type: AssignmentTypeInfo::ActualParameter,
                    ..
                } => {
                    // Skip failed param assignment parses instead of returning error
                    // since unsupported types in un-traced functions will error out
                    if let Ok((index, value)) = trace.parse_as_param_assign() {
                        let record = func_call_stack.last_mut()
                            .ok_or(JBMCLogError("func_call_stack empty"))?;
                        if record.args.len() < index + 1 {
                            record.args.extend(vec![Value::Null; index + 1 - record.args.len()]);
                        }
                        *record.args.get_mut(index).expect("Length checked") = Value::from(value);
                    } else {
                        // On parse failure, Do nothing for now
                        // FIXME: It might be better to pass a dummy value, but need to figure out
                        // index to store dummy
                    }
                },
                trace @ VerifyTrace::Assignment{
                    assignment_type: AssignmentTypeInfo::Variable,
                    ref lhs,
                    ..
                } if lhs.ends_with("#return_value") => {
                    // Skip failed var assignment parses instead of returning error
                    // since unsupported types in un-traced functions will error out
                    if let Ok((_, value)) = trace.parse_as_var_assign() {
                        last_return = Some(value);
                    } else {
                        // On parse failure, use 0 as dummy value for last_return
                        // This is to ensure FunctionReturn record parsing won't fail out
                        // for un-traced functions with unsupported types
                        // FIXME: Use a 3-value enum for last_return (None, NotSupported, Value)
                        last_return = Some(0);
                    }
                },
                _ => {continue;}
            }
        }
        self.c_e_s.push(current_input);
        Ok(())
    }

    fn analyze_results<'l>(&mut self, results: &'l Vec<VerifyResult>) -> Result<(), TraceError> {
        for result in results {
            if let StatusInfo::Success = result.status {
                continue;
            }
            let unwind_prop_regex = Regex::new(r"^.*(\.unwind\.\d+$|\.recursion)$").expect("Hardcoded regex");
            if unwind_prop_regex.is_match(&result.property) {
                return Err(TraceError::JBMCUnwindError(result.property.clone()));
            }
            self.analyze_traces(&result.trace)?;
        }
        Ok(())
    }

    pub fn analyze_logs<'l>(&mut self, logs: &'l VerifyLogs) -> Result<bool, TraceError> {
        // Clear internal storage
        self.c_e_s.clear();
        self.traces.clear();
        for msg in logs.iter().rev() {
            match msg {
                LogItem::CProverStatus {status} => {
                    if let StatusInfo::Success = status {
                        return Ok(true);
                    }
                }
                LogItem::Result {result} => {
                    self.analyze_results(result)?;
                }
                _ => {continue;}
            }
        }
        return Ok(false);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    static JBMC_SAMPLE_OUTPUT : &'static str = include_str!("../../../tests/data/jbmc_sample_output.json");
    #[test]
    fn parses_json_output() -> Result<(), Box<dyn Error>> {
        let result : Result<VerifyLogs, serde_json::Error> = serde_json::from_str(JBMC_SAMPLE_OUTPUT);
        if let Ok(content) = result {
            println!("{:?}", content);
        } else if let Err(error) = result {
            println!("{:?}", error);
            return Err(Box::new(error));
        }
        Ok(())
    }

    static JBMC_SAMPLE_TRACES : &'static str = include_str!("../../../tests/data/jbmc_sample_traces.json");
    #[test]
    fn parses_json_traces() -> Result<(), Box<dyn Error>> {
        let result : Result<Vec<VerifyTrace>, serde_json::Error> = serde_json::from_str(JBMC_SAMPLE_TRACES);
        if let Ok(content) = result {
            println!("{:?}", content);
        } else if let Err(error) = result {
            println!("{:?}", error);
            return Err(Box::new(error));
        }
        Ok(())
    }

    #[test]
    fn extracts_input() -> Result<(), Box<dyn Error>> {
        let logs: VerifyLogs = serde_json::from_str(JBMC_SAMPLE_OUTPUT)?;
        let func_sigs = vec!["SimpleTest.main(int)".to_string()];
        let mut analyzer = LogAnalyzer::new(func_sigs);
        analyzer.analyze_logs(&logs)?;
        assert_eq!(analyzer.c_e_s, vec![vec![0 as i32]]);
        Ok(())
    }

    static JBMC_TEST_SIMPLE_RETURN : &'static str = include_str!("../../../tests/data/jbmc_test_simple_return.json");
    #[test]
    fn extracts_trace() -> Result<(), Box<dyn Error>> {
        let logs: VerifyLogs = serde_json::from_str(JBMC_TEST_SIMPLE_RETURN)?;
        let func_sigs = vec!["Library.add(int, int)".to_string()];
        let mut analyzer = LogAnalyzer::new(func_sigs);
        analyzer.analyze_logs(&logs)?;
        // TODO: assertions for traces
        println!("{:?}", analyzer.traces);
        Ok(())
    }

    static JBMC_UNWINDING_ERROR: &'static str = include_str!("../../../tests/data/jbmc_unwinding_error.json");
    #[test]
    fn errors_on_unwinding_failure() -> Result<(), Box<dyn Error>> {
        let logs: VerifyLogs = serde_json::from_str(JBMC_UNWINDING_ERROR)?;
        let mut analyzer = LogAnalyzer::new(Vec::<String>::new());
        let result = analyzer.analyze_logs(&logs);
        println!("{:?}", result);
        assert!(matches!(result, Err(TraceError::JBMCUnwindError{..})));
        Ok(())
    }

    static JBMC_UNWINDING_TRACES: &'static str = include_str!("../../../tests/data/jbmc_unwinding_traces.json");
    #[test]
    fn parses_unwinding_traces() -> Result<(), Box<dyn Error>> {
        let result : Result<Vec<VerifyTrace>, serde_json::Error> = serde_json::from_str(JBMC_UNWINDING_TRACES);
        if let Ok(content) = result {
            println!("{:?}", content);
        } else if let Err(error) = result {
            println!("{:?}", error);
            return Err(Box::new(error));
        }
        Ok(())
    }

    // FIXME: Wrong sqrt implementation used for this regression test log, though still
    // good for regression test, might be bette to use a correct one
    static JBMC_REGRESS_001: &'static str = include_str!("../../../tests/data/jbmc_parser_regress_001/full.json");
    #[test]
    fn parses_regress_001_full() -> Result<(), Box<dyn Error>> {
        let logs : VerifyLogs = serde_json::from_str(JBMC_REGRESS_001)?;
        let mut analyzer = LogAnalyzer::new(vec!["Library.sqrt(int)".to_string()]);
        analyzer.analyze_logs(&logs)?;
        println!("{:#?}", analyzer.get_c_e_s());
        println!("{:#?}", analyzer.get_traces());
        Ok(())
    }

    static JBMC_REGRESS_001_TRACE1: &'static str = include_str!("../../../tests/data/jbmc_parser_regress_001/trace1.json");
    #[test]
    fn parses_regress_001_trace1() -> Result<(), Box<dyn Error>> {
        let result : Result<Vec<VerifyTrace>, serde_json::Error> = serde_json::from_str(JBMC_REGRESS_001_TRACE1);
        if let Ok(content) = result {
            println!("{:#?}", content);
        } else if let Err(error) = result {
            println!("{:#?}", error);
            return Err(Box::new(error));
        }
        Ok(())
    }

    static JBMC_REGRESS_001_TRACE2: &'static str = include_str!("../../../tests/data/jbmc_parser_regress_001/trace2.json");
    #[test]
    fn parses_regress_001_trace2() -> Result<(), Box<dyn Error>> {
        let result : Result<Vec<VerifyTrace>, serde_json::Error> = serde_json::from_str(JBMC_REGRESS_001_TRACE2);
        if let Ok(content) = result {
            println!("{:#?}", content);
        } else if let Err(error) = result {
            println!("{:#?}", error);
            return Err(Box::new(error));
        }
        Ok(())
    }

}