use serde::Serialize;
use crate::cegis::FuncLog;
use super::super::cegis_state::point_set_transpose;
use std::collections::{HashMap, HashSet, LinkedList};
use std::path::PathBuf;
use std::convert::TryFrom;
use super::CEGISConfigParams;

#[derive(Serialize, Debug)]
pub struct CEGISStateParams {
    pub logs: HashMap<String, Vec<FuncLog>>,
    pub n_unknowns: usize,
    pub c_e_s: Vec<Vec<i32>>,
    pub hist_cap: usize,
    pub func_hist_codes: HashMap<String, usize>
}

pub struct CEGISState {
    // Param for use with template engines, generate on demand
    params: Option<CEGISStateParams>,
    // Cache for processed log map
    log_map_cache: Option<HashMap<String, Vec<FuncLog>>>,
    // States
    n_unknowns: usize,
    iter_count: usize,
    max_hist_size: usize,
    hist_cap_padding: usize,
    lib_funcs: Vec<String>,
    func_hist_codes: HashMap<String, usize>,
    pub c_e_set: HashSet<Vec<i32>>,
    pub logs: Vec<Vec<FuncLog>>,
    pub current_cand: Vec<PathBuf>
}

impl CEGISState {
    pub fn new(config_params: &CEGISConfigParams) -> Self {
        Self {
            params: None,
            log_map_cache: None,
            c_e_set: HashSet::new(),
            n_unknowns: config_params.n_unknowns,
            max_hist_size: 0,
            hist_cap_padding: config_params.hist_cap_padding,
            lib_funcs: config_params.lib_funcs.clone(),
            // We offset all func codes by 1, in order to avoid using code 0
            // which may conflict with default values in history arrays
            func_hist_codes: config_params.lib_funcs.iter().enumerate()
                .map(|(i, n)| (n.clone(), i+1)).collect(),
            logs: vec![],
            iter_count: 0,
            current_cand: vec![]
        }
    }

    pub fn get_params(&self) -> Option<&CEGISStateParams> {
        self.params.as_ref()
    }

    pub fn take_params(&mut self) -> Option<CEGISStateParams> {
        self.params.take()
    }

    pub fn update_params(&mut self) -> Option<()> {
        if self.log_map_cache.is_none() {
            self.log_map_cache = Some(self.unpack_logs()?);
        }
        self.params = Some(CEGISStateParams {
            logs: self.log_map_cache.take().unwrap(),
            n_unknowns: self.n_unknowns,
            c_e_s: point_set_transpose(&self.c_e_set)?,
            hist_cap: self.max_hist_size + self.hist_cap_padding,
            func_hist_codes: self.func_hist_codes.clone()
        });
        Some(())
    }

    // Encode history into integer array
    fn encode_method_hist(&self, hist: &Vec<FuncLog>) -> Option<Vec<serde_json::Value>> {
        let mut current_encoded : LinkedList<i64> = LinkedList::new();
        for log in hist.iter() {
            // 1st arg is always Null now due to "this" arg, skip it
            let mut args_encoded = log.args.iter().skip(1)
                // For now, only do integer arguments
                // TODO: support other types here
                // TODO: support objects here
                .map(|v| v.as_i64()).collect::<Option<LinkedList<i64>>>()?;
            current_encoded.append(&mut args_encoded);
            current_encoded.push_front(i64::try_from(*self.func_hist_codes.get(&log.func)?).ok()?);
        }
        Some(current_encoded.into_iter().map(|v| serde_json::Value::from(v)).collect())
    }

    // We assume all non-method functions are pure functions
    // In the future we could distinguish between:
    // 1. pure functions: no side effect, non-method
    // 2. pure method: side effect limited to this
    // 3. stateful functions: global side effect, non-method
    // 4. stateful methods: side effect unlimited
    // Effectively now we only have 1 and 2
    // TODO: support type 3 and 4 here
    fn unpack_logs(&mut self) -> Option<HashMap<String, Vec<FuncLog>>> {

        // log matrix is indexed by (func_code - 1) due to the offset we put into function codes
        let mut log_matrix: Vec<HashSet<FuncLog>> = vec![HashSet::new(); self.func_hist_codes.len()];
        for log_run in self.logs.iter(){
            let mut current_obj_stack: HashMap<String, Vec<FuncLog>> = HashMap::new();
            for log in log_run.iter(){
                // function index is func_code - 1
                let func_idx = *self.func_hist_codes.get(&log.func)? - 1;
                if let Some(ref obj) = log.this {
                    // Pure method
                    // Put into current obj stack
                    let current_hist = current_obj_stack.entry(obj.clone()).or_default();
                    // Put this call into history
                    current_hist.push(log.clone());
                    // Encode current history (after this call) into int array
                    let hist_encoded = self.encode_method_hist(current_hist)?;
                    // Ensure history array length
                    if hist_encoded.len() > self.max_hist_size {
                        self.max_hist_size = hist_encoded.len();
                    }
                    // Insert encoded func log
                    log_matrix.get_mut(func_idx)?.insert(
                        FuncLog {
                            args: hist_encoded,
                            func: log.func.clone(),
                            this: None, // Remove this arg, to make the log atomic
                            rtn: log.rtn.clone()
                        }
                    );
                } else {
                    // Pure function
                    if log.rtn.is_none() {
                        // Ignore pure void functions (effectively noop)
                        continue;
                    }
                    log_matrix.get_mut(func_idx)?.insert(log.clone());
                }
            }
        }
        Some(log_matrix.into_iter().enumerate()
            .map(|(i, v)| Some((self.lib_funcs.get(i)?.clone(), v.into_iter().collect())) )
            .collect::<Option<_>>()?)
    }

    pub fn get_iter_count(&self) -> usize {self.iter_count}
    pub fn incr_iteration(&mut self) {self.iter_count += 1}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use crate::backend::java::{JBMCLogs, JBMCLogAnalyzer};
    use crate::cegis::java::CEGISConfigParams;

    static JBMC_OBJECT_SAMPLE : &'static str = include_str!("../../../tests/data/jbmc_object_sample.json");
    #[test]
    fn unpacks_java_logs() -> Result<(), Box<dyn Error>> {
        let logs: JBMCLogs = serde_json::from_str(JBMC_OBJECT_SAMPLE)?;
        let func_sigs = vec!["Adder(int)".to_string(), "Adder.add(int)".to_string()];
        let mut analyzer = JBMCLogAnalyzer::new(func_sigs);
        analyzer.analyze_logs(&logs)?;
        let mut params = CEGISConfigParams::test_fixture_dummy();
        params.lib_funcs.push("Adder(int)".into());
        params.lib_funcs.push("Adder.add(int)".into());
        let mut state = CEGISState::new(&params);
        state.logs.extend(analyzer.get_traces().iter().cloned());
        state.update_params();
        // TODO: add assertions for params
        println!("{:#?}", state.get_params());
        Ok(())
    }


    static JBMC_REGRESS_002: &'static str = include_str!("../../../tests/data/jbmc_parser_regress_002/full.json");
    #[test]
    fn unpacks_regress_002_logs() -> Result<(), Box<dyn Error>> {
        let logs: JBMCLogs = serde_json::from_str(JBMC_REGRESS_002)?;
        let func_sigs = vec![
            "Stack()".to_string(),
            "Stack.push(int)".to_string(),
            "Stack.pop()".to_string(),
            ];
        let mut analyzer = JBMCLogAnalyzer::new(func_sigs);
        analyzer.analyze_logs(&logs)?;
        let mut params = CEGISConfigParams::test_fixture_dummy();
        params.lib_funcs.push("Stack()".into());
        params.lib_funcs.push("Stack.push(int)".into());
        params.lib_funcs.push("Stack.pop()".into());
        let mut state = CEGISState::new(&params);
        state.logs.extend(analyzer.get_traces().iter().cloned());
        state.update_params();
        // TODO: add assertions for params
        println!("{:#?}", state.get_params());
        Ok(())
    }
}