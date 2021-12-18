use serde::Serialize;
use std::io::{Seek, Write};
use super::super::cegis_record::{CEGISTimer, Error};
use super::super::cegis_state::FuncLog;
use tempfile::NamedTempFile;
use std::time::Duration;
use std::path::Path;

#[derive(Serialize)]
struct CEGISRecordEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    iter_nth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_c_e_s: Option<Vec<Vec<i32>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_traces: Option<Vec<Vec<FuncLog>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    synthesis_wall_time: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verification_wall_time: Option<f32>,
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
    total_iter: usize
}

pub struct CEGISRecorder {
    record: CEGISRecord,
    iter_nth: Option<usize>,
    new_c_e_s: Option<Vec<Vec<i32>>>,
    new_traces: Option<Vec<Vec<FuncLog>>>,
    total_clock: CEGISTimer,
    synthesis_clock: CEGISTimer,
    verification_clock: CEGISTimer,
    current_total_cost: Option<Duration>,
    current_synthesis_cost: Option<Duration>,
    current_verification_cost: Option<Duration>,
    ephemeral_record: Option<NamedTempFile>
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
                total_iter: 0,
            },
            iter_nth: None,
            new_c_e_s: None,
            new_traces: None,
            total_clock: CEGISTimer::new(),
            synthesis_clock: CEGISTimer::new(),
            verification_clock: CEGISTimer::new(),
            current_total_cost: None,
            current_synthesis_cost: None,
            current_verification_cost: None,
            ephemeral_record: ephemeral_record
        }
    }

    pub fn set_iter_nth(&mut self, iter_nth: usize) {
        self.iter_nth = Some(iter_nth);
    }

    pub fn set_new_c_e_s(&mut self, new_c_e_s: &Vec<Vec<i32>>) {
        self.new_c_e_s = Some(new_c_e_s.clone());
    }

    pub fn set_new_traces(&mut self, new_traces: &Vec<Vec<FuncLog>>) {
        self.new_traces = Some(new_traces.clone());
    }

    pub fn set_total_iter(&mut self, total_iter: usize) {
        self.record.total_iter = total_iter;
    }

    pub fn set_solved(&mut self, solved: bool) {
        self.record.solved = solved;
    }

    pub fn commit(&mut self) {
        self.record.entries.push(CEGISRecordEntry{
            iter_nth : self.iter_nth.take(),
            new_c_e_s: self.new_c_e_s.take(),
            new_traces: self.new_traces.take(),
            synthesis_wall_time: self.current_synthesis_cost.take().map(|d| d.as_secs_f32()),
            verification_wall_time: self.current_verification_cost.take().map(|d| d.as_secs_f32()),
            total_wall_time: self.current_total_cost.take().map(|d| d.as_secs_f32())
        });
    }

    pub fn start_total_clock(&mut self) {self.total_clock.start();}

    pub fn start_synthesis(&mut self) {self.synthesis_clock.start();}
  
    pub fn start_verification(&mut self) {self.verification_clock.start();}

    pub fn stop_synthesis(&mut self) {
        self.current_synthesis_cost = Some(self.synthesis_clock.stop());
    }

    pub fn stop_verification(&mut self) {
        self.current_verification_cost = Some(self.verification_clock.stop());
    }

    pub fn step_iteration(&mut self) {
        self.current_total_cost = Some(self.total_clock.step());
    }

    pub fn commit_time(&mut self) -> Option<()> {
        self.total_clock.stop();
        self.synthesis_clock.stop();
        self.verification_clock.stop();
        self.record.total_wall_time = self.total_clock.total_elapsed().as_secs_f32();
        self.record.total_synthesis_time = self.synthesis_clock.total_elapsed().as_secs_f32();
        self.record.total_verification_time = self.verification_clock.total_elapsed().as_secs_f32();
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
}