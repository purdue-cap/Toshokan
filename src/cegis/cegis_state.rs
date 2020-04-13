use serde::Serialize;
use std::iter::repeat;
use std::collections::HashSet;

#[derive(Serialize)]
pub struct CEGISStateParams {
    pub n_logs: usize,
    pub logs_i: Vec<Vec<isize>>,
    pub logs_r: Vec<isize>,
    pub n_unknowns: usize,
    pub c_e_s: Vec<Vec<isize>>,
    pub holes: Vec<isize>,
    pub verify_points: Vec<Vec<isize>>
}

pub struct CEGISState {
    params: CEGISStateParams,
    n_f_args: usize,
    n_input: usize,
    n_holes: usize,
    c_e_set: HashSet<Vec<isize>>,
    v_p_set: HashSet<Vec<isize>>,
    log_set: LibFuncLog,
    iter_count: usize
}

enum LibFuncLog {
    Pure(HashSet<(Vec<isize>, isize)>),
    NonPure
}

impl CEGISState {
    pub fn new(n_f_args: usize, n_input: usize, n_unknowns: usize, n_holes: usize, pure_function: bool) -> Self {
        CEGISState {
            params: CEGISStateParams {
                n_logs: 0,
                logs_i: repeat(Vec::<isize>::new()).take(n_f_args).collect(),
                logs_r: Vec::<isize>::new(),
                n_unknowns: n_unknowns,
                c_e_s: repeat(Vec::<isize>::new()).take(n_input).collect(),
                holes: repeat(0).take(n_holes).collect(),
                verify_points: repeat(Vec::<isize>::new()).take(n_input).collect()
            },
            n_f_args: n_f_args,
            n_input: n_input,
            n_holes: n_holes,
            c_e_set: HashSet::new(),
            v_p_set: HashSet::new(),
            log_set: if pure_function {LibFuncLog::Pure(HashSet::new())} else {LibFuncLog::NonPure},
            iter_count: 0
        }
    }

    pub fn get_params(&self) -> &CEGISStateParams {
        &self.params
    }

    pub fn get_n_f_args(&self) -> usize {self.n_f_args}

    pub fn get_n_input(&self) -> usize {self.n_input}

    pub fn get_n_holes(&self) -> usize {self.n_holes}

    pub fn get_n_c_e_s(&self) -> usize {self.c_e_set.len()}

    pub fn get_n_v_p_s(&self) -> usize {self.v_p_set.len()}

    pub fn get_iter_count(&self) -> usize {self.iter_count}

    pub fn incr_iteration(&mut self) {self.iter_count += 1}

    pub fn update_hole(&mut self, index: usize, value: isize) -> Option<()> {
        *self.params.holes.get_mut(index)? = value;
        Some(())
    }

    pub fn update_holes(&mut self, holes: &[isize]) -> Option<()> {
        if holes.len() != self.n_holes {
            None
        } else {
            for (index_i, value_i) in holes.iter().enumerate() {
                self.update_hole(index_i, *value_i)?;
            }
            Some(())
        }

    }

    pub fn update_n_unknowns(&mut self, new_unknowns: usize) {
        self.params.n_logs = new_unknowns;
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

    pub fn add_log(&mut self, i: Vec<isize>, r: isize) -> Option<()> {
        match self.log_set {
            LibFuncLog::NonPure => {unimplemented!();},
            LibFuncLog::Pure(ref mut set) => {
                let log_pair = (i, r);
                if set.contains(&log_pair) {
                    return Some(());
                }

                for (index_i, value_i) in log_pair.0.iter().enumerate() {
                    self.params.logs_i.get_mut(index_i)?.push(*value_i);
                }
                self.params.logs_r.push(log_pair.1);
                set.insert(log_pair);
                self.params.n_logs = set.len();

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