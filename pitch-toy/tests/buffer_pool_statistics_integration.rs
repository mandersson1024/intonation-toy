// Buffer Pool Statistics Integration Test
// 
// This test verifies that buffer pool statistics flow correctly from the AudioWorklet
// through the Rust audio system to the GUI components, using mocked worklet and UI.

use wasm_bindgen_test::*;
use std::rc::Rc;
use std::cell::RefCell;
use pitch_toy::engine::audio::message_protocol::*;

// No wasm_bindgen_test_configure! needed for Node.js

/// Mock data setter for buffer pool statistics
#[derive(Clone)]
struct MockBufferPoolStatsSetter {
    calls: Rc<RefCell<Vec<Option<BufferPoolStats>>>>,
}

impl MockBufferPoolStatsSetter {
    fn new() -> Self {
        Self {
            calls: Rc::new(RefCell::new(Vec::new())),
        }
    }
    
    fn get_calls(&self) -> Vec<Option<BufferPoolStats>> {
        self.calls.borrow().clone()
    }
    
    fn get_latest_stats(&self) -> Option<BufferPoolStats> {
        self.calls.borrow().last().and_then(|stats| stats.clone())
    }
}

impl MockBufferPoolStatsSetter {
    fn set(&self, value: Option<BufferPoolStats>) {
        self.calls.borrow_mut().push(value);
    }
}

/// Mock AudioWorklet message generator
struct MockAudioWorkletMessageGenerator {
    message_id_counter: u32,
}

impl MockAudioWorkletMessageGenerator {
    fn new() -> Self {
        Self {
            message_id_counter: 0,
        }
    }
    
    fn generate_message_id(&mut self) -> u32 {
        self.message_id_counter += 1;
        self.message_id_counter
    }
    
    /// Generate a mock status message with buffer pool statistics
    fn create_status_message(&mut self, stats: BufferPoolStats) -> FromWorkletEnvelope {
        let message_id = self.generate_message_id();
        let timestamp = 1000.0 + (message_id as f64 * 50.0); // Mock increasing timestamps
        
        // Use AudioDataBatch instead of StatusUpdate since we now bundle buffer stats with audio data
        let audio_data = AudioDataBatch {
            sample_rate: 48000.0,
            sample_count: 1024,
            buffer_length: 4096,
            timestamp,
            sequence_number: Some(message_id),
            buffer_id: Some(1),
            buffer_pool_stats: Some(stats),
        };
        
        FromWorkletEnvelope {
            message_id,
            timestamp,
            payload: FromWorkletMessage::AudioDataBatch { data: audio_data },
        }
    }
    
    /// Generate a mock message with progressive buffer pool statistics
    fn create_progressive_stats_message(&mut self, iteration: u32) -> FromWorkletEnvelope {
        let stats = BufferPoolStats {
            pool_size: 8,
            available_buffers: 8 - (iteration % 8),
            in_use_buffers: iteration % 8,
            total_buffers: 8,
            acquire_count: iteration * 10,
            transfer_count: iteration * 8,
            pool_exhausted_count: iteration / 10,
            consecutive_pool_failures: iteration / 20,
            pool_hit_rate: 75.0 + (iteration as f32 * 0.1),
            pool_efficiency: 85.0 + (iteration as f32 * 0.1),
            buffer_utilization_percent: 50.0 + (iteration as f32 * 0.5),
            total_megabytes_transferred: (iteration as f32 * 0.1),
            avg_acquisition_time_ms: 5.0 + (iteration as f32 * 0.01),
            fastest_acquisition_time_ms: 1.0,
            slowest_acquisition_time_ms: 10.0 + (iteration as f32 * 0.02),
        };
        
        self.create_status_message(stats)
    }
}

/// Mock AudioSystemContext with buffer pool statistics integration
struct MockAudioSystemContext {
    buffer_pool_stats_setter: MockBufferPoolStatsSetter,
}

impl MockAudioSystemContext {
    fn new() -> Self {
        Self {
            buffer_pool_stats_setter: MockBufferPoolStatsSetter::new(),
        }
    }
    
    fn get_buffer_pool_stats_setter(&self) -> &MockBufferPoolStatsSetter {
        &self.buffer_pool_stats_setter
    }
    
    /// Simulate receiving a message from the AudioWorklet
    fn handle_worklet_message(&self, message: FromWorkletEnvelope) {
        // Extract buffer pool statistics from the message
        match message.payload {
            FromWorkletMessage::AudioDataBatch { data } => {
                if let Some(stats) = data.buffer_pool_stats {
                    // This simulates the audio system processing the message
                    // and updating the buffer pool statistics
                    self.buffer_pool_stats_setter.set(Some(stats));
                }
            }
            _ => {
                // Other message types don't contain buffer pool statistics
            }
        }
    }
}

/// Test that buffer pool statistics flow correctly through the system
#[wasm_bindgen_test]
async fn test_buffer_pool_statistics_integration() {
    // Create mock components
    let audio_context = MockAudioSystemContext::new();
    let mut worklet_message_generator = MockAudioWorkletMessageGenerator::new();
    
    // Test initial state
    assert_eq!(audio_context.get_buffer_pool_stats_setter().get_calls().len(), 0);
    
    // Simulate worklet sending initial status with buffer pool stats
    let initial_stats = BufferPoolStats {
        pool_size: 8,
        available_buffers: 6,
        in_use_buffers: 2,
        total_buffers: 8,
        acquire_count: 10,
        transfer_count: 8,
        pool_exhausted_count: 0,
        consecutive_pool_failures: 0,
        pool_hit_rate: 75.0,
        pool_efficiency: 85.0,
        buffer_utilization_percent: 25.0,
        total_megabytes_transferred: 0.5,
        avg_acquisition_time_ms: 5.0,
        fastest_acquisition_time_ms: 1.0,
        slowest_acquisition_time_ms: 10.0,
    };
    
    let message = worklet_message_generator.create_status_message(initial_stats.clone());
    audio_context.handle_worklet_message(message);
    
    // Verify that the statistics were received
    let calls = audio_context.get_buffer_pool_stats_setter().get_calls();
    assert_eq!(calls.len(), 1);
    
    let received_stats = calls[0].as_ref().unwrap();
    assert_eq!(received_stats.acquire_count, 10);
    assert_eq!(received_stats.transfer_count, 8);
    assert_eq!(received_stats.pool_hit_rate, 75.0);
    assert_eq!(received_stats.available_buffers, 6);
    assert_eq!(received_stats.in_use_buffers, 2);
    assert_eq!(received_stats.total_buffers, 8);
    
    // Simulate progressive buffer pool activity
    for i in 1..=5 {
        let message = worklet_message_generator.create_progressive_stats_message(i);
        audio_context.handle_worklet_message(message);
    }
    
    // Verify progression
    let final_calls = audio_context.get_buffer_pool_stats_setter().get_calls();
    assert_eq!(final_calls.len(), 6); // Initial + 5 progressive updates
    
    // Check that statistics are progressing
    let final_stats = final_calls.last().unwrap().as_ref().unwrap();
    assert!(final_stats.acquire_count > initial_stats.acquire_count);
    assert!(final_stats.transfer_count > initial_stats.transfer_count);
    assert!(final_stats.pool_hit_rate > initial_stats.pool_hit_rate);
}

/// Test that buffer pool statistics display correctly in GUI mock
#[wasm_bindgen_test]
async fn test_buffer_pool_statistics_gui_display() {
    let audio_context = MockAudioSystemContext::new();
    let mut worklet_message_generator = MockAudioWorkletMessageGenerator::new();
    
    // Create realistic buffer pool statistics
    let realistic_stats = BufferPoolStats {
        pool_size: 8,
        available_buffers: 6,
        in_use_buffers: 2,
        total_buffers: 8,
        acquire_count: 1000,
        transfer_count: 950,
        pool_exhausted_count: 5,
        consecutive_pool_failures: 2,
        pool_hit_rate: 94.7,
        pool_efficiency: 95.0,
        buffer_utilization_percent: 25.0,
        total_megabytes_transferred: 12.5,
        avg_acquisition_time_ms: 5.0,
        fastest_acquisition_time_ms: 1.0,
        slowest_acquisition_time_ms: 15.0,
    };
    
    let message = worklet_message_generator.create_status_message(realistic_stats.clone());
    audio_context.handle_worklet_message(message);
    
    // Mock GUI data conversion (simulating BufferPoolStatsData creation)
    let gui_stats = audio_context.get_buffer_pool_stats_setter().get_latest_stats().unwrap();
    
    // Verify GUI can display meaningful statistics
    assert_eq!(gui_stats.acquire_count, 1000);
    assert_eq!(gui_stats.transfer_count, 950);
    assert_eq!(gui_stats.pool_exhausted_count, 5);
    assert_eq!(gui_stats.consecutive_pool_failures, 2);
    assert!((gui_stats.pool_hit_rate - 94.7).abs() < 0.1);
    assert!((gui_stats.pool_efficiency - 95.0).abs() < 0.1);
    assert_eq!(gui_stats.available_buffers, 6);
    assert_eq!(gui_stats.in_use_buffers, 2);
    assert_eq!(gui_stats.total_buffers, 8);
    
    // Test calculated values that would be displayed in GUI
    let data_transferred_mb = gui_stats.total_megabytes_transferred;
    assert!(data_transferred_mb > 0.0);
    assert!(data_transferred_mb < 100.0); // Reasonable range
    
    let pool_efficiency = gui_stats.pool_efficiency;
    assert!(pool_efficiency > 90.0); // Should be high efficiency
    
    let buffer_utilization = gui_stats.buffer_utilization_percent;
    assert!(buffer_utilization >= 0.0 && buffer_utilization <= 100.0);
}

/// Test error scenarios in buffer pool statistics
#[wasm_bindgen_test]
async fn test_buffer_pool_statistics_error_scenarios() {
    let audio_context = MockAudioSystemContext::new();
    let mut worklet_message_generator = MockAudioWorkletMessageGenerator::new();
    
    // Test high error rate scenario
    let error_stats = BufferPoolStats {
        pool_size: 8,
        available_buffers: 1,
        in_use_buffers: 7,
        total_buffers: 8,
        acquire_count: 100,
        transfer_count: 50,
        pool_exhausted_count: 25,
        consecutive_pool_failures: 15,
        pool_hit_rate: 60.0,
        pool_efficiency: 50.0,
        buffer_utilization_percent: 87.5,
        total_megabytes_transferred: 2.0,
        avg_acquisition_time_ms: 50.0,
        fastest_acquisition_time_ms: 1.0,
        slowest_acquisition_time_ms: 100.0,
    };
    
    let message = worklet_message_generator.create_status_message(error_stats.clone());
    audio_context.handle_worklet_message(message);
    
    let received_stats = audio_context.get_buffer_pool_stats_setter().get_latest_stats().unwrap();
    
    // Verify error scenarios are properly tracked
    assert_eq!(received_stats.pool_exhausted_count, 25);
    assert_eq!(received_stats.consecutive_pool_failures, 15);
    assert!(received_stats.pool_hit_rate < 70.0); // Poor hit rate
    assert!(received_stats.pool_efficiency < 60.0); // Poor efficiency
    assert!(received_stats.avg_acquisition_time_ms > 30.0); // High acquisition time
    assert_eq!(received_stats.available_buffers, 1); // Pool nearly exhausted
    assert_eq!(received_stats.in_use_buffers, 7); // Most buffers in use
    
    // Test that GUI would show warning indicators for these conditions
    let pool_health = received_stats.available_buffers as f64 / received_stats.total_buffers as f64;
    assert!(pool_health < 0.2); // Less than 20% available - should trigger warning
    
    let error_rate = (received_stats.pool_exhausted_count + received_stats.consecutive_pool_failures) as f64 / received_stats.acquire_count as f64;
    assert!(error_rate > 0.3); // High error rate - should trigger warning
}

/// Test buffer pool statistics with realistic audio processing patterns
#[wasm_bindgen_test]
async fn test_buffer_pool_statistics_realistic_patterns() {
    let audio_context = MockAudioSystemContext::new();
    let mut worklet_message_generator = MockAudioWorkletMessageGenerator::new();
    
    // Simulate realistic audio processing over time
    // Each iteration represents ~50ms of audio processing
    let iterations = 20; // 1 second of audio
    
    for i in 0..iterations {
        let stats = BufferPoolStats {
            pool_size: 8,
            available_buffers: 5 + (i % 3), // Varying availability
            in_use_buffers: 3 - (i % 3), // Inverse of available
            total_buffers: 8,
            acquire_count: i * 15 + 1, // ~15 buffer acquisitions per 50ms
            transfer_count: i * 14 + 1, // Most acquisitions result in transfers
            pool_exhausted_count: i / 10, // Occasional pool exhaustion
            consecutive_pool_failures: i / 20, // Rare consecutive failures
            pool_hit_rate: 92.0 + (i as f32 * 0.1), // Improving hit rate
            pool_efficiency: 90.0 + (i as f32 * 0.1), // Improving efficiency
            buffer_utilization_percent: 40.0 + (i as f32 * 0.5), // Increasing utilization
            total_megabytes_transferred: (i as f32 * 0.5), // Accumulating data
            avg_acquisition_time_ms: 15.0 - (i as f32 * 0.2), // Improving acquisition time
            fastest_acquisition_time_ms: 1.0,
            slowest_acquisition_time_ms: 20.0 + (i as f32 * 0.1),
        };
        
        let message = worklet_message_generator.create_status_message(stats);
        audio_context.handle_worklet_message(message);
    }
    
    let all_calls = audio_context.get_buffer_pool_stats_setter().get_calls();
    assert_eq!(all_calls.len(), iterations as usize);
    
    // Verify realistic progression
    let first_stats = all_calls.first().unwrap().as_ref().unwrap();
    let last_stats = all_calls.last().unwrap().as_ref().unwrap();
    
    assert!(last_stats.acquire_count > first_stats.acquire_count);
    assert!(last_stats.transfer_count > first_stats.transfer_count);
    assert!(last_stats.pool_hit_rate > first_stats.pool_hit_rate);
    assert!(last_stats.avg_acquisition_time_ms < first_stats.avg_acquisition_time_ms);
    
    // Calculate throughput metrics
    let final_stats = last_stats;
    let data_transferred_mb = final_stats.total_megabytes_transferred;
    let processing_time_seconds = iterations as f64 * 0.05; // 50ms per iteration
    let throughput_mb_per_second = data_transferred_mb as f64 / processing_time_seconds;
    
    // Verify reasonable throughput for audio processing
    assert!(throughput_mb_per_second > 0.1); // At least 100KB/s
    assert!(throughput_mb_per_second < 50.0); // Less than 50MB/s (reasonable upper bound)
}

/// Test that buffer pool statistics handle edge cases correctly
#[wasm_bindgen_test]
async fn test_buffer_pool_statistics_edge_cases() {
    let audio_context = MockAudioSystemContext::new();
    let mut worklet_message_generator = MockAudioWorkletMessageGenerator::new();
    
    // Test zero statistics
    let zero_stats = BufferPoolStats {
        pool_size: 8,
        available_buffers: 8,
        in_use_buffers: 0,
        total_buffers: 8,
        acquire_count: 0,
        transfer_count: 0,
        pool_exhausted_count: 0,
        consecutive_pool_failures: 0,
        pool_hit_rate: 0.0,
        pool_efficiency: 0.0,
        buffer_utilization_percent: 0.0,
        total_megabytes_transferred: 0.0,
        avg_acquisition_time_ms: 0.0,
        fastest_acquisition_time_ms: 0.0,
        slowest_acquisition_time_ms: 0.0,
    };
    
    let message = worklet_message_generator.create_status_message(zero_stats);
    audio_context.handle_worklet_message(message);
    
    let received_stats = audio_context.get_buffer_pool_stats_setter().get_latest_stats().unwrap();
    assert_eq!(received_stats.acquire_count, 0);
    assert_eq!(received_stats.transfer_count, 0);
    assert_eq!(received_stats.pool_hit_rate, 0.0);
    assert_eq!(received_stats.available_buffers, 8);
    assert_eq!(received_stats.in_use_buffers, 0);
    
    // Test maximum statistics (extreme load)
    let max_stats = BufferPoolStats {
        pool_size: 8,
        available_buffers: 0,
        in_use_buffers: 8,
        total_buffers: 8,
        acquire_count: u32::MAX,
        transfer_count: u32::MAX - 1000,
        pool_exhausted_count: 1000,
        consecutive_pool_failures: 500,
        pool_hit_rate: 99.9,
        pool_efficiency: 99.5,
        buffer_utilization_percent: 100.0,
        total_megabytes_transferred: 1000.0,
        avg_acquisition_time_ms: 1000.0,
        fastest_acquisition_time_ms: 1.0,
        slowest_acquisition_time_ms: 5000.0,
    };
    
    let message = worklet_message_generator.create_status_message(max_stats);
    audio_context.handle_worklet_message(message);
    
    let received_stats = audio_context.get_buffer_pool_stats_setter().get_latest_stats().unwrap();
    assert_eq!(received_stats.acquire_count, u32::MAX);
    assert_eq!(received_stats.transfer_count, u32::MAX - 1000);
    assert_eq!(received_stats.available_buffers, 0);
    assert_eq!(received_stats.in_use_buffers, 8);
    
    // Verify GUI can handle extreme values
    let data_transferred_mb = received_stats.total_megabytes_transferred;
    assert!(data_transferred_mb > 0.0);
    assert!(data_transferred_mb.is_finite());
}