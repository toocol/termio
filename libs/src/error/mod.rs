use code::ErrorCode;
use handle::ErrorHandle;
use std::{
    fmt::{Debug, Display, Formatter},
    panic::Location,
};

pub mod code;
pub mod convert;
pub mod handle;

pub type TermioError = Error;

pub struct Error {
    inner: Box<ErrorInner>,
}

struct ErrorInner {
    code: ErrorCode,
    message: String,
    file: String,
    line: u32,
    column: u32,
    crash: bool,
}

impl Error {
    #[inline]
    pub fn new(
        code: ErrorCode,
        message: impl ToString,
        file: &str,
        line: u32,
        column: u32,
    ) -> Self {
        Self {
            inner: Box::new(ErrorInner {
                code,
                message: message.to_string(),
                file: file.to_string(),
                line,
                column,
                crash: false,
            }),
        }
    }

    #[inline]
    pub fn new_localtion(
        code: ErrorCode,
        message: impl ToString,
        location: &'static Location<'static>,
    ) -> Self {
        Self::new(
            code,
            message,
            location.file(),
            location.line(),
            location.column(),
        )
    }

    #[inline]
    pub(crate) fn new_empty(code: ErrorCode, message: impl ToString) -> Self {
        Self::new(code, message, "", 0, 0)
    }

    #[inline]
    pub fn code(&self) -> ErrorCode {
        self.inner.code
    }

    #[inline]
    pub fn message(&self) -> &str {
        &self.inner.message
    }

    #[inline]
    pub fn file(&self) -> &str {
        &self.inner.file
    }

    #[inline]
    pub fn line(&self) -> u32 {
        self.inner.line
    }

    #[inline]
    pub fn column(&self) -> u32 {
        self.inner.column
    }

    #[inline]
    pub fn set_localtion(&mut self, location: &'static Location<'static>) {
        self.inner.file = location.file().to_string();
        self.inner.line = location.line();
        self.inner.column = location.column();
    }

    #[inline]
    pub fn is_crash(&self) -> bool {
        self.inner.crash
    }

    /// Indicate that `Error` should crash the program. 
    /// If so, the program will exit in the [`handle()`](Error::handle) function.
    #[inline]
    pub fn crash(mut self) -> Self {
        self.inner.crash = true;
        self
    }

    #[inline]
    pub fn handle(self) {
        ErrorHandle::handle(self)
    }
}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Termio error occurred, code = {:?}, message = {}, at file {}({}:{})",
            self.code(),
            self.message(),
            self.file(),
            self.line(),
            self.column()
        )
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TermioError")
            .field("code", &self.code())
            .field("message", &self.message())
            .field("file", &self.file())
            .field("line", &self.line())
            .field("column", &self.column())
            .finish()
    }
}

impl std::error::Error for Error {}

/// Macro for handling errors in a standardized way within the Termio project.
/// **(use `libs::prelude::*` to import all the used dependencies.)**
/// This macro has two variants:
///
/// 1. Handling `Result` types:
///    - `err!($err:expr)`:
///      - If the expression evaluates to `Ok`, it returns the value.
///      - If the expression evaluates to `Err`, it wraps the error into a custom `Error` type and sets the location (file, line, and column) of the caller.
///
/// 2. Creating a new `Error` with an error code and a custom message:
///    - `err!($code:ident, $msg:expr)`:
///      - Creates a new `Error` object with the specified error code and a custom error message.
///      - Automatically captures the current file, line, and column information.
///
/// # Examples
///
/// ```
/// // Example for the first variant
/// use libs::prelude::*;
/// 
/// let result: Result<(), std::io::Error> = Err(std::io::Error::new(std::io::ErrorKind::Other, "IO Error"));
/// let _ = err!(result);
///
/// // Example for the second variant
/// let err = err!(SerdeJsonError, "Failed to serialize JSON");
/// ```
#[macro_export]
macro_rules! err {
    ( $err:expr ) => {{
        match $err {
            Ok(val) => Ok(val),
            Err(e) => {
                let mut err = Error::from(e);
                err.set_localtion(std::panic::Location::caller());
                Err(err)
            }
        }
    }};
    ( $code:ident, $msg:expr ) => {
        Error::new(ErrorCode::$code, $msg, file!(), line!(), column!())
    };
}

/// See [`err`]
#[macro_export]
macro_rules! err_crash {
    ( $err:expr ) => {{
        match $err {
            Ok(val) => Ok(val),
            Err(e) => {
                let mut err = Error::from(e);
                err.set_localtion(std::panic::Location::caller());
                err.crash(true);
                Err(err)
            }
        }
    }};
    ( $code:ident, $msg:expr ) => {
        let mut error = Error::new(ErrorCode::$code, $msg, file!(), line!(), column!())
        error.crash();
        error
    };
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::prelude::*;

    #[test]
    fn test_convert() {
        match cause_error() {
            Ok(()) => unreachable!(),
            Err(e) => {
                assert_eq!(e.code(), ErrorCode::IoError);
                println!("{}", e);
            }
        };
    }

    fn cause_error() -> Result<(), Error> {
        // Open a non existent file.
        let _ = err!(File::open("non_existent_file"))?;
        Ok(())
    }
}
