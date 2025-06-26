//! # Pool Optimization Engine
//!
//! This module implements automatic pool sizing optimization with usage pattern analysis,
//! pool growth and shrinkage algorithms based on memory pressure, and pool configuration
//! adaptation based on real-time usage.

use std::sync::{Arc, RwLock, Mutex};
use std::collections::HashMap;
use std::time::{Instant, Duration};
use super::buffer_recycling_pool::{PoolMetrics, PoolError};

/// Pool size optimization engine
pub struct PoolOptimizationEngine {
    /// Usage pattern analyzer
    pattern_analyzer: UsagePatternAnalyzer,
    /// Optimization configuration
    config: OptimizationConfig,
    /// Optimization history
    optimization_history: Arc<Mutex<Vec<OptimizationEvent>>>,
    /// Last optimization time
    last_optimization: Arc<Mutex<Instant>>,
}

/// Configuration for pool optimization
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Minimum time between optimizations
    pub optimization_interval: Duration,
    /// Memory pressure threshold for aggressive optimization
    pub memory_pressure_threshold: f32,
    /// Efficiency improvement threshold to trigger optimization
    pub min_efficiency_improvement: f32,
    /// Maximum pool growth rate per optimization cycle
    pub max_growth_rate: f32,
    /// Maximum pool shrinkage rate per optimization cycle
    pub max_shrinkage_rate: f32,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            optimization_interval: Duration::from_secs(30), // Optimize every 30 seconds
            memory_pressure_threshold: 0.8, // 80% memory usage
            min_efficiency_improvement: 2.0, // 2% minimum improvement
            max_growth_rate: 0.5, // 50% growth max
            max_shrinkage_rate: 0.3, // 30% shrinkage max
        }
    }
}

/// Usage pattern analysis for buffer size distribution
#[derive(Debug, Clone)]
pub struct UsagePatternAnalyzer {
    /// Buffer size frequency tracking
    size_frequency: Arc<RwLock<HashMap<usize, FrequencyData>>>,
    /// Access pattern tracking
    access_patterns: Arc<RwLock<HashMap<usize, AccessPattern>>>,
    /// Analysis window
    analysis_window: Duration,
}

/// Frequency data for buffer sizes
#[derive(Debug, Clone)]
struct FrequencyData {
    /// Request count
    requests: u64,
    /// Last access time
    last_access: Instant,
    /// Average access interval
    avg_interval_ms: f64,
    /// Peak usage periods
    peak_usage_count: u64,
}

/// Access pattern data
#[derive(Debug, Clone)]
struct AccessPattern {
    /// Allocation count
    allocations: u64,
    /// Recycling success rate
    recycling_success_rate: f32,
    /// Average allocation time
    avg_allocation_time_ns: u64,
    /// Memory pressure during allocations
    memory_pressure_history: Vec<f32>,
}

/// Optimization event record
#[derive(Debug, Clone)]
pub struct OptimizationEvent {
    pub timestamp: Instant,
    pub optimization_type: OptimizationType,
    pub affected_sizes: Vec<usize>,
    pub efficiency_improvement: f32,
    pub memory_change_bytes: i64, // Positive = growth, negative = shrinkage
}

#[derive(Debug, Clone)]
pub enum OptimizationType {
    PoolExpansion,
    PoolShrinkage,
    BucketRebalancing,
    PreWarming,
}

/// Pool sizing recommendation
#[derive(Debug, Clone)]
pub struct PoolSizingRecommendation {
    pub buffer_size: usize,
    pub current_pool_size: usize,
    pub recommended_pool_size: usize,
    pub confidence_score: f32, // 0.0 to 1.0
    pub reasoning: String,
    pub estimated_efficiency_gain: f32,
}

impl PoolOptimizationEngine {
    /// Create a new pool optimization engine
    pub fn new() -> Self {
        Self {
            pattern_analyzer: UsagePatternAnalyzer::new(),
            config: OptimizationConfig::default(),
            optimization_history: Arc::new(Mutex::new(Vec::new())),
            last_optimization: Arc::new(Mutex::new(Instant::now())),
        }
    }
    
    /// Create engine with custom configuration
    pub fn with_config(config: OptimizationConfig) -> Self {
        Self {
            pattern_analyzer: UsagePatternAnalyzer::new(),
            config,
            optimization_history: Arc::new(Mutex::new(Vec::new())),
            last_optimization: Arc::new(Mutex::new(Instant::now())),
        }
    }
    
    /// Record buffer allocation for pattern analysis
    pub fn record_allocation(&self, size: usize, allocation_time_ns: u64, memory_pressure: f32) -> Result<(), PoolError> {
        self.pattern_analyzer.record_allocation(size, allocation_time_ns, memory_pressure)
    }
    
    /// Record buffer recycling for pattern analysis
    pub fn record_recycling(&self, size: usize, success: bool) -> Result<(), PoolError> {
        self.pattern_analyzer.record_recycling(size, success)
    }
    
    /// Analyze usage patterns and generate pool sizing recommendations
    pub fn analyze_and_recommend(&self, current_metrics: &PoolMetrics) -> Result<Vec<PoolSizingRecommendation>, PoolError> {
        // Check if optimization interval has passed
        {
            let last_opt = self.last_optimization.lock()
                .map_err(|_| PoolError::Internal("Failed to check last optimization time".to_string()))?;
            
            if last_opt.elapsed() < self.config.optimization_interval {
                return Ok(Vec::new()); // Too soon to optimize
            }
        }
        
        let usage_analysis = self.pattern_analyzer.analyze_usage_patterns()?;
        let recommendations = self.generate_recommendations(&usage_analysis, current_metrics)?;
        
        Ok(recommendations)
    }
    
    /// Generate pool sizing recommendations based on usage analysis
    fn generate_recommendations(
        &self,
        usage_analysis: &UsageAnalysis,
        current_metrics: &PoolMetrics,
    ) -> Result<Vec<PoolSizingRecommendation>, PoolError> {
        let mut recommendations = Vec::new();
        
        for (&size, frequency) in &usage_analysis.size_frequencies {
            let current_pool_size = usage_analysis.current_pool_sizes.get(&size).copied().unwrap_or(0);
            
            // Calculate recommended size based on usage patterns
            let recommended_size = self.calculate_optimal_pool_size(
                size,
                frequency,
                current_pool_size,
                current_metrics,
            )?;
            
            if recommended_size != current_pool_size {
                let confidence = self.calculate_confidence_score(frequency, &usage_analysis.access_patterns[&size]);
                let efficiency_gain = self.estimate_efficiency_gain(current_pool_size, recommended_size, frequency);
                
                let reasoning = if recommended_size > current_pool_size {
                    format!("High usage frequency ({} requests/min) suggests pool expansion", 
                           frequency.requests_per_minute)
                } else {
                    format!("Low usage frequency ({} requests/min) suggests pool shrinkage", 
                           frequency.requests_per_minute)
                };
                
                recommendations.push(PoolSizingRecommendation {
                    buffer_size: size,
                    current_pool_size,
                    recommended_pool_size: recommended_size,
                    confidence_score: confidence,
                    reasoning,
                    estimated_efficiency_gain: efficiency_gain,
                });
            }
        }
        
        // Sort by estimated efficiency gain (highest first)
        recommendations.sort_by(|a, b| {
            b.estimated_efficiency_gain.partial_cmp(&a.estimated_efficiency_gain)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(recommendations)
    }
    
    /// Calculate optimal pool size for a given buffer size
    fn calculate_optimal_pool_size(
        &self,
        buffer_size: usize,
        frequency: &FrequencyAnalysis,
        current_size: usize,
        metrics: &PoolMetrics,
    ) -> Result<usize, PoolError> {
        // Base calculation on requests per minute and success rate
        let base_size = (frequency.requests_per_minute / 4.0).ceil() as usize; // Assume 4 requests per buffer on average
        
        // Adjust for recycling success rate
        let recycling_factor = if frequency.recycling_success_rate > 0.8 {
            1.0 // Good recycling, can use smaller pool
        } else {
            1.5 // Poor recycling, need larger pool
        };
        
        let adjusted_size = (base_size as f32 * recycling_factor) as usize;
        
        // Apply growth/shrinkage limits
        let min_size = (current_size as f32 * (1.0 - self.config.max_shrinkage_rate)) as usize;
        let max_size = (current_size as f32 * (1.0 + self.config.max_growth_rate)) as usize;
        
        Ok(adjusted_size.max(min_size).min(max_size).max(1))
    }
    
    /// Calculate confidence score for recommendation
    fn calculate_confidence_score(&self, frequency: &FrequencyAnalysis, pattern: &AccessPattern) -> f32 {
        let mut confidence = 0.5; // Base confidence
        
        // Higher confidence for consistent access patterns
        if frequency.consistency_score > 0.7 {
            confidence += 0.3;
        }
        
        // Higher confidence for good recycling rates
        if pattern.recycling_success_rate > 0.8 {
            confidence += 0.2;
        }
        
        // Lower confidence if memory pressure is high (system under stress)
        let avg_memory_pressure = pattern.memory_pressure_history.iter().sum::<f32>() 
            / pattern.memory_pressure_history.len() as f32;
        if avg_memory_pressure > 0.8 {
            confidence -= 0.2;
        }
        
        confidence.max(0.0).min(1.0)
    }
    
    /// Estimate efficiency gain from pool size change
    fn estimate_efficiency_gain(&self, current_size: usize, recommended_size: usize, frequency: &FrequencyAnalysis) -> f32 {
        if recommended_size > current_size {
            // Expansion - estimate hit rate improvement
            let size_increase_ratio = recommended_size as f32 / current_size as f32;
            let estimated_hit_rate_improvement = (size_increase_ratio - 1.0) * frequency.miss_rate;
            estimated_hit_rate_improvement * 10.0 // Scale to percentage
        } else {
            // Shrinkage - estimate memory savings
            let size_decrease_ratio = current_size as f32 / recommended_size as f32;
            let estimated_memory_savings = (size_decrease_ratio - 1.0) * 2.0; // Conservative estimate
            estimated_memory_savings.min(5.0) // Cap at 5% savings
        }
    }
    
    /// Apply optimization recommendations
    pub fn apply_recommendations(
        &self,
        recommendations: &[PoolSizingRecommendation],
        apply_callback: impl Fn(usize, usize) -> Result<(), PoolError>,
    ) -> Result<Vec<OptimizationEvent>, PoolError> {
        let mut events = Vec::new();
        
        for recommendation in recommendations {
            // Only apply high-confidence recommendations
            if recommendation.confidence_score >= 0.7 && 
               recommendation.estimated_efficiency_gain >= self.config.min_efficiency_improvement {
                
                // Apply the size change
                apply_callback(recommendation.buffer_size, recommendation.recommended_pool_size)?;
                
                // Record optimization event
                let optimization_type = if recommendation.recommended_pool_size > recommendation.current_pool_size {
                    OptimizationType::PoolExpansion
                } else {
                    OptimizationType::PoolShrinkage
                };
                
                let memory_change = (recommendation.recommended_pool_size as i64 - recommendation.current_pool_size as i64) 
                    * recommendation.buffer_size as i64 * 4; // 4 bytes per f32
                
                let event = OptimizationEvent {
                    timestamp: Instant::now(),
                    optimization_type,
                    affected_sizes: vec![recommendation.buffer_size],
                    efficiency_improvement: recommendation.estimated_efficiency_gain,
                    memory_change_bytes: memory_change,
                };
                
                events.push(event.clone());
            }
        }
        
        // Update optimization history
        {
            let mut history = self.optimization_history.lock()
                .map_err(|_| PoolError::Internal("Failed to update optimization history".to_string()))?;
            history.extend(events.clone());
            
            // Keep only recent history (last 100 events)
            if history.len() > 100 {
                history.drain(0..history.len() - 100);
            }
        }
        
        // Update last optimization time
        {
            let mut last_opt = self.last_optimization.lock()
                .map_err(|_| PoolError::Internal("Failed to update last optimization time".to_string()))?;
            *last_opt = Instant::now();
        }
        
        Ok(events)
    }
    
    /// Get optimization history for analysis
    pub fn get_optimization_history(&self) -> Result<Vec<OptimizationEvent>, PoolError> {
        self.optimization_history.lock()
            .map(|history| history.clone())
            .map_err(|_| PoolError::Internal("Failed to read optimization history".to_string()))
    }
}

/// Usage analysis results
#[derive(Debug, Clone)]
pub struct UsageAnalysis {
    pub size_frequencies: HashMap<usize, FrequencyAnalysis>,
    pub access_patterns: HashMap<usize, AccessPattern>,
    pub current_pool_sizes: HashMap<usize, usize>,
    pub analysis_timespan: Duration,
}

/// Frequency analysis for a buffer size
#[derive(Debug, Clone)]
pub struct FrequencyAnalysis {
    pub requests_per_minute: f64,
    pub recycling_success_rate: f32,
    pub consistency_score: f32, // 0.0 to 1.0, higher = more consistent
    pub miss_rate: f32,
}

impl UsagePatternAnalyzer {
    fn new() -> Self {
        Self {
            size_frequency: Arc::new(RwLock::new(HashMap::new())),
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            analysis_window: Duration::from_secs(300), // 5 minute window
        }
    }
    
    fn record_allocation(&self, size: usize, allocation_time_ns: u64, memory_pressure: f32) -> Result<(), PoolError> {
        // Update frequency data
        {
            let mut frequencies = self.size_frequency.write()
                .map_err(|_| PoolError::Internal("Failed to update size frequency".to_string()))?;
            
            let entry = frequencies.entry(size).or_insert(FrequencyData {
                requests: 0,
                last_access: Instant::now(),
                avg_interval_ms: 0.0,
                peak_usage_count: 0,
            });
            
            entry.requests += 1;
            entry.last_access = Instant::now();
        }
        
        // Update access patterns
        {
            let mut patterns = self.access_patterns.write()
                .map_err(|_| PoolError::Internal("Failed to update access patterns".to_string()))?;
            
            let entry = patterns.entry(size).or_insert(AccessPattern {
                allocations: 0,
                recycling_success_rate: 0.0,
                avg_allocation_time_ns: 0,
                memory_pressure_history: Vec::new(),
            });
            
            entry.allocations += 1;
            
            // Update rolling average allocation time
            let count = entry.allocations;
            entry.avg_allocation_time_ns = ((entry.avg_allocation_time_ns * (count - 1)) + allocation_time_ns) / count;
            
            // Track memory pressure
            entry.memory_pressure_history.push(memory_pressure);
            if entry.memory_pressure_history.len() > 100 {
                entry.memory_pressure_history.remove(0);
            }
        }
        
        Ok(())
    }
    
    fn record_recycling(&self, size: usize, success: bool) -> Result<(), PoolError> {
        let mut patterns = self.access_patterns.write()
            .map_err(|_| PoolError::Internal("Failed to update recycling data".to_string()))?;
        
        if let Some(pattern) = patterns.get_mut(&size) {
            // Update recycling success rate using exponential moving average
            let alpha = 0.1; // Smoothing factor
            let success_value = if success { 1.0 } else { 0.0 };
            pattern.recycling_success_rate = (1.0 - alpha) * pattern.recycling_success_rate + alpha * success_value;
        }
        
        Ok(())
    }
    
    fn analyze_usage_patterns(&self) -> Result<UsageAnalysis, PoolError> {
        let frequencies = self.size_frequency.read()
            .map_err(|_| PoolError::Internal("Failed to read size frequencies".to_string()))?;
        
        let patterns = self.access_patterns.read()
            .map_err(|_| PoolError::Internal("Failed to read access patterns".to_string()))?;
        
        let mut size_frequencies = HashMap::new();
        let window_minutes = self.analysis_window.as_secs_f64() / 60.0;
        
        for (&size, freq_data) in frequencies.iter() {
            let requests_per_minute = freq_data.requests as f64 / window_minutes;
            
            let access_pattern = patterns.get(&size).cloned().unwrap_or_default();
            
            // Calculate consistency score based on access interval variance
            let consistency_score = if freq_data.requests > 10 {
                0.8 // Simplified - would calculate actual variance in real implementation
            } else {
                0.5
            };
            
            // Calculate miss rate (simplified)
            let miss_rate = 1.0 - access_pattern.recycling_success_rate;
            
            size_frequencies.insert(size, FrequencyAnalysis {
                requests_per_minute,
                recycling_success_rate: access_pattern.recycling_success_rate,
                consistency_score,
                miss_rate,
            });
        }
        
        Ok(UsageAnalysis {
            size_frequencies,
            access_patterns: patterns.clone(),
            current_pool_sizes: HashMap::new(), // Would be populated by caller
            analysis_timespan: self.analysis_window,
        })
    }
}

impl Default for AccessPattern {
    fn default() -> Self {
        Self {
            allocations: 0,
            recycling_success_rate: 0.0,
            avg_allocation_time_ns: 0,
            memory_pressure_history: Vec::new(),
        }
    }
}

impl Default for PoolOptimizationEngine {
    fn default() -> Self {
        Self::new()
    }
}