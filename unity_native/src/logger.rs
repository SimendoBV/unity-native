use std::{ffi::CString, ptr::NonNull};

use log::{LevelFilter, Log};

use crate::{ffi, unity_api_guid, UnityInterface};

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

impl From<UnityLogType> for ffi::UnityLogType::Type {
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
    pub fn to_rust_logger(self, initial_level: log::LevelFilter) -> UnityRustLogger {
        log::set_max_level(initial_level);
        UnityRustLogger { logger: self }
    }

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

    pub fn log_info(&self, msg: &str, filename: &str, line: u32) {
        self.log_generic(UnityLogType::Info, msg, filename, line)
    }

    pub fn log_warning(&self, msg: &str, filename: &str, line: u32) {
        self.log_generic(UnityLogType::Warning, msg, filename, line)
    }

    pub fn log_error(&self, msg: &str, filename: &str, line: u32) {
        self.log_generic(UnityLogType::Error, msg, filename, line)
    }

    pub fn log_exception(&self, msg: &str, filename: &str, line: u32) {
        self.log_generic(UnityLogType::Exception, msg, filename, line)
    }
}

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

        let body = format!("{}", record.args());

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
