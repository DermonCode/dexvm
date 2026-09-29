//! VM errors: fatal (abort run) vs throwable (normal JVM exception).

use crate::dex::read::DexError;
use crate::vm::value::JValue;

/// A fatal VM error. Aborts the current execution run.
#[derive(Debug, Clone, thiserror::Error)]
pub enum JvmError {
    /// A Java exception object (arena id) was thrown and not caught.
    #[error("uncaught java exception")]
    Uncaught(u32),
    /// DEX/class resolution problem.
    #[error("resolution error: {0}")]
    Resolution(String),
    /// Instruction decode problem.
    #[error("decode error: {0}")]
    Decode(String),
    /// Instruction budget exhausted (infinite-loop guard).
    #[error("instruction budget exceeded")]
    BudgetExceeded,
    /// Stack depth limit hit.
    #[error("stack overflow")]
    StackOverflow,
    /// System.exit(code)
    #[error("System.exit({0})")]
    Exit(i32),
    /// Everything else.
    #[error("fatal: {0}")]
    Fatal(String),
}

impl From<DexError> for JvmError {
    fn from(e: DexError) -> Self {
        JvmError::Decode(e.to_string())
    }
}

impl From<crate::vm::NatErr> for JvmError {
    fn from(e: crate::vm::NatErr) -> Self {
        match e {
            crate::vm::NatErr::Throw(t) => JvmError::Uncaught(t),
            crate::vm::NatErr::Fatal(j) => j,
        }
    }
}

/// Result of a VM operation: normal result or a thrown Java exception object.
pub type ExecResult<T = JValue> = Result<T, u32>;
