use serde::Serialize;
use std::iter::repeat;

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
    n_c_e_s: usize
}

impl CEGISState {
    pub fn new(n_f_args: usize, n_input: usize, n_unknowns: usize, n_holes: usize) -> Self {
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
            n_c_e_s: 0
        }
    }

    pub fn get_params(&self) -> &CEGISStateParams {
        &self.params
    }

    pub fn get_n_f_args(&self) -> usize {self.n_f_args}

    pub fn get_n_input(&self) -> usize {self.n_input}

    pub fn get_n_holes(&self) -> usize {self.n_holes}

    pub fn get_n_c_e_s(&self) -> usize {self.n_c_e_s}

    pub fn update_hole(&mut self, index: usize, value: isize) -> Option<()> {
        *self.params.holes.get_mut(index)? = value;
        Some(())
    }

    pub fn add_log(&mut self, i: &Vec<isize>, r: isize) -> Option<()> {
        self.params.n_logs += 1;
        for (index_i, value_i) in i.iter().enumerate() {
            self.params.logs_i.get_mut(index_i)?.push(*value_i);
        }
        self.params.logs_r.push(r);
        Some(())
    }

    pub fn add_c_e(&mut self, c_e: &Vec<isize>) -> Option<()> {
        self.n_c_e_s += 1;
        for (index_i, value_i) in c_e.iter().enumerate() {
            self.params.c_e_s.get_mut(index_i)?.push(*value_i);
        }
        Some(())
    }
}