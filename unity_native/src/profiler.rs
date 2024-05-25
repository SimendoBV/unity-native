use std::ffi::CStr;
use std::ffi::CString;
use std::ptr::null;
use std::ptr::null_mut;
use std::ptr::NonNull;

use thiserror::Error;

use crate::ffi;
use crate::unity_api_guid;
use crate::UnityInterface;

pub struct UnityProfiler {
    ptr: NonNull<ffi::IUnityProfilerV2>,
    available: bool,
}

#[derive(Debug, Error)]
pub enum ProfilerCreationError {
    #[error("Cannot check for profiler availability because the function pointer is missing")]
    MissingAvailableFn,
}

unsafe impl UnityInterface for UnityProfiler {
    type FFIType = ffi::IUnityProfilerV2;
    type FFIConversionError = ProfilerCreationError;
    const GUID: ffi::UnityInterfaceGUID = unity_api_guid!(0xB957E0189CB6A30B 0x83CE589AE85B9068);
}

impl TryFrom<NonNull<ffi::IUnityProfilerV2>> for UnityProfiler {
    type Error = ProfilerCreationError;

    fn try_from(value: NonNull<ffi::IUnityProfilerV2>) -> Result<Self, Self::Error> {
        let is_available = unsafe {
            match value.as_ref().IsAvailable {
                Some(func) => func() != 0,
                None => return Err(ProfilerCreationError::MissingAvailableFn),
            }
        };

        Ok(Self {
            ptr: value,
            available: is_available,
        })
    }
}

pub struct ProfilerMarker {
    desc_ptr: *const ffi::UnityProfilerMarkerDesc,
}

unsafe impl Send for ProfilerMarker {}

#[derive(Debug, Clone, Copy)]
enum EventType {
    Begin,
    End,
    Single,
}

impl From<EventType> for ffi::UnityProfilerMarkerEventType {
    fn from(value: EventType) -> Self {
        match value {
            EventType::Begin => {
                ffi::UnityProfilerMarkerEventType_::kUnityProfilerMarkerEventTypeBegin as u16
            }
            EventType::End => {
                ffi::UnityProfilerMarkerEventType_::kUnityProfilerMarkerEventTypeEnd as u16
            }
            EventType::Single => {
                ffi::UnityProfilerMarkerEventType_::kUnityProfilerMarkerEventTypeBegin as u16
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum CreateMarkerErr {
    #[error("Error returned by Unity during marker creation: {0}")]
    UnityErr(std::os::raw::c_int),
}

pub enum ScopedProfilerSample<'a, 'b> {
    Disabled,
    Enabled {
        marker: &'a ProfilerMarker,
        profiler: &'b UnityProfiler,
    },
}

pub enum ManualProfilerSample<'a, 'b> {
    Disabled,
    Enabled {
        ended: bool,
        marker: &'a ProfilerMarker,
        profiler: &'b UnityProfiler,
    },
}

impl UnityProfiler {
    pub fn is_enabled(&self) -> bool {
        if !self.available {
            return false;
        }

        let as_ref = unsafe { self.ptr.as_ref() };

        let func = as_ref.IsEnabled.unwrap();

        unsafe { func() != 0 }
    }

    pub fn create_marker(&mut self, name: &str) -> Result<ProfilerMarker, CreateMarkerErr> {
        let name_c = CString::new(name).unwrap();

        let mut raw_marker: *const ffi::UnityProfilerMarkerDesc = null_mut();

        unsafe {
            let createfn = self.ptr.as_ref().CreateMarker.unwrap();

            let create_result = createfn(
                &mut raw_marker,
                name_c.as_ptr(),
                ffi::UnityBuiltinProfilerCategory_::kUnityProfilerCategoryOther as u16,
                ffi::UnityProfilerMarkerFlag_::kUnityProfilerMarkerFlagDefault as u16,
                0,
            );

            if create_result != 0 {
                return Err(CreateMarkerErr::UnityErr(create_result));
            }
        }

        Ok(ProfilerMarker {
            desc_ptr: raw_marker,
        })
    }

    fn emit_event(&self, marker: &ProfilerMarker, event: EventType) {
        debug_assert!(self.is_enabled());
        debug_assert!(!marker.desc_ptr.is_null());

        unsafe {
            let emitfn = self.ptr.as_ref().EmitEvent.unwrap();

            emitfn(marker.desc_ptr, event.into(), 0, null());
        }
    }
}

impl ProfilerMarker {
    fn get_name(&self) -> &str {
        let name_c = unsafe { CStr::from_ptr(self.desc_ptr.as_ref().unwrap().name) };

        name_c.to_str().unwrap()
    }

    pub fn sample_scope<'a, 'b>(
        &'a self,
        profiler: &'b UnityProfiler,
    ) -> ScopedProfilerSample<'a, 'b> {
        if !profiler.is_enabled() {
            return ScopedProfilerSample::Disabled;
        }

        profiler.emit_event(self, EventType::Begin);

        ScopedProfilerSample::Enabled {
            marker: self,
            profiler,
        }
    }

    pub fn sample_manual<'a, 'b>(
        &'a self,
        profiler: &'b UnityProfiler,
    ) -> ManualProfilerSample<'a, 'b> {
        if !profiler.is_enabled() {
            return ManualProfilerSample::Disabled;
        }

        profiler.emit_event(self, EventType::Begin);

        ManualProfilerSample::Enabled {
            ended: false,
            marker: self,
            profiler,
        }
    }
}

impl<'a, 'b> Drop for ScopedProfilerSample<'a, 'b> {
    fn drop(&mut self) {
        match self {
            ScopedProfilerSample::Disabled => {}
            ScopedProfilerSample::Enabled { marker, profiler } => {
                profiler.emit_event(marker, EventType::End);
            }
        }
    }
}

impl<'a, 'b> ManualProfilerSample<'a, 'b> {
    pub fn end_sample(&mut self) {
        match self {
            ManualProfilerSample::Disabled => {}
            ManualProfilerSample::Enabled {
                ended,
                marker,
                profiler,
            } => {
                debug_assert!(
                    !(*ended),
                    "Profiler sample of marker {} ended multiple times",
                    marker.get_name()
                );
                *ended = true;
                profiler.emit_event(marker, EventType::End);
            }
        }
    }
}

impl<'a, 'b> Drop for ManualProfilerSample<'a, 'b> {
    fn drop(&mut self) {
        if let ManualProfilerSample::Enabled {
            ended,
            marker,
            profiler: _,
        } = self
        {
            debug_assert!(
                *ended,
                "Profiler sample of marker {} not ended",
                marker.get_name()
            );
        }
    }
}
