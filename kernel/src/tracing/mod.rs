//! Minimal kernel tracing (§48): correlate cpu/thread/process/device/job + mono timestamp.

#[derive(Debug, Clone, Copy, Default)]
pub struct TraceCtx {
    pub cpu_id: u32,
    pub thread_id: u32,
    pub process_id: u32,
    pub device_id: u32,
    pub job_id: u64,
    pub trace_id: u64,
    pub ts_mono_ns: u64,
}

impl TraceCtx {
    pub fn new(cpu_id: u32, thread_id: u32, process_id: u32, ts_mono_ns: u64) -> Self {
        Self {
            cpu_id,
            thread_id,
            process_id,
            device_id: 0,
            job_id: 0,
            trace_id: 0,
            ts_mono_ns,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ctx() {
        let c = TraceCtx::new(0, 1, 1, 99);
        assert_eq!(c.ts_mono_ns, 99);
    }
}
