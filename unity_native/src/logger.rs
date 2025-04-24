use std::{ffi::CString, ptr::NonNull};

use log::Log;

use crate::{UnityInterface, ffi, unity_api_guid};

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
        self.make_rust_logger_internal(initial_level, None)
    }

    /// Same as [Self::to_rust_logger], but adds the given prefix to all logged entries
    /// so that their source is recognizable from Unity
    pub fn to_rust_logger_for_app(
        self,
        initial_level: log::LevelFilter,
        app_name: &'static str,
    ) -> UnityRustLogger {
        self.make_rust_logger_internal(initial_level, Some(app_name))
    }

    fn make_rust_logger_internal(
        self,
        initial_level: log::LevelFilter,
        app_name: Option<&'static str>,
    ) -> UnityRustLogger {
        log::set_max_level(initial_level);
        UnityRustLogger {
            app_prefix: app_name,
            logger: self,
        }
    }

    /// Logs a generic message using the Unity Log API, with the provided
    /// level. The filename and line are supposed to be the file and line of
    /// the function generating the log, so using the Rust [file!] and [line!]
    /// macros is recommended
    pub fn log_generic(&self, level: UnityLogType, msg: &str, filename: &str, line: u32) {
        let line_c = std::os::raw::c_int::try_from(line).unwrap_or(std::os::raw::c_int::MIN);

        let message_c_str = filter_str_to_c_string(msg);
        let filename_c_str = filter_str_to_c_string(filename);

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
pub struct UnityRustLogger {
    app_prefix: Option<&'static str>,
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

        let needs_colons = self.app_prefix.is_some()
            || matches!(
                record.level(),
                log::Level::Info | log::Level::Debug | log::Level::Trace
            );

        let level_prefix = match record.level() {
            log::Level::Error => "",
            log::Level::Warn => "",
            log::Level::Info => "info",
            log::Level::Debug => "debug",
            log::Level::Trace => "trace",
        };

        let app_prefix = self.app_prefix.unwrap_or("");

        let colons = if needs_colons { ": " } else { "" };

        let body = format!(
            "{}{}{}",
            format!("{} {}", app_prefix, level_prefix).trim(),
            colons,
            record.args()
        );

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

/// Make a nul-terminated UTF8 string by replacing all characters with a NUL
/// in it with the UTF8 replacement character
fn filter_str_to_c_string(s: &str) -> CString {
    if !s.as_bytes().contains(&0) {
        return CString::new(s).expect("No NUL bytes were detected, check invalid");
    }

    // A NUL byte was found, reconstruct the string char-by-char
    let mut tmp_str = String::new();

    for c in s.chars() {
        if c == '\0' {
            tmp_str.push(char::REPLACEMENT_CHARACTER);
        } else {
            tmp_str.push(c);
        }
    }

    CString::new(tmp_str).expect("Reconstructed string was invalid")
}
