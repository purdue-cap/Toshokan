use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::iter::repeat;
use serde_json::Value;
use std::hash::{Hash, Hasher};
use super::FuncConfig;

#[derive(Serialize)]
pub struct CEGISStateParams {
    pub logs: HashMap<String, Vec<FuncLog>>,
    pub n_unknowns: usize,
    pub c_e_s: Vec<Vec<isize>>,
    pub holes: HashMap<String, isize>,
    pub verify_points: Vec<Vec<isize>>,
    pub hist_cap: usize,
    pub func_hist_codes: HashMap<String, usize>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "meta")]
pub enum TraceLog {
    TestStart,
    TestEnd,
    TestAFE,
    FuncCall(FuncLog)
}

// TODO: Need to remove possible @address fields (and other ephemeral fields) from the log
// when doing Hash/PartialEq/Eq on the logs to avoid duplicate logs being encoded
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct FuncLog {
    pub args: Vec<Value>,
    pub rtn: Value,
    pub func: String
}

impl Hash for FuncLog {
    fn hash<H: Hasher>(&self, state:&mut H) {
        for value in self.args.iter() {
            value.to_string().hash(state);
        }
        self.rtn.to_string().hash(state);
        self.func.hash(state);
    }
}

fn remove_key_from_json(val: &mut Value, key: &str) {
    if let Some(obj) = val.as_object_mut() {
        obj.remove(key);
        for (_name, value) in obj.iter_mut() {
            remove_key_from_json(value, key);
        }
    }
}

// Use @hashcode field if it is present
fn encode_json_to_i64(val: &Value) -> Option<i64> {
    if let Some(obj) = val.as_object() {
        obj.get("@hashcode").and_then(|code_v| code_v.as_i64())
    } else {
        val.as_i64()
    }
}

impl FuncLog {
    pub fn remove_key_from_data(&mut self, key: &str) {
        self.args.iter_mut().for_each(|arg| remove_key_from_json(arg, key));
        remove_key_from_json(&mut self.rtn, key);
    }
    // Remove all non-atomic @ fields to atomize the Log
    pub fn atomize(&mut self) {
        self.remove_key_from_data("@address")
    }
}

pub struct CEGISState {
    // Param for use with template engines, generate on demand
    params: Option<CEGISStateParams>,
    // Cache for processed log map
    log_map_cache: Option<HashMap<String, Vec<FuncLog>>>,
    // Max history length currently logged
    max_hist_length: usize,
    // Max log length currently encountered
    max_log_length: usize,
    // Configurations
    // Function configurations, should preserve order since history encoding scheme is dependent on it
    func_config: Vec<(String, FuncConfig)>,
    // Function Configurations lookup map, for quicker access
    func_config_lookup: HashMap<String, FuncConfig>,
    // Map from function name to their code in history, for easy access, initialized and never changed
    func_hist_codes: HashMap<String, usize>,
    n_input: usize,
    n_unknowns: usize,
    // States
    c_e_set: HashSet<Vec<isize>>,
    v_p_set: HashSet<Vec<isize>>,
    logs: Vec<TraceLog>,
    holes: HashMap<String, isize>,
    iter_count: usize
}

fn point_set_transpose(point_set: &HashSet<Vec<isize>>) -> Option<Vec<Vec<isize>>> {
    if point_set.is_empty() {
        Some(vec![])
    } else {
        let mut result = vec![];
        for point in point_set.iter() {
            if result.is_empty() {
                result.extend(repeat(vec![]).take(point.len()));
            }
            if result.len() != point.len() {
                return None;
            }
            for (i, v) in point.iter().enumerate() {
                result.get_mut(i)?.push(*v);
            }
        }
        Some(result)
    }
}


impl CEGISState {
    pub fn new(func_config: Vec<(String, FuncConfig)>, n_input: usize, n_unknowns: usize) -> Self {
        CEGISState {
            params: None,
            log_map_cache: None,
            max_hist_length: 0,
            max_log_length: 0,
            func_hist_codes: func_config.iter().enumerate().map(|(i, (name, _))| (name.clone(), i + 1)).collect(),
            func_config_lookup: func_config.iter().cloned().collect(),
            func_config: func_config,
            n_input: n_input,
            n_unknowns: n_unknowns,
            c_e_set: HashSet::new(),
            v_p_set: HashSet::new(),
            logs: vec![],
            holes: HashMap::new(),
            iter_count: 0
        }
    }

    pub fn update_params(&mut self) -> Option<()> {
        if self.log_map_cache.is_none() {
            self.log_map_cache = Some(self.unpack_logs()?);
        }
        self.params = Some(CEGISStateParams {
            logs: self.log_map_cache.as_ref().unwrap().clone(),
            n_unknowns: self.n_unknowns,
            c_e_s: point_set_transpose(&self.c_e_set)?,
            holes: self.holes.clone(),
            verify_points: point_set_transpose(&self.v_p_set)?,
            hist_cap: self.max_hist_length + self.n_unknowns,
            func_hist_codes: self.func_hist_codes.clone()
        });
        Some(())
    }

    // Builds log maps for each function that are ready to be encoded by template engine
    fn unpack_logs(&mut self) -> Option<HashMap<String, Vec<FuncLog>>> {
        let mut pure_logs: HashMap<String, HashSet<FuncLog>> = HashMap::new();
        let mut provisioned_logs: HashSet<Vec<FuncLog>> = HashSet::new();

        let mut current_logs: Option<HashMap<usize, Vec<FuncLog>>> = None;
        for log in self.logs.iter() {
            match log {
                TraceLog::TestStart => {
                    current_logs = Some(HashMap::new());
                },
                TraceLog::FuncCall(ref func_log) => {
                    let config = self.func_config_lookup.get(&func_log.func)?;
                    match config {
                        FuncConfig::Pure{args: _} => {
                            if !pure_logs.contains_key(&func_log.func) {
                                pure_logs.insert(func_log.func.clone(), HashSet::new());
                            }
                            let mut atomic_func_log = func_log.clone();
                            atomic_func_log.atomize();
                            pure_logs.get_mut(&func_log.func).expect("Should ensured key").insert(atomic_func_log);
                        },
                        FuncConfig::NonPure{args: _, state_arg_idx} | FuncConfig::StateQuery{args: _, state_arg_idx} => {
                            let address = func_log.args.get(*state_arg_idx)?.as_object()?.get("@address")?.as_u64()? as usize;
                            if !current_logs.as_ref()?.contains_key(&address) {
                                current_logs.as_mut()?.insert(address, vec![]);
                            }
                            let mut atomic_func_log = func_log.clone();
                            atomic_func_log.atomize();
                            current_logs.as_mut()?.get_mut(&address).expect("Should ensured key").push(atomic_func_log);
                        },
                        FuncConfig::Init{args: _} => {
                            let address = func_log.rtn.as_object()?.get("@address")?.as_u64()? as usize;
                            if !current_logs.as_ref()?.contains_key(&address) {
                                current_logs.as_mut()?.insert(address, vec![]);
                            }
                            let mut atomic_func_log = func_log.clone();
                            atomic_func_log.atomize();
                            current_logs.as_mut()?.get_mut(&address).expect("Should ensured key").push(atomic_func_log);
                        }
                    }

                },
                TraceLog::TestEnd | TraceLog::TestAFE => {
                    for (_key, value) in current_logs.take()?.into_iter() {
                        provisioned_logs.insert(value);
                    }
                }
            }
        }

        // Take any remaining (due to timeout, crash, etc.) logs in
        if let Some(remaining) = current_logs {
            for (_key, value) in remaining.into_iter() {
                provisioned_logs.insert(value);
            }
        }

        let mut log_map: HashMap<String, Vec<FuncLog>> = self.func_config.iter().map(|(k, _)| (k.clone(), vec![])).collect();
        log_map.extend(pure_logs.into_iter().map(|(k, v)| (k, v.into_iter().collect())));

        let mut non_pure_logs: HashMap<String, HashSet<FuncLog>> = HashMap::new();
        for hist in provisioned_logs.into_iter() {
            let mut current_history : LinkedList<i64> = LinkedList::new();
            for entry in hist.into_iter() {
                match &self.func_config_lookup.get(&entry.func)? {
                    FuncConfig::Pure{args: _} => {
                        return None;
                    },
                    FuncConfig::NonPure{args: _, state_arg_idx} => {
                        let encoded_args = entry.args.iter().enumerate()
                            .filter(|(i, _) | i != state_arg_idx)
                            .map(|(_, v)| encode_json_to_i64(v))
                            .collect::<Option<Vec<_>>>()?;
                        current_history.extend(encoded_args.into_iter());
                        current_history.push_front(*self.func_hist_codes.get(entry.func.as_str())? as i64);
                        if !non_pure_logs.contains_key(&entry.func) {
                            non_pure_logs.insert(entry.func.clone(), HashSet::new());
                        }
                        non_pure_logs.get_mut(&entry.func).expect("Should ensured key").insert(FuncLog {
                            args: current_history.iter().map(|v| Value::from(*v)).collect(),
                            func: entry.func,
                            rtn: entry.rtn
                        });
                    },
                    FuncConfig::StateQuery{args: _, state_arg_idx} => {
                        let encoded_args = entry.args.iter().enumerate()
                            .filter(|(i, _) | i != state_arg_idx)
                            .map(|(_, v)| encode_json_to_i64(v))
                            .collect::<Option<Vec<_>>>()?;
                        let mut query_call_hist = current_history.clone();
                        query_call_hist.extend(encoded_args.into_iter());
                        if !non_pure_logs.contains_key(&entry.func) {
                            non_pure_logs.insert(entry.func.clone(), HashSet::new());
                        }
                        non_pure_logs.get_mut(&entry.func).expect("Should ensured key").insert(FuncLog {
                            args: query_call_hist.iter().map(|v| Value::from(*v)).collect(),
                            func: entry.func,
                            rtn: entry.rtn
                        });
                    }
                    FuncConfig::Init{args: _} => {
                        let encoded_args = entry.args.iter().enumerate()
                            .map(|(_, v)| encode_json_to_i64(v))
                            .collect::<Option<Vec<_>>>()?;
                        current_history.extend(encoded_args.into_iter());
                        current_history.push_front(*self.func_hist_codes.get(entry.func.as_str())? as i64);
                        if !non_pure_logs.contains_key(&entry.func) {
                            non_pure_logs.insert(entry.func.clone(), HashSet::new());
                        }
                        non_pure_logs.get_mut(&entry.func).expect("Should ensured key").insert(FuncLog {
                            args: current_history.iter().map(|v| Value::from(*v)).collect(),
                            func: entry.func,
                            rtn: entry.rtn
                        });
                    }
                }
            }
            if current_history.len() > self.max_hist_length {
                self.max_hist_length = current_history.len();
            }
        }
        println!("{:?}", non_pure_logs);

        log_map.extend(non_pure_logs.into_iter().map(|(k, v)| (k, v.into_iter().collect())));
        Some(log_map)
    }

    pub fn get_params(&self) -> Option<&CEGISStateParams> {
        self.params.as_ref()
    }

    pub fn get_func_config(&self) -> &Vec<(String, FuncConfig)> {&self.func_config}

    pub fn get_n_input(&self) -> usize {self.n_input}

    pub fn get_n_unknowns(&self) -> usize {self.n_unknowns}

    pub fn get_max_log_length(&self) -> usize {self.max_log_length}

    pub fn get_n_c_e_s(&self) -> usize {self.c_e_set.len()}

    pub fn get_c_e_set(&self) -> &HashSet<Vec<isize>> {&self.c_e_set}

    pub fn get_n_v_p_s(&self) -> usize {self.v_p_set.len()}

    pub fn get_v_p_set(&self) -> &HashSet<Vec<isize>> {&self.v_p_set}

    pub fn get_logs(&self) -> &Vec<TraceLog> {&self.logs}

    pub fn get_n_logs(&self) -> usize {self.logs.len()}

    pub fn get_holes(&self) -> &HashMap<String, isize> {&self.holes}

    pub fn get_iter_count(&self) -> usize {self.iter_count}

    pub fn set_h_names(&mut self, h_names: HashSet<String>) {
        self.holes = h_names.iter().map(|name| (name.clone(), 0)).collect();
        self.params = None;
    }

    pub fn get_h_names(&self) -> HashSet<String> {self.holes.keys().cloned().collect()}

    pub fn incr_iteration(&mut self) {self.iter_count += 1}

    pub fn update_hole<S: AsRef<str>>(&mut self, h_name: S, value: isize) -> Option<isize> {
        self.params = None;
        self.holes.insert(h_name.as_ref().to_string(), value)
    }

    pub fn update_holes<S: AsRef<str>>(&mut self, holes: HashMap<S, isize>) -> Option<()> {
        self.params = None;
        for (k, v) in holes.into_iter() {
            self.update_hole(k, v)?;
        }
        Some(())
    }

    pub fn update_n_unknowns(&mut self, new_unknowns: usize) {
        self.n_unknowns = new_unknowns;
        self.params = None;
    }

    pub fn add_verify_point(&mut self, inputs: Vec<isize>) -> Option<()> {
        if self.v_p_set.contains(&inputs) {
            return Some(());
        }
        self.v_p_set.insert(inputs);
        self.params = None;
        Some(())
    }

    pub fn add_log(&mut self, log: TraceLog) {
        self.logs.push(log);
        self.params = None;
        self.log_map_cache = None;
    }

    pub fn add_c_e(&mut self, c_e: Vec<isize>) -> Option<()> {
        if self.c_e_set.contains(&c_e) {
            return Some(());
        }
        self.c_e_set.insert(c_e);
        self.params = None;
        Some(())
    }
}