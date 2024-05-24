#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


pub mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::{ffi::CString, ptr::NonNull};

use ffi::*;

#[derive(Debug)]
pub struct UnityInterfaces {
    ptr: NonNull<IUnityInterfaces>
}

unsafe impl Send for UnityInterfaces {}

pub struct UnityLogger {
    ptr: NonNull<IUnityLog>
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

            if logger_raw.Log.is_none() {
                return Err(());
            } 

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

unsafe impl UnityInterface for UnityLogger {
    type FFIType = IUnityLog;

    const GUID: UnityInterfaceGUID = UnityInterfaceGUID {
        m_GUIDHigh: 0x9E7507FA5B444D5D,
        m_GUIDLow: 0x92FB979515EA83FC,
    };

    fn from_ffi(ptr: NonNull<Self::FFIType>) -> Result<Self, ()> where Self: Sized {
        Ok(Self {
            ptr
        })
    }
}

pub unsafe trait UnityInterface {
    type FFIType;
    const GUID: UnityInterfaceGUID;

    fn from_ffi(ptr: NonNull<Self::FFIType>) -> Result<Self, ()> where Self: Sized;
}

impl UnityInterfaces {
    pub fn new(ptr: *mut IUnityInterfaces) -> Result<Self, ()> {
        NonNull::new(ptr).map(|nonnull| UnityInterfaces {
            ptr: nonnull
        }).ok_or(())
    }

    pub fn get<T: UnityInterface>(&self) -> Result<T, ()> {        
        let iface = unsafe { 
            self.ptr.as_ref().GetInterface.map(|f| f(T::GUID))
        };

        let as_nonnull = iface.and_then(|ptr| NonNull::new(ptr as *mut T::FFIType));
        if as_nonnull.is_none() {
            return Err(());
        }

        let iface = as_nonnull.unwrap();

        T::from_ffi(iface)
    }
}
