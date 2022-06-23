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
        JSONError(json_buf: Option<Result<String, std::string::FromUtf8Error>>, err: JSONError) {
            from(err: JSONError) -> (None, err)
            context(buf: Vec<u8>, err: JSONError) -> (Some(String::from_utf8(buf)), err)
            source(err)
            display("{}", err)
        }
        JBMCLogError(err: &'static str) {
            display("{}", err)
        }
        JavaTracerLogError(err: &'static str) {
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