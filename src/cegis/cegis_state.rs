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
            } else {
                if result.len() != point.len() {
                    return None;
                }
                for (i, v) in point.iter().enumerate() {
                    result.get_mut(i)?.push(*v);
                }
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
        let mut log_map : HashMap<String, Vec<FuncLog>> = HashMap::new();
        for (name, config) in self.func_config.iter() {
            match config {
                FuncConfig::Pure{args: _} => {
                    // If function is pure
                    // Use a hash set to de-dup the func logs
                    let mut pure_log_set : HashSet<FuncLog> = HashSet::new();
                    for log in self.logs.iter() {
                        if let TraceLog::FuncCall(ref func_log) = log {
                            if &func_log.func == name {
                                pure_log_set.insert(func_log.clone());
                            }
                        }
                    }
                    // Collect de-duped logs to vec and insert into log map
                    if self.max_log_length < pure_log_set.len() {
                        self.max_log_length = pure_log_set.len();
                    }
                    log_map.insert(name.clone(), pure_log_set.into_iter().collect());
                },
                FuncConfig::NonPure {args: _, state_arg_idx} => {
                    // Building encodings for history
                    let mut current_history : Option<LinkedList<i64>> = None;
                    let mut hist_log_set : HashSet<FuncLog> = HashSet::new();
                    for log in self.logs.iter() {
                        match log {
                            TraceLog::TestStart => {
                                current_history = Some(LinkedList::new());
                            },
                            TraceLog::TestEnd | TraceLog::TestAFE => {
                                current_history = None;
                            }
                            TraceLog::FuncCall(ref func_log) => {
                                let encoded_args = func_log.args.iter().enumerate()
                                    .filter(|(i, _) | i != state_arg_idx)
                                    .map(|(_, v)| v.as_i64())
                                    .collect::<Option<Vec<_>>>()?;
                                current_history.as_mut()?.extend(encoded_args.into_iter());
                                current_history.as_mut()?.push_front(*self.func_hist_codes.get(name)? as i64);
                                if &func_log.func == name {
                                    hist_log_set.insert(FuncLog {
                                        args: current_history.as_ref()?.iter().map(|v| Value::from(*v)).collect(),
                                        func: func_log.func.clone(),
                                        rtn: func_log.rtn.clone()
                                    });
                                }
                                if current_history.as_ref()?.len() > self.max_hist_length {
                                    self.max_hist_length = current_history.as_ref()?.len();
                                }
                            }
                        }
                    };
                    if self.max_log_length < hist_log_set.len() {
                        self.max_log_length = hist_log_set.len();
                    }
                    log_map.insert(name.clone(), hist_log_set.into_iter().collect());
                }
            };
        }
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