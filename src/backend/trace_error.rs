use std::io::Error as IOError;
use serde_json::Error as JSONError;

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
        OtherError(desc: &'static str) {
            display("{}", desc)
        }
    }
}