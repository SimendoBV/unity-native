use std::ffi::CStr;
use std::{ffi::CString, marker::PhantomData};

use crate::ffi;

use super::sample::ManualProfilerSample;
use super::sample::ScopedProfilerSample;
use super::EventType;
use super::UnityProfiler;

pub struct ProfilerMarker<T: MarkerMeta<N> = (), const N: usize = 0> {
    desc_ptr: *const ffi::UnityProfilerMarkerDesc,
    meta_type: PhantomData<T>,
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

#[derive(Debug, Clone)]
pub struct MarkerMetaDescriptor {
    name: CString,
    pub(super) datatype: MarkerDataType,
    pub(super) unit: MarkerDataUnit,
}

impl MarkerMetaDescriptor {
    pub fn new(name: &'static str, datatype: MarkerDataType, unit: MarkerDataUnit) -> Self {
        Self {
            name: CString::new(name).unwrap(),
            datatype,
            unit,
        }
    }

    pub fn name_c(&self) -> &CStr {
        self.name.as_c_str()
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

macro_rules! impl_from {
    ($srctype:ty, $variant:ident) => {
        impl<'a> From<$srctype> for MarkerMetaData<'a> {
            fn from(value: $srctype) -> Self {
                Self::$variant(value)
            }
        }
    };
}

impl_from!(i32, Int32);
impl_from!(u32, Uint32);
impl_from!(i64, Int64);
impl_from!(u64, Uint64);
impl_from!(f32, Float);
impl_from!(f64, Double);
impl_from!(&'a str, String);
impl_from!(&'a [u8], Bytes);

impl<'a> MarkerMetaData<'a> {
    pub(super) fn to_c_compatible_bytes(self) -> Vec<u8> {
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
    pub(super) unsafe fn new(raw_ptr: *const ffi::UnityProfilerMarkerDesc) -> Self {
        Self {
            desc_ptr: raw_ptr,
            meta_type: PhantomData,
        }
    }

    pub(super) fn raw(&self) -> *const ffi::UnityProfilerMarkerDesc {
        self.desc_ptr
    }

    pub fn get_name(&self) -> &str {
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
