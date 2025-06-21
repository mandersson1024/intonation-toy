// Audio processing module exports
pub mod engine;
pub mod pitch_detector;
pub mod educational_validator;
pub mod stress_tester;
pub mod test_reporter;
pub mod performance_bench;
pub mod realtime_processor;
pub mod performance_monitor;
pub mod signal_analyzer;

pub use engine::AudioEngine;
pub use pitch_detector::PitchDetector;
pub use educational_validator::EducationalValidator;
pub use stress_tester::StressTester;
pub use test_reporter::TestReporter;
pub use performance_bench::PerformanceBenchmark;
pub use realtime_processor::{RealtimeProcessor, RealtimeProcessingResult};
pub use performance_monitor::{PerformanceMonitor, PerformanceMetrics, PipelineStatus};
pub use signal_analyzer::{SignalAnalyzer, AudioAnalysis, BufferConfig, BufferConstraints};
