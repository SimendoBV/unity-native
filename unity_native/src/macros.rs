#[macro_export]
macro_rules! unity_api_guid {
    ($high:literal $low:literal) => {
        $crate::ffi::UnityInterfaceGUID {
            m_GUIDHigh: $high,
            m_GUIDLow: $low,
        }
    };
}
