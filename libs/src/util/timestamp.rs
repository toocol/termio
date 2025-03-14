use std::time::{Duration, SystemTime};
use chrono::{DateTime, Local};

/// Get the timestamp since unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Timestamp(SystemTime);

impl Default for Timestamp {
    #[inline]
    fn default() -> Self {
        Self::now()
    }
}

impl Timestamp {
    #[inline]
    pub fn now() -> Self {
        Self(SystemTime::now())
    }

    #[inline]
    pub fn from_millis(millis: u128) -> Self {
        Self(SystemTime::UNIX_EPOCH + Duration::from_millis(millis as u64))
    }

    #[inline]
    pub fn from_micros(micros: u128) -> Self {
        Self(SystemTime::UNIX_EPOCH + Duration::from_micros(micros as u64))
    }

    #[inline]
    pub fn from_nanos(nanos: u128) -> Self {
        Self(SystemTime::UNIX_EPOCH + Duration::from_nanos(nanos as u64))
    }

    #[inline]
    pub fn as_millis<T: From<u128>>(&self) -> T {
        T::from(self.duration().as_millis())
    }

    #[inline]
    pub fn as_micros<T: From<u128>>(&self) -> T {
        T::from(self.duration().as_micros())
    }

    #[inline]
    pub fn as_nanos<T: From<u128>>(&self) -> T {
        T::from(self.duration().as_nanos())
    }

    #[inline]
    pub fn as_u16(&self) -> u16 {
        let mut ts = (self.duration().as_millis() % 65536) as u16;
        if ts == u16::MAX {
            ts += 1;
        }
        ts
    }

    /// Default format: "%Y-%m-%d %H:%M:%S"
    ///
    /// See the [`chrono::format::strftime`] module for the whole supported escape sequences.
    #[inline]
    pub fn format_string(&self, format: Option<&str>) -> String {
        let format = format.unwrap_or("%Y-%m-%d %H:%M:%S");

        let date_time: DateTime<Local> = self.0.into();
        date_time.format(format).to_string()
    }
}

impl Timestamp {
    #[inline]
    fn duration(&self) -> Duration {
        self.0.duration_since(SystemTime::UNIX_EPOCH).unwrap()
    }
}

pub trait From<T> {
    fn from(t: T) -> Self;
}
impl From<u128> for u128 {
    #[inline]
    fn from(t: u128) -> Self {
        t
    }
}
impl From<u128> for u64 {
    #[inline]
    fn from(t: u128) -> Self {
        t as u64
    }
}
impl From<u128> for u32 {
    #[inline]
    fn from(t: u128) -> Self {
        t as u32
    }
}
impl From<u128> for u16 {
    #[inline]
    fn from(t: u128) -> Self {
        t as u16
    }
}
impl From<u128> for i128 {
    #[inline]
    fn from(t: u128) -> Self {
        t as i128
    }
}
impl From<u128> for i64 {
    #[inline]
    fn from(t: u128) -> Self {
        t as i64
    }
}
impl From<u128> for i32 {
    #[inline]
    fn from(t: u128) -> Self {
        t as i32
    }
}
impl From<u128> for i16 {
    #[inline]
    fn from(t: u128) -> Self {
        t as i16
    }
}