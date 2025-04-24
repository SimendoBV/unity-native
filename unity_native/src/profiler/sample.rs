use super::EventType;
use super::UnityProfiler;
use super::marker::MarkerMeta;
use super::marker::ProfilerMarker;

#[derive(Debug)]
pub enum ScopedProfilerSample<'a, 'b, T: MarkerMeta<N>, const N: usize> {
    Disabled,
    Enabled {
        marker: &'a ProfilerMarker<T, N>,
        profiler: &'b UnityProfiler,
    },
}

#[derive(Debug)]
pub enum ManualProfilerSample<'a, 'b, T: MarkerMeta<N>, const N: usize> {
    Disabled,
    Enabled {
        ended: bool,
        marker: &'a ProfilerMarker<T, N>,
        profiler: &'b UnityProfiler,
    },
}

impl<T: MarkerMeta<N>, const N: usize> Drop for ScopedProfilerSample<'_, '_, T, N> {
    fn drop(&mut self) {
        match self {
            ScopedProfilerSample::Disabled => {}
            ScopedProfilerSample::Enabled { marker, profiler } => {
                profiler.emit_event(marker, EventType::End, None);
            }
        }
    }
}

impl<T: MarkerMeta<N>, const N: usize> ManualProfilerSample<'_, '_, T, N> {
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

impl<T: MarkerMeta<N>, const N: usize> Drop for ManualProfilerSample<'_, '_, T, N> {
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
