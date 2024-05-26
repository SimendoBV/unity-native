use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;
use std::marker::PhantomData;
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

pub struct ProfilerMarker<T: MarkerMeta<N> = (), const N: usize = 0> {
    desc_ptr: *const ffi::UnityProfilerMarkerDesc,
    meta_type: PhantomData<T>,
}

#[derive(Debug, Clone, Copy)]
pub struct MarkerMetaDescriptor {
    name: &'static str,
    datatype: MarkerDataType,
    unit: MarkerDataUnit,
}

impl MarkerMetaDescriptor {
    pub fn new(name: &'static str, datatype: MarkerDataType, unit: MarkerDataUnit) -> Self {
        Self {
            name,
            datatype,
            unit,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, u8)] // Gotta make the layout consistent so we can sent it to the Unity C++ API with some trickery
pub enum MarkerMetaData<'a> {
    Int32(i32),
    Uint32(u32),
    Int64(i64),
    Uint64(u64),
    Float(f32),
    Double(f64),
    String(&'a str),
    Bytes(&'a [u8]),
}

impl<'a> MarkerMetaData<'a> {
    fn to_c_compatible_bytes(self) -> Vec<u8> {
        match self {
            MarkerMetaData::Int32(x) => x.to_ne_bytes().to_vec(),
            MarkerMetaData::Uint32(x) => x.to_ne_bytes().to_vec(),
            MarkerMetaData::Int64(x) => x.to_ne_bytes().to_vec(),
            MarkerMetaData::Uint64(x) => x.to_ne_bytes().to_vec(),
            MarkerMetaData::Float(x) => x.to_ne_bytes().to_vec(),
            MarkerMetaData::Double(x) => x.to_ne_bytes().to_vec(),
            MarkerMetaData::String(x) => CString::new(x).unwrap().as_bytes_with_nul().to_vec(),
            MarkerMetaData::Bytes(x) => x.to_vec(),
        }
    }
}

unsafe impl<T: MarkerMeta<N>, const N: usize> Send for ProfilerMarker<T, N> {}

pub trait MarkerMeta<const N: usize> {
    fn get_descriptors() -> [MarkerMetaDescriptor; N];
    fn get_data(&self) -> [MarkerMetaData; N];
}

impl MarkerMeta<0> for () {
    fn get_descriptors() -> [MarkerMetaDescriptor; 0] {
        []
    }

    fn get_data(&self) -> [MarkerMetaData; 0] {
        []
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MarkerDataType {
    InstanceId,
    Int32,
    Uint32,
    Int64,
    Uint64,
    Float,
    Double,
    String,
    String16,
    Blob8,
    GfxResourceId,
}

impl From<MarkerMetaData<'_>> for MarkerDataType {
    fn from(value: MarkerMetaData) -> Self {
        match value {
            MarkerMetaData::Int32(_) => MarkerDataType::Int32,
            MarkerMetaData::Uint32(_) => MarkerDataType::Uint32,
            MarkerMetaData::Int64(_) => MarkerDataType::Int64,
            MarkerMetaData::Uint64(_) => MarkerDataType::Uint64,
            MarkerMetaData::Float(_) => MarkerDataType::Float,
            MarkerMetaData::Double(_) => MarkerDataType::Double,
            MarkerMetaData::String(_) => MarkerDataType::String,
            MarkerMetaData::Bytes(_) => MarkerDataType::Blob8,
        }
    }
}

impl From<MarkerDataType> for ffi::UnityProfilerMarkerDataType {
    fn from(value: MarkerDataType) -> Self {
        match value {
            MarkerDataType::InstanceId => 1,
            MarkerDataType::Int32 => 2,
            MarkerDataType::Uint32 => 3,
            MarkerDataType::Int64 => 4,
            MarkerDataType::Uint64 => 5,
            MarkerDataType::Float => 6,
            MarkerDataType::Double => 7,
            MarkerDataType::String => 8,
            MarkerDataType::String16 => 9,
            MarkerDataType::Blob8 => 11,
            MarkerDataType::GfxResourceId => 12,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MarkerDataUnit {
    Undefined,
    Nanoseconds,
    Bytes,
    Count,
    Percent,
    FrequencyHz,
}

impl From<MarkerDataUnit> for ffi::UnityProfilerMarkerDataUnit {
    fn from(value: MarkerDataUnit) -> Self {
        match value {
            MarkerDataUnit::Undefined => 0,
            MarkerDataUnit::Nanoseconds => 1,
            MarkerDataUnit::Bytes => 2,
            MarkerDataUnit::Count => 3,
            MarkerDataUnit::Percent => 4,
            MarkerDataUnit::FrequencyHz => 5,
        }
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
    Marker(std::os::raw::c_int),

    #[error("Error returned by Unity during marker metadata creation: {0}")]
    MarkerMeta(std::os::raw::c_int),
}

pub enum ScopedProfilerSample<'a, 'b, T: MarkerMeta<N>, const N: usize> {
    Disabled,
    Enabled {
        marker: &'a ProfilerMarker<T, N>,
        profiler: &'b UnityProfiler,
    },
}

pub enum ManualProfilerSample<'a, 'b, T: MarkerMeta<N>, const N: usize> {
    Disabled,
    Enabled {
        ended: bool,
        marker: &'a ProfilerMarker<T, N>,
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

    pub fn create_marker(&mut self, name: &str) -> Result<ProfilerMarker<(), 0>, CreateMarkerErr> {
        self.create_marker_with_data::<(), 0>(name)
    }

    pub fn create_marker_with_data<MetaType: MarkerMeta<N>, const N: usize>(
        &mut self,
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
                ffi::UnityBuiltinProfilerCategory_::kUnityProfilerCategoryOther as u16,
                ffi::UnityProfilerMarkerFlag_::kUnityProfilerMarkerFlagDefault as u16,
                N as i32,
            );

            if create_result != 0 {
                return Err(CreateMarkerErr::Marker(create_result));
            }
        }

        for (i, &descriptor) in descriptors.iter().enumerate() {
            let descriptor_name_c = CString::new(descriptor.name).unwrap();

            let set_result = unsafe {
                let setmetafn = self.ptr.as_ref().SetMarkerMetadataName.unwrap();

                setmetafn(
                    raw_marker,
                    i as i32,
                    descriptor_name_c.as_ptr(),
                    descriptor.datatype.into(),
                    descriptor.unit.into(),
                )
            };

            if set_result != 0 {
                return Err(CreateMarkerErr::MarkerMeta(set_result));
            }
        }

        Ok(ProfilerMarker {
            desc_ptr: raw_marker,
            meta_type: PhantomData,
        })
    }

    fn emit_event<T: MarkerMeta<N>, const N: usize>(
        &self,
        marker: &ProfilerMarker<T, N>,
        event: EventType,
        meta: Option<&T>,
    ) {
        debug_assert!(self.is_enabled());
        debug_assert!(!marker.desc_ptr.is_null());

        unsafe {
            let emitfn = self.ptr.as_ref().EmitEvent.unwrap();

            match meta {
                None => emitfn(marker.desc_ptr, event.into(), 0, null()),
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

                    emitfn(marker.desc_ptr, event.into(), N as u16, unity_edata_ptr);
                }
            }
        }
    }
}

impl ProfilerMarker<(), 0> {
    pub fn sample_scope<'a, 'b>(
        &'a self,
        profiler: &'b UnityProfiler,
    ) -> ScopedProfilerSample<'a, 'b, (), 0> {
        self.sample_scope_with_meta(profiler, &())
    }

    pub fn sample_manual<'a, 'b>(
        &'a self,
        profiler: &'b UnityProfiler,
    ) -> ManualProfilerSample<'a, 'b, (), 0> {
        self.sample_manual_with_meta(profiler, &())
    }

    pub fn single_timeless(&self, profiler: &UnityProfiler) {
        self.single_timeless_with_meta(profiler, &())
    }
}

impl<T: MarkerMeta<N>, const N: usize> ProfilerMarker<T, N> {
    fn get_name(&self) -> &str {
        let name_c = unsafe { CStr::from_ptr(self.desc_ptr.as_ref().unwrap().name) };

        name_c.to_str().unwrap()
    }

    pub fn sample_scope_with_meta<'a, 'b>(
        &'a self,
        profiler: &'b UnityProfiler,
        meta: &T,
    ) -> ScopedProfilerSample<'a, 'b, T, N> {
        if !profiler.is_enabled() {
            return ScopedProfilerSample::Disabled;
        }

        profiler.emit_event(self, EventType::Begin, Some(meta));

        ScopedProfilerSample::Enabled {
            marker: self,
            profiler,
        }
    }

    pub fn sample_manual_with_meta<'a, 'b>(
        &'a self,
        profiler: &'b UnityProfiler,
        meta: &T,
    ) -> ManualProfilerSample<'a, 'b, T, N> {
        if !profiler.is_enabled() {
            return ManualProfilerSample::Disabled;
        }

        profiler.emit_event(self, EventType::Begin, Some(meta));

        ManualProfilerSample::Enabled {
            ended: false,
            marker: self,
            profiler,
        }
    }

    pub fn single_timeless_with_meta(&self, profiler: &UnityProfiler, meta: &T) {
        if !profiler.is_enabled() {
            return;
        }

        profiler.emit_event(self, EventType::Single, Some(meta));
    }
}

impl<'a, 'b, T: MarkerMeta<N>, const N: usize> Drop for ScopedProfilerSample<'a, 'b, T, N> {
    fn drop(&mut self) {
        match self {
            ScopedProfilerSample::Disabled => {}
            ScopedProfilerSample::Enabled { marker, profiler } => {
                profiler.emit_event(marker, EventType::End, None);
            }
        }
    }
}

impl<'a, 'b, T: MarkerMeta<N>, const N: usize> ManualProfilerSample<'a, 'b, T, N> {
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
                profiler.emit_event(marker, EventType::End, None);
            }
        }
    }
}

impl<'a, 'b, T: MarkerMeta<N>, const N: usize> Drop for ManualProfilerSample<'a, 'b, T, N> {
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
