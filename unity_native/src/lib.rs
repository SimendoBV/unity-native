#[cfg(feature = "log")]
pub mod logger;

#[cfg(feature = "profiler")]
pub mod profiler;

pub mod types;

pub use ffi::IUnityInterfaces as RawUnityInterfaces;
use thiserror::Error;
use unity_native_sys as ffi;

pub use unity_native_proc_macro::*;

#[macro_use]
mod macros;

use std::ptr::NonNull;

/// The main interface provider for the Unity Native Plugin API.
/// Constructed with a [RawUnityInterfaces] pointer, which is obtained
/// through the main Unity plugin load hook.
///
/// A convenient way to construct a function that receives this struct
/// automatically is by using the [unity_plugin_load] and [unity_plugin_unload]
/// to construct Unity plugin load and unload hooks.
#[derive(Debug)]
pub struct UnityInterfaces {
    ptr: NonNull<ffi::IUnityInterfaces>,
}

//TODO: Is this actually true? Docs are unclear, gotta test manually probably
unsafe impl Send for UnityInterfaces {}
unsafe impl Sync for UnityInterfaces {}

/// The main trait implemented by all supported Unity API wrappers.
/// This should not be implemented manually. Instead, the structs provided by this crate
/// that implement this trait for you should be used.
///
/// Any struct implementing this trait can be obtained through [UnityInterfaces::get]
///
/// # Safety
///
/// The internal library functions using the implementors of this trait
/// expect that the pointer returned by Unity actually matches a struct of the
/// declared [UnityInterface::FFIType]. If this is not true, any function calls
/// on the returned struct will be UB.
pub unsafe trait UnityInterface:
    TryFrom<NonNull<Self::FFIType>, Error = Self::FFIConversionError>
{
    /// The type of the struct that the pointer returned by Unity points to.
    type FFIType;

    /// The error returned when conversion fails
    type FFIConversionError;

    /// The GUID of the Unity API corresponding to the implementor
    const GUID: ffi::UnityInterfaceGUID;
}

/// An error during the construction of the requested Unity API
#[derive(Error, Debug)]
pub enum GetError<T> {
    /// A NULL pointer was returned by Unity
    #[error("Null pointer provided")]
    NullPtr,

    /// The pointer returned by Unity could not be used to actually
    /// construct a safe wrapper
    #[error("Conversion from C-type to wrapper failed: {0}")]
    ConversionError(#[source] T),
}

/// An error during the construction of the main UnityInterfaces wrapper
#[derive(Error, Debug)]
pub enum UnityInterfaceCreateErr {
    /// User provided a NULL pointer
    #[error("Null pointer provided")]
    NullPtr,
}

impl UnityInterfaces {
    /// Constructs a safe UnityInterfaces wrapper over a raw IUnityInterfaces pointer.
    /// The given pointer must be non-NULL.
    ///
    /// # Safety
    /// The caller must ensure that the provided pointer actually points to
    /// the correct struct, if it is not null. NULL pointers will not result in UB,
    /// but will simply return an [Err] instead.
    pub unsafe fn new(ptr: *mut ffi::IUnityInterfaces) -> Result<Self, UnityInterfaceCreateErr> {
        NonNull::new(ptr)
            .map(|nonnull| UnityInterfaces { ptr: nonnull })
            .ok_or(UnityInterfaceCreateErr::NullPtr)
    }

    /// Attempts to construct a safe wrapper for the requested Unity API.
    pub fn get<T: UnityInterface>(&self) -> Result<T, GetError<T::FFIConversionError>> {
        let iface = unsafe { self.ptr.as_ref().GetInterface.map(|f| f(T::GUID)) };

        let as_nonnull = iface.and_then(|ptr| NonNull::new(ptr as *mut T::FFIType));
        if as_nonnull.is_none() {
            return Err(GetError::NullPtr);
        }

        let iface = as_nonnull.unwrap();

        T::try_from(iface).map_err(GetError::ConversionError)
    }
}
