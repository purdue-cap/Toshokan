use std::io::Error as IOError;
use serde_json::Error as JSONError;
use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum TraceError {
        IOError(err: IOError) {
            from()
            cause(err)
            display("{}", err)
        }
        JSONError(err: JSONError) {
            from()
            cause(err)
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