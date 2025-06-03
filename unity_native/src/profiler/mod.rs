use std::ffi::CString;
use std::ffi::c_void;
use std::os::raw::c_int;
use std::ptr::NonNull;
use std::ptr::null;
use std::ptr::null_mut;

use thiserror::Error;

use crate::UnityInterface;
use crate::ffi;
use crate::unity_api_guid;

mod marker;
mod sample;

pub use marker::*;
pub use sample::*;

#[derive(Debug)]
pub struct UnityProfiler {
    ptr: NonNull<ffi::IUnityProfilerV2>,
    available: bool,
}

unsafe impl Send for UnityProfiler {}
unsafe impl Sync for UnityProfiler {}

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
                ffi::UnityProfilerMarkerEventType_::kUnityProfilerMarkerEventTypeBegin.into()
            }
            EventType::End => {
                ffi::UnityProfilerMarkerEventType_::kUnityProfilerMarkerEventTypeEnd.into()
            }
            EventType::Single => {
                ffi::UnityProfilerMarkerEventType_::kUnityProfilerMarkerEventTypeBegin.into()
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum CreateMarkerErr {
    #[error("Error returned by Unity during marker creation: {0}")]
    Marker(std::os::raw::c_int),

    #[error("Error returned by Unity during marker metadata creation: {0}")]
    MarkerMeta(std::os::raw::c_int),
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

    pub fn create_marker(&self, name: &str) -> Result<ProfilerMarker<(), 0>, CreateMarkerErr> {
        self.create_marker_with_data::<(), 0>(name)
    }

    pub fn create_marker_with_data<MetaType: MarkerMeta<N>, const N: usize>(
        &self,
        name: &str,
    ) -> Result<ProfilerMarker<MetaType, N>, CreateMarkerErr> {
        debug_assert!(
            N < u16::MAX as usize,
            "Cannot handle more than {} metadata items, {} given",
            u16::MAX,
            N
        );

        let name_c = CString::new(name).unwrap();
        let descriptors = MetaType::get_descriptors();

        let mut raw_marker: *const ffi::UnityProfilerMarkerDesc = null_mut();

        unsafe {
            let createfn = self.ptr.as_ref().CreateMarker.unwrap();

            let create_result = createfn(
                &mut raw_marker,
                name_c.as_ptr(),
                ffi::UnityBuiltinProfilerCategory_::kUnityProfilerCategoryOther.into(),
                ffi::UnityProfilerMarkerFlag_::kUnityProfilerMarkerFlagDefault.into(),
                N as c_int,
            );

            if create_result != 0 {
                return Err(CreateMarkerErr::Marker(create_result));
            }
        }

        for (i, descriptor) in descriptors.iter().enumerate() {
            let set_result = unsafe {
                let setmetafn = self.ptr.as_ref().SetMarkerMetadataName.unwrap();

                setmetafn(
                    raw_marker,
                    i as i32,
                    descriptor.name_c().as_ptr(),
                    descriptor.datatype.into(),
                    descriptor.unit.into(),
                )
            };

            if set_result != 0 {
                return Err(CreateMarkerErr::MarkerMeta(set_result));
            }
        }

        Ok(unsafe { ProfilerMarker::new(raw_marker) })
    }

    fn emit_event<T: MarkerMeta<N>, const N: usize>(
        &self,
        marker: &ProfilerMarker<T, N>,
        event: EventType,
        meta: Option<&T>,
    ) {
        debug_assert!(self.available);
        debug_assert!(!marker.raw().is_null());

        unsafe {
            let emitfn = self.ptr.as_ref().EmitEvent.unwrap();

            match meta {
                None => emitfn(marker.raw(), event.into(), 0, null()),
                Some(meta) => {
                    let eventdata = meta.get_data();
                    let data_buffers = eventdata.map(|edata| edata.to_c_compatible_bytes());

                    let mut unity_eventdata: [ffi::UnityProfilerMarkerData; N] =
                        [ffi::UnityProfilerMarkerData::default(); N];

                    for i in 0..N {
                        unity_eventdata[i] = ffi::UnityProfilerMarkerData {
                            type_: MarkerDataType::from(eventdata[i]).into(),
                            reserved0: 0,
                            reserved1: 0,
                            size: data_buffers[i].len() as u32,
                            ptr: data_buffers[i].as_ptr() as *const c_void,
                        }
                    }

                    let unity_edata_ptr = if N == 0 {
                        null()
                    } else {
                        unity_eventdata.as_ptr()
                    };

                    emitfn(marker.raw(), event.into(), N as u16, unity_edata_ptr);
                }
            }
        }
    }

    pub fn register_current_thread(
        &self,
        group_name: &str,
        thread_name: &str,
    ) -> Result<UnityThreadId, RegisterThreadErr> {
        if !group_name.is_ascii() || !thread_name.is_ascii() {
            return Err(RegisterThreadErr::NonAscii);
        }

        assert!(group_name.is_ascii(), "Group name must be ASCII");
        assert!(thread_name.is_ascii(), "Thread name must be ASCII");

        let mut out_thread_id = 0;
        let group_name_c = CString::new(group_name).map_err(|_| RegisterThreadErr::Nul)?;
        let thread_name_c = CString::new(thread_name).map_err(|_| RegisterThreadErr::Nul)?;

        let result = unsafe {
            self.ptr.as_ref().RegisterThread.unwrap()(
                &raw mut out_thread_id,
                group_name_c.as_ptr(),
                thread_name_c.as_ptr(),
            )
        };

        match result {
            0 => Ok(UnityThreadId(out_thread_id)),
            other => Err(RegisterThreadErr::Unity(other)),
        }
    }

    pub fn unregister_thread(&self, thread_id: UnityThreadId) -> Result<(), c_int> {
        let result = unsafe { self.ptr.as_ref().UnregisterThread.unwrap()(thread_id.0) };

        match result {
            0 => Ok(()),
            other => Err(other),
        }
    }

    pub fn unregister_current_thread(&self) -> Result<(), c_int> {
        self.unregister_thread(UnityThreadId(0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnityThreadId(ffi::UnityProfilerThreadId);

#[derive(Debug, Error)]
pub enum RegisterThreadErr {
    #[error("Group or thread name contained non-ascii characters")]
    NonAscii,

    #[error("Group or thread name contained nul-bytes")]
    Nul,

    #[error("Unity API returned an error code: {}", .0)]
    Unity(c_int),
}
