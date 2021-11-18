use serde::Serialize;
use crate::cegis::FuncLog;
use super::super::cegis_state::point_set_transpose;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use super::CEGISConfigParams;

#[derive(Serialize)]
pub struct CEGISStateParams {
    pub logs: HashMap<String, Vec<FuncLog>>,
    pub n_unknowns: usize,
    pub c_e_s: Vec<Vec<i32>>,
}

pub struct CEGISState {
    // Param for use with template engines, generate on demand
    params: Option<CEGISStateParams>,
    // Cache for processed log map
    log_map_cache: Option<HashMap<String, Vec<FuncLog>>>,
    // States
    n_unknowns: usize,
    iter_count: usize,
    pub c_e_set: HashSet<Vec<i32>>,
    pub logs: Vec<FuncLog>,
    pub current_cand: Vec<PathBuf>
}

impl CEGISState {
    pub fn new(_config_params: &CEGISConfigParams) -> Self {
        Self {
            params: None,
            log_map_cache: None,
            c_e_set: HashSet::new(),
            n_unknowns: 10,
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
        });
        Some(())
    }

    fn unpack_logs(&mut self) -> Option<HashMap<String, Vec<FuncLog>>> {
        let mut log_map: HashMap<String, HashSet<FuncLog>> = HashMap::new();
        for l in self.logs.iter() {
            if let Some(inner_set) = log_map.get_mut(&l.func) {
                inner_set.insert(l.clone());
            } else {
                log_map.insert(l.func.clone(), vec![l].into_iter().cloned().collect());
            }
        }
        Some(log_map.into_iter().map(|(s, c)| (s, c.into_iter().collect())).collect())
    }

    pub fn get_iter_count(&self) -> usize {self.iter_count}
    pub fn incr_iteration(&mut self) {self.iter_count += 1}
}