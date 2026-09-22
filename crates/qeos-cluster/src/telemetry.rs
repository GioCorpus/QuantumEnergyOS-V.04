//! P7.7-02 — Distributed telemetry with correlation IDs.
//!
//! Telemetry spans across nodes carry a [`TraceContext`] of correlation IDs
//! (trace, request, job, node, device, service, experiment). A
//! [`TelemetryCollector`] collects metric samples, logs, events and resource
//! usage in a bounded structure so long-running fleets do not grow unbounded.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Correlation IDs linking telemetry across a distributed operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub request_id: Option<String>,
    pub job_id: Option<String>,
    pub node_id: Option<String>,
    pub device_id: Option<String>,
    pub service_id: Option<String>,
    pub experiment_id: Option<String>,
}

impl TraceContext {
    pub fn new(trace_id: &str) -> Self {
        Self {
            trace_id: trace_id.to_string(),
            ..Default::default()
        }
    }
}

/// A single telemetry sample (metric-like).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    pub trace: TraceContext,
    pub name: String,
    pub value: f64,
    pub unit: String,
}

/// A telemetry log/event entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub trace: TraceContext,
    pub level: String,
    pub message: String,
}

/// A resource-utilization snapshot attached to a trace.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_percent: f64,
    pub ram_bytes: u64,
    pub gpu_percent: Option<f64>,
    pub vram_bytes: Option<u64>,
    pub network_throughput_bps: Option<u64>,
    pub temp_celsius: Option<f64>,
}

/// Bounded telemetry collector for a fleet.
#[derive(Debug)]
pub struct TelemetryCollector {
    metrics: VecDeque<MetricSample>,
    events: VecDeque<TelemetryEvent>,
    utilizations: VecDeque<ResourceUtilization>,
    limit: usize,
    total_metrics: u64,
    total_events: u64,
}

impl TelemetryCollector {
    pub fn new(limit: usize) -> Self {
        Self {
            metrics: VecDeque::new(),
            events: VecDeque::new(),
            utilizations: VecDeque::new(),
            limit: limit.max(1),
            total_metrics: 0,
            total_events: 0,
        }
    }

    /// Record a metric sample (bounded).
    pub fn record_metric(&mut self, sample: MetricSample) {
        if self.metrics.len() >= self.limit {
            self.metrics.pop_front();
        }
        self.metrics.push_back(sample);
        self.total_metrics += 1;
    }

    /// Record an event (bounded).
    pub fn record_event(&mut self, event: TelemetryEvent) {
        if self.events.len() >= self.limit {
            self.events.pop_front();
        }
        self.events.push_back(event);
        self.total_events += 1;
    }

    /// Record resource utilization (bounded).
    pub fn record_utilization(&mut self, util: ResourceUtilization) {
        if self.utilizations.len() >= self.limit {
            self.utilizations.pop_front();
        }
        self.utilizations.push_back(util);
    }

    pub fn metrics(&self) -> impl Iterator<Item = &MetricSample> {
        self.metrics.iter()
    }

    pub fn events(&self) -> impl Iterator<Item = &TelemetryEvent> {
        self.events.iter()
    }

    pub fn last_utilization(&self) -> Option<&ResourceUtilization> {
        self.utilizations.back()
    }

    /// Total samples/events ever recorded (monotonic counters for growth checks).
    pub fn totals(&self) -> (u64, u64) {
        (self.total_metrics, self.total_events)
    }

    /// Current buffered sizes (must stay bounded on long runs).
    pub fn buffered_counts(&self) -> (usize, usize, usize) {
        (
            self.metrics.len(),
            self.events.len(),
            self.utilizations.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(prefix: &str) -> TraceContext {
        TraceContext::new(prefix)
    }

    #[test]
    fn correlates_id_set() {
        let c = TraceContext {
            trace_id: "t1".into(),
            request_id: Some("r1".into()),
            job_id: Some("j1".into()),
            node_id: Some("n1".into()),
            device_id: Some("d1".into()),
            service_id: Some("s1".into()),
            experiment_id: Some("e1".into()),
        };
        assert_eq!(c.trace_id, "t1");
        assert_eq!(c.job_id.as_deref(), Some("j1"));
    }

    #[test]
    fn collector_stays_bounded() {
        let mut col = TelemetryCollector::new(10);
        for i in 0..50 {
            col.record_metric(MetricSample {
                trace: ctx(&format!("t{i}")),
                name: "cpu".into(),
                value: i as f64,
                unit: "%".into(),
            });
        }
        // Buffer stays at the limit; cumulative counter grows.
        let (metrics, _, _) = col.buffered_counts();
        assert_eq!(metrics, 10);
        assert_eq!(col.totals().0, 50);
    }

    #[test]
    fn event_and_utilization() {
        let mut col = TelemetryCollector::new(10);
        col.record_event(TelemetryEvent {
            trace: ctx("t"),
            level: "info".into(),
            message: "boot".into(),
        });
        col.record_utilization(ResourceUtilization {
            cpu_percent: 12.0,
            ..Default::default()
        });
        assert_eq!(col.events().count(), 1);
        assert_eq!(col.last_utilization().unwrap().cpu_percent, 12.0);
    }
}
