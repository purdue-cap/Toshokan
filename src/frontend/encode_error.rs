use handlebars::TemplateError;
use handlebars::TemplateFileError;
use handlebars::RenderError;
use std::cell::BorrowError;
use std::cell::BorrowMutError;
use std::io::Error as IOError;
use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum EncodeError {
        TemplateError(err: TemplateError) {
            from()
            source(err)
            display("{}", err)
        }
        TemplateFileError(err: TemplateFileError) {
            from()
            source(err)
            display("{}", err)
        }
        RenderError(err: RenderError){
            from()
            source(err)
            display("{}", err)
        }
        BorrowError(err: BorrowError){
            from()
            source(err)
            display("{}", err)
        }
        BorrowMutError(err: BorrowMutError){
            from()
            source(err)
            display("{}", err)
        }
        IOError(err: IOError) {
            from()
            source(err)
            display("{}", err)
        }
        RewriteError(desc: &'static str) {
            display("{}", desc)
        }
        SimpleRenderError(desc: &'static str) {
            display("{}", desc)
        }
        ParamError {
            display("State Param not found")
        }
    }
}

