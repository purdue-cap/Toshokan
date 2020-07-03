use handlebars::TemplateError;
use handlebars::TemplateFileError;
use handlebars::RenderError;
use std::cell::BorrowError;
use std::cell::BorrowMutError;
use std::io::Error as IOError;

quick_error! {
    #[derive(Debug)]
    pub enum EncodeError {
        TemplateError(err: TemplateError) {
            from()
            cause(err)
            display("{}", err)
        }
        TemplateFileError(err: TemplateFileError) {
            from()
            cause(err)
            display("{}", err)
        }
        RenderError(err: RenderError){
            from()
            cause(err)
            display("{}", err)
        }
        BorrowError(err: BorrowError){
            from()
            cause(err)
            display("{}", err)
        }
        BorrowMutError(err: BorrowMutError){
            from()
            cause(err)
            display("{}", err)
        }
        IOError(err: IOError) {
            from()
            cause(err)
            display("{}", err)
        }
        RewriteError(desc: &'static str) {
            display("{}", desc)
        }
        SimpleRenderError(desc: &'static str) {
            display("{}", desc)
        }
    }
}

