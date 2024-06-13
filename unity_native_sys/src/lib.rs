#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

macro_rules! impl_from_simple {
    ($from:ty, $to:ty) => {
        impl From<$from> for $to {
            fn from(value: $from) -> Self {
                <$to>::try_from(value.0).expect("Could not convert type")
            }
        }
    };
}

impl_from_simple!(UnityProfilerMarkerEventType_, UnityProfilerMarkerEventType);
impl_from_simple!(UnityBuiltinProfilerCategory_, UnityProfilerCategoryId);
impl_from_simple!(UnityProfilerMarkerFlag_, UnityProfilerMarkerFlags);
