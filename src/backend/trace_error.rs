use std::io::Error as IOError;
use serde_json::Error as JSONError;
use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum TraceError {
        IOError(err: IOError) {
            from()
            source(err)
            display("{}", err)
        }
        JSONError(json_buf: Option<Vec<u8>>, err: JSONError) {
            from(err: JSONError) -> (None, err)
            context(buf: Vec<u8>, err: JSONError) -> (Some(buf), err)
            source(err)
            display("{}", err)
        }
        JBMCLogError(err: &'static str) {
            display("{}", err)
        }
        JBMCUnwindError(assertion: String) {
            display("UnwindAssertion:{}", assertion)
        }
        OtherError(desc: &'static str) {
            display("{}", desc)
        }
    }
}