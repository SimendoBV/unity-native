use std::{ffi::CString, ptr::NonNull};

use crate::{ffi, unity_api_guid, UnityInterface};

pub struct UnityLogger {
    ptr: NonNull<ffi::IUnityLog>
}

unsafe impl UnityInterface for UnityLogger {
    type FFIType = ffi::IUnityLog;
    type FFIConversionError = ();
    const GUID: ffi::UnityInterfaceGUID = unity_api_guid!(0x9E7507FA5B444D5D 0x92FB979515EA83FC);
}

impl TryFrom<NonNull<ffi::IUnityLog>> for UnityLogger {
    type Error = ();

    fn try_from(value: NonNull<ffi::IUnityLog>) -> Result<Self, Self::Error> {
        Ok(Self {
            ptr: value
        })
    }
}

pub enum UnityLogType {
    Info,
    Warning,
    Error,
    Exception
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
    pub fn log_generic(&self, level: UnityLogType, msg: &str, filename: &str, line: u32) -> Result<(), ()> {
        let line_c = std::os::raw::c_int::try_from(line).unwrap(); 
        let message_c_str = CString::new(msg).unwrap();
        let filename_c_str = CString::new(filename).unwrap();

        unsafe {
            let logger_raw = self.ptr.as_ref();
            let logfn = logger_raw.Log.unwrap();

            logfn(level.into(), message_c_str.as_ptr(), filename_c_str.as_ptr(), line_c);
        }

        Ok(())
    }

    pub fn log_info(&self, msg: &str, filename: &str, line: u32) -> Result<(), ()> {
        self.log_generic(UnityLogType::Info, msg, filename, line)
    }

    pub fn log_warning(&self, msg: &str, filename: &str, line: u32) -> Result<(), ()> {
        self.log_generic(UnityLogType::Warning, msg, filename, line)
    }

    pub fn log_error(&self, msg: &str, filename: &str, line: u32) -> Result<(), ()> {
        self.log_generic(UnityLogType::Error, msg, filename, line)
    }

    pub fn log_exception(&self, msg: &str, filename: &str, line: u32) -> Result<(), ()> {
        self.log_generic(UnityLogType::Exception, msg, filename, line)
    }
}

#[macro_export]
macro_rules! info {
    ($logger:expr, $msg:expr) => {
        $logger.log_info($msg, file!(), line!())
    };
}

#[macro_export]
macro_rules! warning {
    ($logger:expr, $msg:expr) => {
        $logger.log_warning($msg, file!(), line!())
    };
}

#[macro_export]
macro_rules! error {
    ($logger:expr, $msg:expr) => {
        $logger.log_error($msg, file!(), line!())
    };
}

#[macro_export]
macro_rules! exception {
    ($logger:expr, $msg:expr) => {
        $logger.log_exception($msg, file!(), line!())
    };
}
