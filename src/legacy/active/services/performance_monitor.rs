use crate::legacy::active::services::browser_compat::BrowserInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{console, window, Performance};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBaseline {
    pub browser_name: String,
    pub browser_version: String,
    pub wasm_loading_time_ms: f64,
    pub audio_context_creation_ms: f64,
    pub initial_render_time_ms: f64,
    pub memory_usage_mb: f64,
    pub audio_latency_ms: f64,
    pub compatibility_score: f64,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub wasm_load_start: f64,
    pub wasm_load_end: f64,
    pub audio_context_start: f64,
    pub audio_context_end: f64,
    pub render_start: f64,
    pub render_end: f64,
    pub baseline_established: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserPerformanceReport {
    pub browser_info: String,
    pub performance_grade: PerformanceGrade,
    pub baseline: PerformanceBaseline,
    pub recommendations: Vec<String>,
    pub performance_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceGrade {
    Excellent,  // < 100ms total load time
    Good,       // 100-300ms
    Fair,       // 300-1000ms
    Poor,       // > 1000ms
}

pub struct PerformanceMonitor {
    performance: Option<Performance>,
    metrics: PerformanceMetrics,
    browser_info: Option<BrowserInfo>,
    baselines: HashMap<String, PerformanceBaseline>,
    monitoring_enabled: bool,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        let performance = window()
            .map(|w| w.performance());

        Self {
            performance: performance.flatten(),
            metrics: PerformanceMetrics {
                wasm_load_start: 0.0,
                wasm_load_end: 0.0,
                audio_context_start: 0.0,
                audio_context_end: 0.0,
                render_start: 0.0,
                render_end: 0.0,
                baseline_established: false,
            },
            browser_info: None,
            baselines: HashMap::new(),
            monitoring_enabled: true,
        }
    }

    pub fn initialize(&mut self, browser_info: BrowserInfo) {
        self.browser_info = Some(browser_info);
        self.start_render_timing();
    }

    pub fn start_wasm_loading(&mut self) {
        if let Some(perf) = &self.performance {
            self.metrics.wasm_load_start = perf.now();
            console::log_1(&"[Performance] WASM loading started".into());
        }
    }

    pub fn end_wasm_loading(&mut self) {
        if let Some(perf) = &self.performance {
            self.metrics.wasm_load_end = perf.now();
            let load_time = self.metrics.wasm_load_end - self.metrics.wasm_load_start;
            console::log_1(&format!("[Performance] WASM loaded in {:.2}ms", load_time).into());
        }
    }

    pub fn start_audio_context_creation(&mut self) {
        if let Some(perf) = &self.performance {
            self.metrics.audio_context_start = perf.now();
            console::log_1(&"[Performance] AudioContext creation started".into());
        }
    }

    pub fn end_audio_context_creation(&mut self) {
        if let Some(perf) = &self.performance {
            self.metrics.audio_context_end = perf.now();
            let create_time = self.metrics.audio_context_end - self.metrics.audio_context_start;
            console::log_1(&format!("[Performance] AudioContext created in {:.2}ms", create_time).into());
        }
    }

    pub fn start_render_timing(&mut self) {
        if let Some(perf) = &self.performance {
            self.metrics.render_start = perf.now();
        }
    }

    pub fn end_render_timing(&mut self) {
        if let Some(perf) = &self.performance {
            self.metrics.render_end = perf.now();
            let render_time = self.metrics.render_end - self.metrics.render_start;
            console::log_1(&format!("[Performance] Initial render completed in {:.2}ms", render_time).into());
        }
    }

    pub fn establish_baseline(&mut self) -> Option<PerformanceBaseline> {
        if let (Some(browser_info), Some(perf)) = (&self.browser_info, &self.performance) {
            let wasm_loading_time = self.metrics.wasm_load_end - self.metrics.wasm_load_start;
            let audio_context_time = self.metrics.audio_context_end - self.metrics.audio_context_start;
            let render_time = self.metrics.render_end - self.metrics.render_start;

            let baseline = PerformanceBaseline {
                browser_name: browser_info.browser_name.clone(),
                browser_version: browser_info.browser_version
                    .as_ref()
                    .map(|v| format!("{}.{}.{}", v.major, v.minor, v.patch))
                    .unwrap_or_else(|| "Unknown".to_string()),
                wasm_loading_time_ms: wasm_loading_time,
                audio_context_creation_ms: audio_context_time,
                initial_render_time_ms: render_time,
                memory_usage_mb: self.get_memory_usage(),
                audio_latency_ms: self.measure_audio_latency(),
                compatibility_score: self.calculate_compatibility_score(browser_info),
                timestamp: perf.now(),
            };

            let browser_key = format!("{}_{}", 
                baseline.browser_name, 
                baseline.browser_version
            );
            
            self.baselines.insert(browser_key, baseline.clone());
            self.metrics.baseline_established = true;

            console::log_1(&format!(
                "[Performance] Baseline established - WASM: {:.2}ms, Audio: {:.2}ms, Render: {:.2}ms",
                wasm_loading_time, audio_context_time, render_time
            ).into());

            Some(baseline)
        } else {
            None
        }
    }

    fn get_memory_usage(&self) -> f64 {
        // Use the Performance API to get memory information if available
        if let Some(perf) = &self.performance {
            js_sys::eval("performance.memory ? performance.memory.usedJSHeapSize / 1024 / 1024 : 0")
                .ok()
                .and_then(|val| val.as_f64())
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    fn measure_audio_latency(&self) -> f64 {
        // This would ideally measure actual audio latency
        // For now, we'll estimate based on browser capabilities
        if let Some(browser_info) = &self.browser_info {
            if browser_info.capabilities.supports_audio_worklet {
                20.0 // AudioWorklet typically has ~20ms latency
            } else {
                50.0 // ScriptProcessorNode typically has ~50ms latency
            }
        } else {
            100.0 // Unknown - assume worst case
        }
    }

    fn calculate_compatibility_score(&self, browser_info: &BrowserInfo) -> f64 {
        let mut score = 0.0;
        let total_features = 8.0;

        // Core features (more weight)
        if browser_info.capabilities.supports_wasm { score += 2.0; }
        if browser_info.capabilities.supports_audio_context { score += 2.0; }
        if browser_info.capabilities.supports_media_devices { score += 1.0; }

        // Advanced features
        if browser_info.capabilities.supports_wasm_streaming { score += 1.0; }
        if browser_info.capabilities.supports_audio_worklet { score += 1.0; }
        if browser_info.capabilities.performance_api { score += 0.5; }
        if browser_info.capabilities.supports_shared_array_buffer { score += 0.5; }

        (score / total_features) * 100.0
    }

    pub fn generate_performance_report(&self) -> Option<BrowserPerformanceReport> {
        if let (Some(browser_info), Some(baseline)) = (
            &self.browser_info,
            self.get_current_baseline()
        ) {
            let performance_grade = self.determine_performance_grade(&baseline);
            let recommendations = self.generate_performance_recommendations(&baseline, &performance_grade);
            let performance_issues = self.identify_performance_issues(&baseline);

            Some(BrowserPerformanceReport {
                browser_info: format!(
                    "{} {}",
                    browser_info.browser_name,
                    browser_info.browser_version
                        .as_ref()
                        .map(|v| format!("{}.{}.{}", v.major, v.minor, v.patch))
                        .unwrap_or_else(|| "Unknown".to_string())
                ),
                performance_grade,
                baseline: baseline.clone(),
                recommendations,
                performance_issues,
            })
        } else {
            None
        }
    }

    fn get_current_baseline(&self) -> Option<&PerformanceBaseline> {
        if let Some(browser_info) = &self.browser_info {
            let browser_key = format!(
                "{}_{}",
                browser_info.browser_name,
                browser_info.browser_version
                    .as_ref()
                    .map(|v| format!("{}.{}.{}", v.major, v.minor, v.patch))
                    .unwrap_or_else(|| "Unknown".to_string())
            );
            self.baselines.get(&browser_key)
        } else {
            None
        }
    }

    fn determine_performance_grade(&self, baseline: &PerformanceBaseline) -> PerformanceGrade {
        let total_load_time = baseline.wasm_loading_time_ms + 
                             baseline.audio_context_creation_ms + 
                             baseline.initial_render_time_ms;

        match total_load_time {
            t if t < 100.0 => PerformanceGrade::Excellent,
            t if t < 300.0 => PerformanceGrade::Good,
            t if t < 1000.0 => PerformanceGrade::Fair,
            _ => PerformanceGrade::Poor,
        }
    }

    fn generate_performance_recommendations(&self, baseline: &PerformanceBaseline, grade: &PerformanceGrade) -> Vec<String> {
        let mut recommendations = Vec::new();

        match grade {
            PerformanceGrade::Poor => {
                recommendations.push("Consider upgrading your browser for better performance".to_string());
                if baseline.wasm_loading_time_ms > 500.0 {
                    recommendations.push("WebAssembly loading is slow - check your internet connection".to_string());
                }
                if baseline.audio_latency_ms > 100.0 {
                    recommendations.push("High audio latency detected - use headphones for better experience".to_string());
                }
            }
            PerformanceGrade::Fair => {
                recommendations.push("Performance is acceptable but could be improved".to_string());
                if baseline.memory_usage_mb > 50.0 {
                    recommendations.push("High memory usage detected - close other tabs".to_string());
                }
            }
            PerformanceGrade::Good => {
                recommendations.push("Good performance - no immediate action needed".to_string());
            }
            PerformanceGrade::Excellent => {
                recommendations.push("Excellent performance - optimal configuration".to_string());
            }
        }

        recommendations
    }

    fn identify_performance_issues(&self, baseline: &PerformanceBaseline) -> Vec<String> {
        let mut issues = Vec::new();

        if baseline.wasm_loading_time_ms > 1000.0 {
            issues.push("Very slow WebAssembly loading".to_string());
        } else if baseline.wasm_loading_time_ms > 500.0 {
            issues.push("Slow WebAssembly loading".to_string());
        }

        if baseline.audio_context_creation_ms > 100.0 {
            issues.push("Slow AudioContext creation".to_string());
        }

        if baseline.initial_render_time_ms > 200.0 {
            issues.push("Slow initial render".to_string());
        }

        if baseline.memory_usage_mb > 100.0 {
            issues.push("High memory usage".to_string());
        } else if baseline.memory_usage_mb > 50.0 {
            issues.push("Moderate memory usage".to_string());
        }

        if baseline.audio_latency_ms > 100.0 {
            issues.push("High audio latency".to_string());
        }

        issues
    }

    pub fn get_performance_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    pub fn get_baselines(&self) -> &HashMap<String, PerformanceBaseline> {
        &self.baselines
    }

    pub fn export_baseline_data(&self) -> String {
        serde_json::to_string_pretty(&self.baselines)
            .unwrap_or_else(|_| "{}".to_string())
    }

    pub fn enable_monitoring(&mut self, enabled: bool) {
        self.monitoring_enabled = enabled;
    }
}

// JavaScript-callable functions for performance monitoring
#[wasm_bindgen]
pub struct JsPerformanceMonitor {
    inner: PerformanceMonitor,
}

#[wasm_bindgen]
impl JsPerformanceMonitor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: PerformanceMonitor::new(),
        }
    }

    #[wasm_bindgen(js_name = startWasmLoading)]
    pub fn start_wasm_loading(&mut self) {
        self.inner.start_wasm_loading();
    }

    #[wasm_bindgen(js_name = endWasmLoading)]
    pub fn end_wasm_loading(&mut self) {
        self.inner.end_wasm_loading();
    }

    #[wasm_bindgen(js_name = establishBaseline)]
    pub fn establish_baseline(&mut self) -> Option<String> {
        self.inner.establish_baseline()
            .and_then(|baseline| serde_json::to_string(&baseline).ok())
    }

    #[wasm_bindgen(js_name = exportData)]
    pub fn export_data(&self) -> String {
        self.inner.export_baseline_data()
    }
} 