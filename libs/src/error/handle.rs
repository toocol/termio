use std::process::exit;

use log::error;
use super::Error;

pub struct ErrorHandle;

impl ErrorHandle {
    #[inline]
    pub fn handle(error: Error) {
        error!("{}", error);

        // TODO: Send error info

        if error.is_crash() {
            exit(error.code() as i32)
        }
    }
}