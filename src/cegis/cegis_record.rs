use serde::Serialize;
use std::io::{Seek, Write};
use std::time::{Instant, Duration};
use std::collections::HashMap;
use super::TraceLog;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use quick_error::quick_error;

pub struct CEGISTimer {
    elapsed_duration: Duration,
    start_time: Option<Instant>
}

impl CEGISTimer {
    pub fn new() -> Self{
        CEGISTimer {
            elapsed_duration: Duration::new(0, 0),
            start_time: None
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn stop(&mut self) -> Duration {
        if let Some(start_time) = self.start_time.take() {
            let new_elapse = start_time.elapsed();
            self.elapsed_duration += new_elapse;
            new_elapse
        } else {
            Duration::new(0, 0)
        }
    }

    pub fn step(&mut self) -> Duration {
        let elapse = self.stop();
        self.start();
        elapse
    }

    pub fn total_elapsed(&self) -> Duration {self.elapsed_duration}

}

#[derive(Serialize)]
struct CEGISRecordEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    iter_nth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_c_e_s: Option<Vec<isize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_traces: Option<Vec<TraceLog>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    holes: Option<HashMap<String, isize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_timed_out: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    synthesis_seed: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pre_synthesis_seed: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    synthesis_wall_time: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pre_synthesis_wall_time: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_wall_time: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_wall_time: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_wall_time: Option<f32>
}

#[derive(Serialize)]
struct CEGISRecord {
    entries: Vec<CEGISRecordEntry>,
    solved: bool,
    total_wall_time: f32,
    total_synthesis_time: f32,
    total_verification_time: f32,
    total_trace_time: f32,
    initialization_time: f32,
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
    total_clock: CEGISTimer,
    synthesis_clock: CEGISTimer,
    verification_clock: CEGISTimer,
    trace_clock: CEGISTimer,
    current_total_cost: Option<Duration>,
    current_synthesis_cost: Option<Duration>,
    current_pre_synthesis_cost: Option<Duration>,
    current_verification_cost: Option<Duration>,
    current_trace_cost: Option<Duration>,
    initialization_cost: Option<Duration>,
    last_synthesis: Option<PathBuf>,
    last_verification: Option<PathBuf>,
    trace_timed_out: Option<bool>,
    synthesis_seed: Option<Vec<u64>>,
    verification_seed: Option<u64>,
    pre_synthesis_seed: Option<Vec<u64>>,
    ephemeral_record: Option<NamedTempFile>
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
    pub fn new(ephemeral_record: Option<NamedTempFile>) -> Self {
        CEGISRecorder {
            record: CEGISRecord {
                entries: Vec::new(),
                solved: false,
                total_wall_time: std::f32::NAN,
                total_synthesis_time: std::f32::NAN,
                total_verification_time: std::f32::NAN,
                total_trace_time: std::f32::NAN,
                initialization_time: std::f32::NAN,
                total_iter: 0,
                last_synthesis: None,
                last_verification: None
            },
            iter_nth: None,
            new_c_e_s: None,
            new_traces: None,
            holes: None,
            total_clock: CEGISTimer::new(),
            synthesis_clock: CEGISTimer::new(),
            verification_clock: CEGISTimer::new(),
            trace_clock: CEGISTimer::new(),
            current_total_cost: None,
            current_synthesis_cost: None,
            current_pre_synthesis_cost: None,
            current_verification_cost: None,
            current_trace_cost: None,
            initialization_cost: None,
            last_synthesis: None,
            last_verification: None,
            trace_timed_out: None,
            synthesis_seed: None,
            verification_seed: None,
            pre_synthesis_seed: None,
            ephemeral_record: ephemeral_record
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

    pub fn set_trace_timed_out(&mut self, trace_timed_out: bool) {
        self.trace_timed_out = Some(trace_timed_out);
    }

    pub fn add_synthesis_seed(&mut self, seed: Option<u64>) {
        if let Some(seed_value) = seed {
            match self.synthesis_seed {
                Some(ref mut seeds) => {seeds.push(seed_value)}
                None => {self.synthesis_seed = Some(vec![seed_value])}
            }
        }
    }

    pub fn set_verification_seed(&mut self, seed: Option<u64>) {
        self.verification_seed = seed;
    }

    pub fn add_pre_synthesis_seed(&mut self, seed: Option<u64>) {
        if let Some(seed_value) = seed {
            match self.pre_synthesis_seed {
                Some(ref mut seeds) => {seeds.push(seed_value)}
                None => {self.pre_synthesis_seed = Some(vec![seed_value])}
            }
        }
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
            holes: self.holes.take(),
            trace_timed_out: self.trace_timed_out.take(),
            synthesis_seed: self.synthesis_seed.take(),
            verification_seed: self.verification_seed.take(),
            pre_synthesis_seed: self.pre_synthesis_seed.take(),
            synthesis_wall_time: self.current_synthesis_cost.take().map(|d| d.as_secs_f32()),
            pre_synthesis_wall_time: self.current_pre_synthesis_cost.take().map(|d| d.as_secs_f32()),
            verification_wall_time: self.current_verification_cost.take().map(|d| d.as_secs_f32()),
            trace_wall_time: self.current_trace_cost.take().map(|d| d.as_secs_f32()),
            total_wall_time: self.current_total_cost.take().map(|d| d.as_secs_f32())
        });
    }

    pub fn start_total_clock(&mut self) {self.total_clock.start();}

    pub fn start_synthesis(&mut self) {self.synthesis_clock.start();}
  
    pub fn start_verification(&mut self) {self.verification_clock.start();}

    pub fn start_trace(&mut self) {self.trace_clock.start();}

    pub fn stop_synthesis(&mut self) {
        self.current_synthesis_cost = Some(self.synthesis_clock.stop());
    }

    pub fn stop_pre_synthesis(&mut self) {
        self.current_pre_synthesis_cost = Some(self.synthesis_clock.stop());
    }

    pub fn stop_verification(&mut self) {
        self.current_verification_cost = Some(self.verification_clock.stop());
    }

    pub fn stop_trace(&mut self) {
        self.current_trace_cost = Some(self.trace_clock.stop());
    }
    
    pub fn step_iteration(&mut self) {
        self.current_total_cost = Some(self.total_clock.step());
    }

    pub fn step_initialization(&mut self) {
        self.initialization_cost = Some(self.total_clock.step());
    }

    pub fn commit_time(&mut self) -> Option<()> {
        self.total_clock.stop();
        self.synthesis_clock.stop();
        self.verification_clock.stop();
        self.trace_clock.stop();
        self.record.total_wall_time = self.total_clock.total_elapsed().as_secs_f32();
        self.record.total_synthesis_time = self.synthesis_clock.total_elapsed().as_secs_f32();
        self.record.total_verification_time = self.verification_clock.total_elapsed().as_secs_f32();
        self.record.total_trace_time = self.trace_clock.total_elapsed().as_secs_f32();
        self.record.initialization_time = self.initialization_cost?.as_secs_f32();
        Some(())
    }

    pub fn get_time(&self) -> f32 {
        self.record.total_wall_time
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

    pub fn write_ephemeral_record(&mut self) -> Result<(), Error> {
        if self.ephemeral_record.is_some() {
            let json_record = self.to_json_pretty()?;
            let er_file = self.ephemeral_record.as_mut().unwrap().as_file_mut();
            er_file.set_len(0)?;
            er_file.seek(std::io::SeekFrom::Start(0))?;
            er_file.write(json_record.as_bytes())?;
        }
        Ok(())
    }

    pub fn get_ephemeral_record_path(&self) -> Option<&Path> {self.ephemeral_record.as_ref().map(|r| r.path())}

    pub fn commit_last_files(&mut self) {
        self.record.last_synthesis = self.last_synthesis.clone();
        self.record.last_verification = self.last_verification.clone();
    }
}