// Metrics Display - Performance metrics rendering utilities
//
// This module provides utilities for displaying performance metrics in the live panel.
// It handles formatting and visualization of FPS, memory usage, latency, and CPU data.

use yew::prelude::*;
use super::component::PerformanceMetrics;

/// Metrics display utilities
#[allow(dead_code)]
pub struct MetricsDisplay;

#[allow(dead_code)]
impl MetricsDisplay {
    /// Render performance metrics grid
    pub fn render_metrics_grid(metrics: &PerformanceMetrics) -> Html {
        html! {
            <div class="metrics-grid">
                {Self::render_metric("FPS", &format!("{:.1}", metrics.fps), Self::get_fps_status(metrics.fps))}
                {Self::render_metric("Memory", &format!("{:.1} MB", metrics.memory_usage), Self::get_memory_status(metrics.memory_usage))}
                {Self::render_metric("Audio Latency", &format!("{:.1} ms", metrics.audio_latency), Self::get_latency_status(metrics.audio_latency))}
                {Self::render_metric("CPU Usage", &format!("{:.1}%", metrics.cpu_usage), Self::get_cpu_status(metrics.cpu_usage))}
            </div>
        }
    }

    /// Render a single metric item
    fn render_metric(label: &str, value: &str, status: MetricStatus) -> Html {
        html! {
            <div class={format!("metric-item metric-{}", status.css_class())}>
                <span class="metric-label">{label}</span>
                <span class="metric-value">{value}</span>
                <span class="metric-indicator">{status.icon()}</span>
            </div>
        }
    }

    /// Get FPS status indicator
    fn get_fps_status(fps: f64) -> MetricStatus {
        if fps >= 55.0 {
            MetricStatus::Good
        } else if fps >= 30.0 {
            MetricStatus::Warning
        } else {
            MetricStatus::Critical
        }
    }

    /// Get memory usage status indicator
    fn get_memory_status(memory_mb: f64) -> MetricStatus {
        if memory_mb <= 50.0 {
            MetricStatus::Good
        } else if memory_mb <= 100.0 {
            MetricStatus::Warning
        } else {
            MetricStatus::Critical
        }
    }

    /// Get audio latency status indicator
    fn get_latency_status(latency_ms: f64) -> MetricStatus {
        if latency_ms <= 30.0 {
            MetricStatus::Good
        } else if latency_ms <= 50.0 {
            MetricStatus::Warning
        } else {
            MetricStatus::Critical
        }
    }

    /// Get CPU usage status indicator
    fn get_cpu_status(cpu_percent: f64) -> MetricStatus {
        if cpu_percent <= 60.0 {
            MetricStatus::Good
        } else if cpu_percent <= 80.0 {
            MetricStatus::Warning
        } else {
            MetricStatus::Critical
        }
    }
}

/// Status indicators for metrics
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum MetricStatus {
    Good,
    Warning,
    Critical,
}

#[allow(dead_code)]
impl MetricStatus {
    /// Get CSS class for the status
    pub fn css_class(&self) -> &'static str {
        match self {
            MetricStatus::Good => "good",
            MetricStatus::Warning => "warning",
            MetricStatus::Critical => "critical",
        }
    }

    /// Get icon for the status
    pub fn icon(&self) -> &'static str {
        match self {
            MetricStatus::Good => "✅",
            MetricStatus::Warning => "⚠️",
            MetricStatus::Critical => "❌",
        }
    }
}