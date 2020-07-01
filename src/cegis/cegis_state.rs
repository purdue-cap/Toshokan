use serde::{Serialize, Deserialize};
use std::iter::repeat;
use std::collections::HashSet;
use std::collections::HashMap;
use serde_json::Value;
use std::hash::{Hash, Hasher};

#[derive(Serialize)]
pub struct CEGISStateParams {
    pub logs: HashMap<String, Vec<TraceLog>>,
    pub n_unknowns: usize,
    pub c_e_s: Vec<Vec<isize>>,
    pub holes: HashMap<String, isize>,
    pub verify_points: Vec<Vec<isize>>
}

// TODO: Need to remove possible @address fields (and other ephemeral fields) from the log
// when doing Hash/PartialEq/Eq on the logs to avoid duplicate logs being encoded
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TraceLog {
    pub args: Vec<Value>,
    pub rtn: Value,
    pub func: String
}

impl Hash for TraceLog {
    fn hash<H: Hasher>(&self, state:&mut H) {
        for value in self.args.iter() {
            value.to_string().hash(state);
        }
        self.rtn.to_string().hash(state);
        self.func.hash(state);
    }
}

pub struct CEGISState {
    params: CEGISStateParams,
    func_config: HashMap<String, usize>,
    n_input: usize,
    c_e_set: HashSet<Vec<isize>>,
    v_p_set: HashSet<Vec<isize>>,
    log_set: LibFuncLog,
    iter_count: usize,
    h_names: HashSet<String>
}

enum LibFuncLog {
    Pure(HashMap<String, HashSet<TraceLog>>),
    NonPure
}

impl CEGISState {
    pub fn new(func_config: HashMap<String, usize>, n_input: usize, n_unknowns: usize, pure_function: bool) -> Self {
        CEGISState {
            params: CEGISStateParams {
                logs: func_config.keys().map(|k| (k.clone(), vec![])).collect(),
                n_unknowns: n_unknowns,
                c_e_s: repeat(Vec::<isize>::new()).take(n_input).collect(),
                holes: HashMap::new(),
                verify_points: repeat(Vec::<isize>::new()).take(n_input).collect()
            },
            log_set: if pure_function {LibFuncLog::Pure(func_config.keys().map(|k| (k.clone(), HashSet::new())).collect())} else {LibFuncLog::NonPure},
            func_config: func_config,
            n_input: n_input,
            c_e_set: HashSet::new(),
            v_p_set: HashSet::new(),
            iter_count: 0,
            h_names: HashSet::new()
        }
    }

    pub fn get_params(&self) -> &CEGISStateParams {
        &self.params
    }

    pub fn get_func_config(&self) -> &HashMap<String, usize> {&self.func_config}

    pub fn get_n_input(&self) -> usize {self.n_input}

    pub fn get_n_c_e_s(&self) -> usize {self.c_e_set.len()}

    pub fn get_n_v_p_s(&self) -> usize {self.v_p_set.len()}

    pub fn get_iter_count(&self) -> usize {self.iter_count}

    pub fn set_h_names(&mut self, h_names: HashSet<String>) {
        self.params.holes = h_names.iter().map(|name| (name.clone(), 0)).collect();
        self.h_names = h_names;
    }

    pub fn get_h_names(&self) -> &HashSet<String> {&self.h_names}

    pub fn incr_iteration(&mut self) {self.iter_count += 1}

    pub fn update_hole<S: AsRef<str>>(&mut self, h_name: S, value: isize) -> Option<isize> {
        self.params.holes.insert(h_name.as_ref().to_string(), value)
    }

    pub fn update_holes<S: AsRef<str>>(&mut self, holes: HashMap<S, isize>) -> Option<()> {
        for (k, v) in holes.into_iter() {
            self.update_hole(k, v)?;
        }
        Some(())
    }

    pub fn update_n_unknowns(&mut self, new_unknowns: usize) {
        self.params.n_unknowns = new_unknowns;
    }

    pub fn add_verify_point(&mut self, inputs: Vec<isize>) -> Option<()> {
        if self.v_p_set.contains(&inputs) {
            return Some(());
        }
        for (index_i, value_i) in inputs.iter().enumerate() {
            self.params.verify_points.get_mut(index_i)?.push(*value_i);
        }
        self.v_p_set.insert(inputs);
        Some(())
    }

    pub fn add_log(&mut self, log: TraceLog) -> Option<()> {
        match self.log_set {
            LibFuncLog::NonPure => {unimplemented!();},
            LibFuncLog::Pure(ref mut map) => {
                let set = map.get_mut(&log.func)?;
                if set.contains(&log) {
                    return Some(());
                }

                set.insert(log.clone());
                let vec = self.params.logs.get_mut(&log.func)?;
                vec.push(log);

                if vec.len() != set.len() {
                    return None;
                }

                Some(())
            }
        }

    }

    pub fn add_c_e(&mut self, c_e: Vec<isize>) -> Option<()> {
        if self.c_e_set.contains(&c_e) {
            return Some(());
        }
        for (index_i, value_i) in c_e.iter().enumerate() {
            self.params.c_e_s.get_mut(index_i)?.push(*value_i);
        }
        self.c_e_set.insert(c_e);
        Some(())
    }
}