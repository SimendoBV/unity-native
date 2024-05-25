pub mod logger;
pub mod profiler;
use thiserror::Error;
pub use unity_native_sys as ffi;
pub use unity_native_proc_macro::*;

#[macro_use]
mod macros;

use std::ptr::NonNull;

#[derive(Debug)]
pub struct UnityInterfaces {
    ptr: NonNull<ffi::IUnityInterfaces>
}

//TODO: Is this actually true? Docs are unclear, gotta test manually probably
unsafe impl Send for UnityInterfaces {}

pub unsafe trait UnityInterface: TryFrom<NonNull<Self::FFIType>, Error = Self::FFIConversionError> {
    type FFIType;
    type FFIConversionError;
    const GUID: ffi::UnityInterfaceGUID;
}

#[derive(Error, Debug)]
pub enum GetError<T> {
    #[error("Null pointer provided")]
    NullPtr,
    
    #[error("Conversion from C-type to wrapper failed: {0}")]
    ConversionError(#[source] T)
}

#[derive(Error, Debug)]
pub enum UnityInterfaceCreateErr {
    #[error("Null pointer provided")]
    NullPtr
}

impl UnityInterfaces {
    pub unsafe fn new(ptr: *mut ffi::IUnityInterfaces) -> Result<Self, UnityInterfaceCreateErr> {
        NonNull::new(ptr).map(|nonnull| UnityInterfaces {
            ptr: nonnull
        }).ok_or(UnityInterfaceCreateErr::NullPtr)
    }

    pub fn get<T: UnityInterface>(&self) -> Result<T, GetError<T::FFIConversionError>> {        
        let iface = unsafe { 
            self.ptr.as_ref().GetInterface.map(|f| f(T::GUID))
        };

        let as_nonnull = iface.and_then(|ptr| NonNull::new(ptr as *mut T::FFIType));
        if as_nonnull.is_none() {
            return Err(GetError::NullPtr);
        }

        let iface = as_nonnull.unwrap();

        T::try_from(iface).map_err(GetError::ConversionError)
    }
}
