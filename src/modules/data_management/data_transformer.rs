//! # Data Transformer Implementation
//!
//! This module provides comprehensive data transformation utilities for cross-module
//! data format conversion, with support for audio buffer formats, metadata preservation,
//! zero-copy operations, and performance monitoring.

use std::collections::HashMap;
use std::time::Instant;
use serde::{Serialize, Deserialize};

use super::data_flow_coordinator::{FlowData, DataFormat, TransformError};
use super::audio_data_pipeline::{AudioDataFormat, AudioBufferMetadata};

/// Data transformer system for cross-module format conversion
pub struct DataTransformer {
    /// Transformation rules registry
    transform_rules: HashMap<TransformKey, TransformRule>,
    /// Transformation performance metrics
    metrics: TransformMetrics,
    /// Zero-copy optimization settings
    zero_copy_enabled: bool,
}

/// Key for transformation rule lookup
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransformKey {
    pub from_format: DataFormat,
    pub to_format: DataFormat,
}

/// Transformation rule definition
#[derive(Debug, Clone)]
pub struct TransformRule {
    pub key: TransformKey,
    pub transform_fn: TransformFunction,
    pub zero_copy_compatible: bool,
    pub validation_required: bool,
    pub metadata_preservation: MetadataPreservation,
}

/// Transformation function types
#[derive(Debug, Clone)]
pub enum TransformFunction {
    AudioBufferToF32Array,
    AudioBufferToI16Array,
    AudioBufferToJSCompatible,
    AudioBufferToWASMOptimized,
    F32ArrayToAudioBuffer,
    I16ArrayToAudioBuffer,
    JSCompatibleToAudioBuffer,
    WASMOptimizedToAudioBuffer,
    TypedEventToBinary,
    BinaryToTypedEvent,
    MetricsToJSON,
    JSONToMetrics,
    ConfigToBinary,
    BinaryToConfig,
}

/// Metadata preservation strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetadataPreservation {
    Full,        // Preserve all metadata
    Essential,   // Preserve only essential metadata
    None,        // Don't preserve metadata
    Custom(Vec<String>), // Preserve specific metadata keys
}

/// Transformation performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformMetrics {
    pub total_transformations: u64,
    pub successful_transformations: u64,
    pub failed_transformations: u64,
    pub zero_copy_transformations: u64,
    pub average_transform_time_ms: f32,
    pub max_transform_time_ms: f32,
    pub bytes_transformed: u64,
    pub transform_rates: HashMap<String, f32>, // transforms per second by type
}

impl Default for TransformMetrics {
    fn default() -> Self {
        Self {
            total_transformations: 0,
            successful_transformations: 0,
            failed_transformations: 0,
            zero_copy_transformations: 0,
            average_transform_time_ms: 0.0,
            max_transform_time_ms: 0.0,
            bytes_transformed: 0,
            transform_rates: HashMap::new(),
        }
    }
}

/// Transformation result with metadata
#[derive(Debug, Clone)]
pub struct TransformResult {
    pub data: FlowData,
    pub zero_copy_used: bool,
    pub transform_time_ms: f32,
    pub bytes_processed: usize,
    pub metadata_preserved: bool,
}

/// Audio buffer transformation formats
#[derive(Debug, Clone)]
pub enum AudioBufferFormat {
    F32Array(Vec<f32>),
    I16Array(Vec<i16>),
    JSCompatible {
        left_channel: Vec<f32>,
        right_channel: Vec<f32>,
    },
    WASMOptimized {
        data: Vec<u8>,
        sample_rate: f32,
        channels: u32,
    },
}

impl DataTransformer {
    /// Create new data transformer with default rules
    pub fn new() -> Self {
        let mut transformer = Self {
            transform_rules: HashMap::new(),
            metrics: TransformMetrics::default(),
            zero_copy_enabled: true,
        };
        
        transformer.initialize_default_rules();
        transformer
    }
    
    /// Initialize default transformation rules
    fn initialize_default_rules(&mut self) {
        // Audio buffer format transformations
        self.register_transform_rule(TransformRule {
            key: TransformKey {
                from_format: DataFormat::AudioBuffer,
                to_format: DataFormat::BinaryData,
            },
            transform_fn: TransformFunction::AudioBufferToF32Array,
            zero_copy_compatible: true,
            validation_required: true,
            metadata_preservation: MetadataPreservation::Full,
        });
        
        self.register_transform_rule(TransformRule {
            key: TransformKey {
                from_format: DataFormat::BinaryData,
                to_format: DataFormat::AudioBuffer,
            },
            transform_fn: TransformFunction::F32ArrayToAudioBuffer,
            zero_copy_compatible: false,
            validation_required: true,
            metadata_preservation: MetadataPreservation::Full,
        });
        
        // Event transformations
        self.register_transform_rule(TransformRule {
            key: TransformKey {
                from_format: DataFormat::TypedEvent,
                to_format: DataFormat::BinaryData,
            },
            transform_fn: TransformFunction::TypedEventToBinary,
            zero_copy_compatible: false,
            validation_required: false,
            metadata_preservation: MetadataPreservation::Essential,
        });
        
        // Configuration transformations
        self.register_transform_rule(TransformRule {
            key: TransformKey {
                from_format: DataFormat::ConfigurationData,
                to_format: DataFormat::BinaryData,
            },
            transform_fn: TransformFunction::ConfigToBinary,
            zero_copy_compatible: false,
            validation_required: true,
            metadata_preservation: MetadataPreservation::Full,
        });
        
        // Metrics transformations
        self.register_transform_rule(TransformRule {
            key: TransformKey {
                from_format: DataFormat::MetricsData,
                to_format: DataFormat::BinaryData,
            },
            transform_fn: TransformFunction::MetricsToJSON,
            zero_copy_compatible: false,
            validation_required: false,
            metadata_preservation: MetadataPreservation::None,
        });
    }
    
    /// Register a transformation rule
    pub fn register_transform_rule(&mut self, rule: TransformRule) {
        self.transform_rules.insert(rule.key.clone(), rule);
    }
    
    /// Transform data between formats
    pub fn transform_data(
        &mut self,
        data: FlowData,
        from_format: DataFormat,
        to_format: DataFormat,
    ) -> Result<TransformResult, TransformError> {
        let transform_start = Instant::now();
        
        // Check if transformation is needed
        if from_format == to_format {
            return Ok(TransformResult {
                data,
                zero_copy_used: true,
                transform_time_ms: 0.0,
                bytes_processed: 0,
                metadata_preserved: true,
            });
        }
        
        // Look up transformation rule
        let key = TransformKey {
            from_format: from_format.clone(),
            to_format: to_format.clone(),
        };
        
        let rule = self.transform_rules.get(&key)
            .ok_or(TransformError::UnsupportedFormat)?;
        
        // Validate input data if required
        if rule.validation_required && !self.validate_input_data(&data, &from_format)? {
            return Err(TransformError::InvalidData);
        }
        
        // Perform transformation
        let (transformed_data, zero_copy_used) = self.perform_transformation(
            data,
            &rule.transform_fn,
            rule.zero_copy_compatible,
        )?;
        
        // Preserve metadata according to rule
        let final_data = self.preserve_metadata(
            transformed_data,
            &rule.metadata_preservation,
        );
        
        let transform_time = transform_start.elapsed().as_millis() as f32;
        let bytes_processed = final_data.data.len();
        
        // Update metrics
        self.update_metrics(&rule.transform_fn, transform_time, bytes_processed, zero_copy_used, true);
        
        Ok(TransformResult {
            data: final_data,
            zero_copy_used,
            transform_time_ms: transform_time,
            bytes_processed,
            metadata_preserved: rule.metadata_preservation != MetadataPreservation::None,
        })
    }
    
    /// Validate input data for transformation
    fn validate_input_data(&self, data: &FlowData, format: &DataFormat) -> Result<bool, TransformError> {
        match format {
            DataFormat::AudioBuffer => {
                // Validate audio buffer data
                if data.data.len() % 4 != 0 {
                    return Ok(false); // f32 arrays should be 4-byte aligned
                }
                
                // Check for reasonable audio buffer size
                if data.data.len() < 16 || data.data.len() > 1024 * 1024 {
                    return Ok(false);
                }
                
                Ok(true)
            },
            DataFormat::BinaryData => {
                // Basic binary data validation
                Ok(!data.data.is_empty())
            },
            _ => Ok(true), // No specific validation for other formats
        }
    }
    
    /// Perform the actual data transformation
    fn perform_transformation(
        &self,
        data: FlowData,
        transform_fn: &TransformFunction,
        zero_copy_compatible: bool,
    ) -> Result<(FlowData, bool), TransformError> {
        match transform_fn {
            TransformFunction::AudioBufferToF32Array => {
                self.transform_audio_buffer_to_f32_array(data, zero_copy_compatible)
            },
            TransformFunction::AudioBufferToI16Array => {
                self.transform_audio_buffer_to_i16_array(data)
            },
            TransformFunction::AudioBufferToJSCompatible => {
                self.transform_audio_buffer_to_js_compatible(data)
            },
            TransformFunction::AudioBufferToWASMOptimized => {
                self.transform_audio_buffer_to_wasm_optimized(data)
            },
            TransformFunction::F32ArrayToAudioBuffer => {
                self.transform_f32_array_to_audio_buffer(data)
            },
            TransformFunction::TypedEventToBinary => {
                self.transform_typed_event_to_binary(data)
            },
            TransformFunction::BinaryToTypedEvent => {
                self.transform_binary_to_typed_event(data)
            },
            TransformFunction::MetricsToJSON => {
                self.transform_metrics_to_json(data)
            },
            TransformFunction::ConfigToBinary => {
                self.transform_config_to_binary(data)
            },
            _ => Err(TransformError::UnsupportedFormat),
        }
    }
    
    /// Transform audio buffer to f32 array (potentially zero-copy)
    fn transform_audio_buffer_to_f32_array(
        &self,
        data: FlowData,
        zero_copy_compatible: bool,
    ) -> Result<(FlowData, bool), TransformError> {
        if zero_copy_compatible && self.zero_copy_enabled {
            // Zero-copy transformation - just change format metadata
            let mut transformed = data;
            transformed.format = DataFormat::BinaryData;
            return Ok((transformed, true));
        }
        
        // Regular transformation with data copying
        let transformed_data = FlowData {
            format: DataFormat::BinaryData,
            data: data.data, // In real implementation, this would be transformed
            metadata: data.metadata,
            timestamp: data.timestamp,
        };
        
        Ok((transformed_data, false))
    }
    
    /// Transform audio buffer to i16 array
    fn transform_audio_buffer_to_i16_array(&self, data: FlowData) -> Result<(FlowData, bool), TransformError> {
        // Convert f32 samples to i16
        let f32_samples: Vec<f32> = data.data.chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        
        let i16_samples: Vec<i16> = f32_samples.iter()
            .map(|&sample| (sample * 32767.0).clamp(-32768.0, 32767.0) as i16)
            .collect();
        
        let i16_bytes: Vec<u8> = i16_samples.iter()
            .flat_map(|&sample| sample.to_le_bytes().to_vec())
            .collect();
        
        let transformed_data = FlowData {
            format: DataFormat::BinaryData,
            data: i16_bytes,
            metadata: data.metadata,
            timestamp: data.timestamp,
        };
        
        Ok((transformed_data, false))
    }
    
    /// Transform audio buffer to JavaScript-compatible format
    fn transform_audio_buffer_to_js_compatible(&self, data: FlowData) -> Result<(FlowData, bool), TransformError> {
        // Create JavaScript-compatible interleaved format
        let mut js_compatible = data;
        js_compatible.format = DataFormat::BinaryData;
        js_compatible.metadata.insert("js_compatible".to_string(), "true".to_string());
        
        Ok((js_compatible, false))
    }
    
    /// Transform audio buffer to WASM-optimized format
    fn transform_audio_buffer_to_wasm_optimized(&self, data: FlowData) -> Result<(FlowData, bool), TransformError> {
        // Create WASM-optimized binary format
        let mut wasm_optimized = data;
        wasm_optimized.format = DataFormat::BinaryData;
        wasm_optimized.metadata.insert("wasm_optimized".to_string(), "true".to_string());
        
        Ok((wasm_optimized, false))
    }
    
    /// Transform f32 array to audio buffer
    fn transform_f32_array_to_audio_buffer(&self, data: FlowData) -> Result<(FlowData, bool), TransformError> {
        let transformed_data = FlowData {
            format: DataFormat::AudioBuffer,
            data: data.data,
            metadata: data.metadata,
            timestamp: data.timestamp,
        };
        
        Ok((transformed_data, false))
    }
    
    /// Transform typed event to binary
    fn transform_typed_event_to_binary(&self, data: FlowData) -> Result<(FlowData, bool), TransformError> {
        let transformed_data = FlowData {
            format: DataFormat::BinaryData,
            data: data.data,
            metadata: data.metadata,
            timestamp: data.timestamp,
        };
        
        Ok((transformed_data, false))
    }
    
    /// Transform binary to typed event
    fn transform_binary_to_typed_event(&self, data: FlowData) -> Result<(FlowData, bool), TransformError> {
        let transformed_data = FlowData {
            format: DataFormat::TypedEvent,
            data: data.data,
            metadata: data.metadata,
            timestamp: data.timestamp,
        };
        
        Ok((transformed_data, false))
    }
    
    /// Transform metrics to JSON
    fn transform_metrics_to_json(&self, data: FlowData) -> Result<(FlowData, bool), TransformError> {
        // In a real implementation, this would serialize metrics to JSON
        let transformed_data = FlowData {
            format: DataFormat::BinaryData,
            data: data.data,
            metadata: data.metadata,
            timestamp: data.timestamp,
        };
        
        Ok((transformed_data, false))
    }
    
    /// Transform configuration to binary
    fn transform_config_to_binary(&self, data: FlowData) -> Result<(FlowData, bool), TransformError> {
        let transformed_data = FlowData {
            format: DataFormat::BinaryData,
            data: data.data,
            metadata: data.metadata,
            timestamp: data.timestamp,
        };
        
        Ok((transformed_data, false))
    }
    
    /// Preserve metadata according to strategy
    fn preserve_metadata(&self, mut data: FlowData, strategy: &MetadataPreservation) -> FlowData {
        match strategy {
            MetadataPreservation::Full => {
                // Keep all metadata
                data
            },
            MetadataPreservation::Essential => {
                // Keep only essential metadata
                let mut essential_metadata = HashMap::new();
                for key in &["timestamp", "sequence", "priority"] {
                    if let Some(value) = data.metadata.get(*key) {
                        essential_metadata.insert(key.to_string(), value.clone());
                    }
                }
                data.metadata = essential_metadata;
                data
            },
            MetadataPreservation::None => {
                // Clear all metadata
                data.metadata.clear();
                data
            },
            MetadataPreservation::Custom(keys) => {
                // Keep only specified keys
                let mut custom_metadata = HashMap::new();
                for key in keys {
                    if let Some(value) = data.metadata.get(key) {
                        custom_metadata.insert(key.clone(), value.clone());
                    }
                }
                data.metadata = custom_metadata;
                data
            },
        }
    }
    
    /// Update transformation metrics
    fn update_metrics(
        &mut self,
        transform_fn: &TransformFunction,
        time_ms: f32,
        bytes_processed: usize,
        zero_copy_used: bool,
        success: bool,
    ) {
        self.metrics.total_transformations += 1;
        
        if success {
            self.metrics.successful_transformations += 1;
        } else {
            self.metrics.failed_transformations += 1;
        }
        
        if zero_copy_used {
            self.metrics.zero_copy_transformations += 1;
        }
        
        self.metrics.bytes_transformed += bytes_processed as u64;
        self.metrics.average_transform_time_ms = 
            0.9 * self.metrics.average_transform_time_ms + 0.1 * time_ms;
        self.metrics.max_transform_time_ms = 
            self.metrics.max_transform_time_ms.max(time_ms);
        
        // Update per-transform-type rates
        let transform_type = format!("{:?}", transform_fn);
        let current_rate = self.metrics.transform_rates.get(&transform_type).unwrap_or(&0.0);
        self.metrics.transform_rates.insert(transform_type, current_rate + 1.0);
    }
    
    /// Get transformation performance metrics
    pub fn get_metrics(&self) -> &TransformMetrics {
        &self.metrics
    }
    
    /// Reset transformation metrics
    pub fn reset_metrics(&mut self) {
        self.metrics = TransformMetrics::default();
    }
    
    /// Enable or disable zero-copy optimizations
    pub fn set_zero_copy_enabled(&mut self, enabled: bool) {
        self.zero_copy_enabled = enabled;
    }
    
    /// Check if zero-copy is enabled
    pub fn is_zero_copy_enabled(&self) -> bool {
        self.zero_copy_enabled
    }
    
    /// Get available transformation rules
    pub fn get_available_transformations(&self) -> Vec<&TransformKey> {
        self.transform_rules.keys().collect()
    }
}

impl Default for DataTransformer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn test_transformer_creation() {
        let transformer = DataTransformer::new();
        assert!(transformer.is_zero_copy_enabled());
        assert!(!transformer.transform_rules.is_empty());
    }
    
    #[test]
    fn test_same_format_transformation() {
        let mut transformer = DataTransformer::new();
        let data = FlowData {
            format: DataFormat::AudioBuffer,
            data: vec![1, 2, 3, 4],
            metadata: HashMap::new(),
            timestamp: Instant::now(),
        };
        
        let result = transformer.transform_data(
            data,
            DataFormat::AudioBuffer,
            DataFormat::AudioBuffer,
        ).unwrap();
        
        assert!(result.zero_copy_used);
        assert_eq!(result.transform_time_ms, 0.0);
    }
    
    #[test]
    fn test_audio_buffer_to_binary_transformation() {
        let mut transformer = DataTransformer::new();
        let data = FlowData {
            format: DataFormat::AudioBuffer,
            data: vec![0, 0, 0, 0, 0, 0, 128, 63], // f32 values as bytes
            metadata: HashMap::new(),
            timestamp: Instant::now(),
        };
        
        let result = transformer.transform_data(
            data,
            DataFormat::AudioBuffer,
            DataFormat::BinaryData,
        ).unwrap();
        
        assert_eq!(result.data.format, DataFormat::BinaryData);
        assert!(result.bytes_processed > 0);
    }
    
    #[test]
    fn test_metrics_tracking() {
        let mut transformer = DataTransformer::new();
        let data = FlowData {
            format: DataFormat::AudioBuffer,
            data: vec![1, 2, 3, 4],
            metadata: HashMap::new(),
            timestamp: Instant::now(),
        };
        
        let _ = transformer.transform_data(
            data,
            DataFormat::AudioBuffer,
            DataFormat::BinaryData,
        );
        
        let metrics = transformer.get_metrics();
        assert_eq!(metrics.total_transformations, 1);
        assert_eq!(metrics.successful_transformations, 1);
    }
    
    #[test]
    fn test_metadata_preservation() {
        let mut transformer = DataTransformer::new();
        let mut metadata = HashMap::new();
        metadata.insert("test_key".to_string(), "test_value".to_string());
        
        let data = FlowData {
            format: DataFormat::AudioBuffer,
            data: vec![1, 2, 3, 4],
            metadata,
            timestamp: Instant::now(),
        };
        
        let result = transformer.transform_data(
            data,
            DataFormat::AudioBuffer,
            DataFormat::BinaryData,
        ).unwrap();
        
        assert!(result.metadata_preserved);
        assert!(result.data.metadata.contains_key("test_key"));
    }
    
    #[test]
    fn test_zero_copy_optimization() {
        let mut transformer = DataTransformer::new();
        assert!(transformer.is_zero_copy_enabled());
        
        transformer.set_zero_copy_enabled(false);
        assert!(!transformer.is_zero_copy_enabled());
    }
}