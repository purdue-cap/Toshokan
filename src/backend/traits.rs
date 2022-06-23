use crate::cegis::FuncLog;

pub trait ProduceTrace {
}

pub trait AnalyzeVerifierLog {
    type Error;
    fn get_c_e_s(&self) -> &Vec<Vec<i32>>;
    fn get_unwind_err_loops(&self) -> &Vec<String>;
    fn analyze_verifier_logs(&mut self, logs_slice: &[u8]) -> Result<bool, Self::Error>;
}

pub trait AnalyzeTracerLog {
    type Error;
    fn get_traces(&self) -> &Vec<Vec<FuncLog>>;
    fn analyze_tracer_logs(&mut self, logs_slice: &[u8]) -> Result<(), Self::Error>;
}

pub trait AnalyzeTracingVerifierLog {
    type Error;
    fn get_c_e_s(&self) -> &Vec<Vec<i32>>;
    fn get_unwind_err_loops(&self) -> &Vec<String>;
    fn get_traces(&self) -> &Vec<Vec<FuncLog>>;
    fn analyze_logs(&mut self, logs_slice: &[u8]) -> Result<bool, Self::Error>;
}

impl<T> AnalyzeVerifierLog for T
    where T: AnalyzeTracingVerifierLog {
    type Error = <T as AnalyzeTracingVerifierLog>::Error;
    fn get_c_e_s(&self) -> &Vec<Vec<i32>> {<T as AnalyzeTracingVerifierLog>::get_c_e_s(self)}
    fn get_unwind_err_loops(&self) -> &Vec<String> {<T as AnalyzeTracingVerifierLog>::get_unwind_err_loops(self)}
    fn analyze_verifier_logs(&mut self, logs_slice: &[u8]) -> Result<bool, Self::Error>
        {<T as AnalyzeTracingVerifierLog>::analyze_logs(self, logs_slice)}
}

impl<T> AnalyzeTracerLog for T
    where T: AnalyzeTracingVerifierLog {
    type Error = <T as AnalyzeTracingVerifierLog>::Error;
    fn get_traces(&self) -> &Vec<Vec<FuncLog>> {<T as AnalyzeTracingVerifierLog>::get_traces(self)}
    fn analyze_tracer_logs(&mut self, _logs_slice: &[u8]) -> Result<(), Self::Error> {
        // Do nothing, conceptually the tracing happens together with verification
        // thus analysis of tracer log shall happen together with analysis of verifier log as well
        Ok(())
    }
}