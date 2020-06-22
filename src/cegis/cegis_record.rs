use serde::Serialize;
use std::io::Write;
use std::time::Instant;
use std::collections::HashMap;
use super::TraceLog;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct CEGISRecordEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    iter_nth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_c_e_s: Option<Vec<isize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_traces: Option<Vec<TraceLog>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    holes: Option<HashMap<String, isize>>

}

#[derive(Serialize)]
struct CEGISRecord {
    entries: Vec<CEGISRecordEntry>,
    solved: bool,
    wall_time: f32,
    total_iter: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_synthesis: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_verification: Option<PathBuf>
}

pub struct CEGISRecorder {
    record: CEGISRecord,
    iter_nth: Option<usize>,
    new_c_e_s: Option<Vec<isize>>,
    new_traces: Option<Vec<TraceLog>>,
    holes: Option<HashMap<String, isize>>,
    clock: Option<Instant>,
    last_synthesis: Option<PathBuf>,
    last_verification: Option<PathBuf>
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        IOError(err: std::io::Error) {
            from()
            cause(err)
            display("{}", err)
        }
        SerdeJSONError(err: serde_json::Error) {
            from()
            cause(err)
            display("{}", err)
        }
    }
}

impl CEGISRecorder {
    pub fn new() -> Self {
        CEGISRecorder {
            record: CEGISRecord {
                entries: Vec::new(),
                solved: false,
                wall_time: std::f32::NAN,
                total_iter: 0,
                last_synthesis: None,
                last_verification: None
            },
            iter_nth: None,
            new_c_e_s: None,
            new_traces: None,
            holes: None,
            clock: None,
            last_synthesis: None,
            last_verification: None
        }
    }

    pub fn set_iter_nth(&mut self, iter_nth: usize) {
        self.iter_nth = Some(iter_nth);
    }
    
    pub fn set_new_c_e_s(&mut self, new_c_e_s: &Vec<isize>) {
        self.new_c_e_s = Some(new_c_e_s.clone());
    }

    pub fn set_new_traces(&mut self, new_traces: &Vec<TraceLog>) {
        self.new_traces = Some(new_traces.clone());
    }

    pub fn set_holes(&mut self, holes: &HashMap<String, isize>) {
        self.holes = Some(holes.clone());
    }

    pub fn set_total_iter(&mut self, total_iter: usize) {
        self.record.total_iter = total_iter;
    }

    pub fn set_solved(&mut self, solved: bool) {
        self.record.solved = solved;
    }

    pub fn set_last_synthesis(&mut self, p:&Path) {self.last_synthesis = Some(p.to_path_buf())}

    pub fn set_last_verification(&mut self, p:&Path) {self.last_verification = Some(p.to_path_buf())}

    pub fn commit(&mut self) {
        self.record.entries.push(CEGISRecordEntry{
            iter_nth : self.iter_nth.take(),
            new_c_e_s: self.new_c_e_s.take(),
            new_traces: self.new_traces.take(),
            holes: self.holes.take()
        });
    }

    pub fn reset_clock(&mut self) -> () {
        self.clock = Some(Instant::now());
    }

    pub fn commit_time(&mut self) -> Option<()> {
        self.record.wall_time = self.clock?.elapsed().as_secs_f32();
        Some(())
    }

    pub fn get_time(&self) -> f32 {
        self.record.wall_time
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(&self.record)
    }

    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.record)
    }

    pub fn write_json<W: Write>(&self, w: &mut W) -> Result<(), Error>{
        w.write(self.to_json()?.as_bytes())?;
        Ok(())
    }

    pub fn write_json_pretty<W: Write>(&self, w: &mut W) -> Result<(), Error>{
        w.write(self.to_json_pretty()?.as_bytes())?;
        Ok(())
    }

    pub fn commit_last_files(&mut self) {
        self.record.last_synthesis = self.last_synthesis.clone();
        self.record.last_verification = self.last_verification.clone();
    }
}