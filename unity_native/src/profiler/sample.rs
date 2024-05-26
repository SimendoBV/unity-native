use super::marker::MarkerMeta;
use super::marker::ProfilerMarker;
use super::EventType;
use super::UnityProfiler;

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
