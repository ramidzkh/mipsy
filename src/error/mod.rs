pub mod compile_error;
pub mod runtime_error;

pub type MipsyResult<T> = Result<T, MipsyError>;
pub type CompileError   = compile_error::CompileError;
pub type RuntimeError   = runtime_error::RuntimeError;

#[derive(Debug)]
pub enum MipsyError {
    Compile(CompileError),
    Runtime(RuntimeError),
}

#[macro_export]
macro_rules! cerr {
    ($err:expr) => {
        Err(crate::error::MipsyError::Compile($err))
    };
}

#[macro_export]
macro_rules! rerr {
    ($err:expr) => {
        Err(crate::error::MipsyError::Runtime($err))
    };
}
