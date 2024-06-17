use std::{ffi::CString, ptr::NonNull};

use log::Log;

use crate::{ffi, unity_api_guid, UnityInterface};

/// A wrapper for the Unity Logging API. It supports
/// both manual logging using [UnityLogger::log_generic] (and friends),
/// and supports conversion to a [UnityRustLogger], which is a wrapper
/// for [UnityLogger] that implements the [log::Log] trait. This allows
/// it to be used with the standard [log::log!] macros.
pub struct UnityLogger {
    ptr: NonNull<ffi::IUnityLog>,
}

unsafe impl Send for UnityLogger {}
unsafe impl Sync for UnityLogger {}

unsafe impl UnityInterface for UnityLogger {
    type FFIType = ffi::IUnityLog;
    type FFIConversionError = ();
    const GUID: ffi::UnityInterfaceGUID = unity_api_guid!(0x9E7507FA5B444D5D 0x92FB979515EA83FC);
}

impl TryFrom<NonNull<ffi::IUnityLog>> for UnityLogger {
    type Error = ();

    fn try_from(value: NonNull<ffi::IUnityLog>) -> Result<Self, Self::Error> {
        Ok(Self { ptr: value })
    }
}

/// The different log levels
/// supported by the Unity logging API
pub enum UnityLogType {
    Info,
    Warning,
    Error,
    Exception,
}

impl From<log::Level> for UnityLogType {
    fn from(value: log::Level) -> Self {
        match value {
            log::Level::Error => UnityLogType::Error,
            log::Level::Warn => UnityLogType::Warning,
            log::Level::Info => UnityLogType::Info,
            log::Level::Debug => UnityLogType::Info,
            log::Level::Trace => UnityLogType::Info,
        }
    }
}

impl From<UnityLogType> for ffi::UnityLogType {
    fn from(value: UnityLogType) -> Self {
        match value {
            UnityLogType::Info => ffi::UnityLogType::kUnityLogTypeLog,
            UnityLogType::Warning => ffi::UnityLogType::kUnityLogTypeWarning,
            UnityLogType::Error => ffi::UnityLogType::kUnityLogTypeError,
            UnityLogType::Exception => ffi::UnityLogType::kUnityLogTypeException,
        }
    }
}

impl UnityLogger {
    /// Converts this [UnityLogger] to a [UnityRustLogger], which is a struct
    /// compatible with the [log::Log] trait. This allows you to use
    /// the UnityLogger with the standard log macros (see [log::log])
    pub fn to_rust_logger(self, initial_level: log::LevelFilter) -> UnityRustLogger {
        log::set_max_level(initial_level);
        UnityRustLogger { logger: self }
    }

    /// Logs a generic message using the Unity Log API, with the provided
    /// level. The filename and line are supposed to be the file and line of
    /// the function generating the log, so using the Rust [file!] and [line!]
    /// macros is recommended
    pub fn log_generic(&self, level: UnityLogType, msg: &str, filename: &str, line: u32) {
        let line_c = std::os::raw::c_int::try_from(line).unwrap();
        let message_c_str = CString::new(msg).unwrap();
        let filename_c_str = CString::new(filename).unwrap();

        unsafe {
            let logger_raw = self.ptr.as_ref();
            let logfn = logger_raw.Log.unwrap();

            logfn(
                level.into(),
                message_c_str.as_ptr(),
                filename_c_str.as_ptr(),
                line_c,
            );
        }
    }

    /// Convenience wrapper for [`UnityLogger::log_generic`] with
    /// level [`UnityLogType::Info`]
    pub fn log_info(&self, msg: &str, filename: &str, line: u32) {
        self.log_generic(UnityLogType::Info, msg, filename, line)
    }

    /// Convenience wrapper for [`UnityLogger::log_generic`] with
    /// level [`UnityLogType::Warning`]
    pub fn log_warning(&self, msg: &str, filename: &str, line: u32) {
        self.log_generic(UnityLogType::Warning, msg, filename, line)
    }

    /// Convenience wrapper for [`UnityLogger::log_generic`] with
    /// level [`UnityLogType::Error`]
    pub fn log_error(&self, msg: &str, filename: &str, line: u32) {
        self.log_generic(UnityLogType::Error, msg, filename, line)
    }

    /// Convenience wrapper for [`UnityLogger::log_generic`] with
    /// level [`UnityLogType::Exception`]
    pub fn log_exception(&self, msg: &str, filename: &str, line: u32) {
        self.log_generic(UnityLogType::Exception, msg, filename, line)
    }
}

/// Wrapper for [UnityLogger] that implements [log::Log], so it can be used with [log::log!].
/// Created using [UnityLogger::to_rust_logger]
#[repr(transparent)]
pub struct UnityRustLogger {
    logger: UnityLogger,
}

impl Log for UnityRustLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::STATIC_MAX_LEVEL && metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let prefix = match record.level() {
            log::Level::Error => "",
            log::Level::Warn => "",
            log::Level::Info => "INFO: ",
            log::Level::Debug => "DEBUG: ",
            log::Level::Trace => "TRACE: ",
        };

        let body = format!("{}{}", prefix, record.args());

        self.logger.log_generic(
            record.level().into(),
            body.as_str(),
            record.file().unwrap_or("<unknown file>"),
            record.line().unwrap_or(0),
        )
    }

    fn flush(&self) {
        //no-op
    }
}
