use super::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Error recovery manager for platform-specific failures
pub struct ErrorRecoveryManager {
    /// Error categorization mapping
    error_categories: Arc<Mutex<HashMap<String, ErrorCategory>>>,
    /// Recovery strategies for different error types
    recovery_strategies: Arc<Mutex<HashMap<ErrorCategory, RecoveryStrategy>>>,
    /// Error occurrence tracking
    error_history: Arc<Mutex<Vec<ErrorOccurrence>>>,
    /// Graceful degradation settings
    degradation_settings: Arc<Mutex<DegradationSettings>>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum ErrorCategory {
    BrowserCompatibility,
    DeviceAccess,
    PermissionDenied,
    HardwareAcceleration,
    MemoryConstraints,
    NetworkConnectivity,
    UnknownFeature,
    RecoveryFailure,
}

#[derive(Debug, Clone)]
struct RecoveryStrategy {
    max_retry_attempts: u32,
    retry_delay: Duration,
    fallback_action: FallbackAction,
    user_guidance: String,
}

#[derive(Debug, Clone)]
enum FallbackAction {
    DisableFeature(String),
    SwitchToCompatibilityMode,
    UseAlternativeDevice,
    ShowUserGuidance,
    GracefulDegrade,
    RestartModule,
}

#[derive(Debug, Clone)]
struct ErrorOccurrence {
    error: PlatformError,
    category: ErrorCategory,
    timestamp: Instant,
    recovery_attempted: bool,
    recovery_successful: bool,
}

#[derive(Debug, Clone)]
struct DegradationSettings {
    allow_feature_disabling: bool,
    allow_performance_reduction: bool,
    minimum_functionality_level: FunctionalityLevel,
    user_notification_enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum FunctionalityLevel {
    Full,
    Reduced,
    Minimal,
    Emergency,
}

impl ErrorRecoveryManager {
    pub fn new() -> Self {
        let mut recovery_strategies = HashMap::new();
        
        // Initialize recovery strategies for each error category
        recovery_strategies.insert(
            ErrorCategory::BrowserCompatibility,
            RecoveryStrategy {
                max_retry_attempts: 0,
                retry_delay: Duration::from_millis(0),
                fallback_action: FallbackAction::SwitchToCompatibilityMode,
                user_guidance: "Your browser version is not fully supported. Please consider upgrading to a newer version for optimal performance.".to_string(),
            }
        );
        
        recovery_strategies.insert(
            ErrorCategory::DeviceAccess,
            RecoveryStrategy {
                max_retry_attempts: 3,
                retry_delay: Duration::from_millis(1000),
                fallback_action: FallbackAction::UseAlternativeDevice,
                user_guidance: "Unable to access the selected audio device. Trying alternative devices...".to_string(),
            }
        );
        
        recovery_strategies.insert(
            ErrorCategory::PermissionDenied,
            RecoveryStrategy {
                max_retry_attempts: 1,
                retry_delay: Duration::from_millis(2000),
                fallback_action: FallbackAction::ShowUserGuidance,
                user_guidance: "Microphone permission is required. Please click 'Allow' when prompted or enable microphone access in your browser settings.".to_string(),
            }
        );
        
        recovery_strategies.insert(
            ErrorCategory::HardwareAcceleration,
            RecoveryStrategy {
                max_retry_attempts: 0,
                retry_delay: Duration::from_millis(0),
                fallback_action: FallbackAction::DisableFeature("hardware_acceleration".to_string()),
                user_guidance: "Hardware acceleration is not available. Using software processing instead.".to_string(),
            }
        );
        
        recovery_strategies.insert(
            ErrorCategory::MemoryConstraints,
            RecoveryStrategy {
                max_retry_attempts: 2,
                retry_delay: Duration::from_millis(500),
                fallback_action: FallbackAction::GracefulDegrade,
                user_guidance: "Memory usage is high. Reducing quality settings to maintain stability.".to_string(),
            }
        );
        
        Self {
            error_categories: Arc::new(Mutex::new(HashMap::new())),
            recovery_strategies: Arc::new(Mutex::new(recovery_strategies)),
            error_history: Arc::new(Mutex::new(Vec::new())),
            degradation_settings: Arc::new(Mutex::new(DegradationSettings {
                allow_feature_disabling: true,
                allow_performance_reduction: true,
                minimum_functionality_level: FunctionalityLevel::Minimal,
                user_notification_enabled: true,
            })),
        }
    }
    
    /// Categorize platform error for appropriate recovery
    pub fn categorize_error(&self, error: &PlatformError) -> ErrorCategory {
        match error {
            PlatformError::BrowserDetectionFailed(_) => ErrorCategory::BrowserCompatibility,
            PlatformError::UnsupportedFeature(_) => ErrorCategory::UnknownFeature,
            PlatformError::DeviceCapabilityError(msg) => {
                if msg.contains("permission") {
                    ErrorCategory::PermissionDenied
                } else if msg.contains("device") {
                    ErrorCategory::DeviceAccess
                } else {
                    ErrorCategory::DeviceAccess
                }
            }
            PlatformError::WasmBridgeError(_) => ErrorCategory::BrowserCompatibility,
            PlatformError::OptimizationError(msg) => {
                if msg.contains("memory") {
                    ErrorCategory::MemoryConstraints
                } else {
                    ErrorCategory::BrowserCompatibility
                }
            }
            PlatformError::PermissionDenied(_) => ErrorCategory::PermissionDenied,
        }
    }
    
    /// Attempt error recovery with appropriate strategy
    pub async fn handle_error(&self, error: PlatformError) -> Result<RecoveryResult, PlatformError> {
        let category = self.categorize_error(&error);
        let strategy = {
            let strategies = self.recovery_strategies.lock().unwrap();
            strategies.get(&category).cloned()
        };
        
        let recovery_strategy = strategy.ok_or_else(|| {
            PlatformError::OptimizationError("No recovery strategy found for error category".to_string())
        })?;
        
        // Record error occurrence
        self.record_error_occurrence(error.clone(), category.clone());
        
        // Attempt recovery with retries
        for attempt in 0..=recovery_strategy.max_retry_attempts {
            if attempt > 0 {
                // Wait before retry
                #[cfg(target_arch = "wasm32")]
                {
                    // Use setTimeout for delay in WASM
                    let delay_ms = recovery_strategy.retry_delay.as_millis() as i32;
                    let promise = js_sys::Promise::new(&mut |resolve, _| {
                        web_sys::window()
                            .unwrap()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, delay_ms)
                            .unwrap();
                    });
                    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                }
                
                #[cfg(not(target_arch = "wasm32"))]
                {
                    std::thread::sleep(recovery_strategy.retry_delay);
                }
            }
            
            // Apply recovery strategy
            match self.apply_recovery_strategy(&recovery_strategy, &error).await {
                Ok(result) => {
                    // Mark recovery as successful
                    self.mark_recovery_successful(&error);
                    return Ok(result);
                }
                Err(recovery_error) => {
                    if attempt == recovery_strategy.max_retry_attempts {
                        // All retries exhausted
                        self.mark_recovery_failed(&error);
                        return Err(recovery_error);
                    }
                }
            }
        }
        
        // Should not reach here, but handle gracefully
        Err(PlatformError::OptimizationError("Recovery attempts exhausted".to_string()))
    }
    
    /// Apply specific recovery strategy
    async fn apply_recovery_strategy(&self, strategy: &RecoveryStrategy, error: &PlatformError) -> Result<RecoveryResult, PlatformError> {
        match &strategy.fallback_action {
            FallbackAction::DisableFeature(feature) => {
                Ok(RecoveryResult {
                    success: true,
                    action_taken: format!("Disabled feature: {}", feature),
                    user_guidance: Some(strategy.user_guidance.clone()),
                    functionality_impact: FunctionalityImpact::FeatureDisabled(feature.clone()),
                })
            }
            FallbackAction::SwitchToCompatibilityMode => {
                Ok(RecoveryResult {
                    success: true,
                    action_taken: "Switched to compatibility mode".to_string(),
                    user_guidance: Some(strategy.user_guidance.clone()),
                    functionality_impact: FunctionalityImpact::PerformanceReduced,
                })
            }
            FallbackAction::UseAlternativeDevice => {
                // Try to find alternative device
                Ok(RecoveryResult {
                    success: true,
                    action_taken: "Attempting to use alternative audio device".to_string(),
                    user_guidance: Some(strategy.user_guidance.clone()),
                    functionality_impact: FunctionalityImpact::DeviceChanged,
                })
            }
            FallbackAction::ShowUserGuidance => {
                Ok(RecoveryResult {
                    success: false, // Requires user action
                    action_taken: "Displayed user guidance".to_string(),
                    user_guidance: Some(strategy.user_guidance.clone()),
                    functionality_impact: FunctionalityImpact::RequiresUserAction,
                })
            }
            FallbackAction::GracefulDegrade => {
                Ok(RecoveryResult {
                    success: true,
                    action_taken: "Applied graceful degradation".to_string(),
                    user_guidance: Some(strategy.user_guidance.clone()),
                    functionality_impact: FunctionalityImpact::QualityReduced,
                })
            }
            FallbackAction::RestartModule => {
                Ok(RecoveryResult {
                    success: true,
                    action_taken: "Restarting platform abstraction module".to_string(),
                    user_guidance: Some("Restarting to recover from error...".to_string()),
                    functionality_impact: FunctionalityImpact::TemporaryDisruption,
                })
            }
        }
    }
    
    /// Record error occurrence for analysis
    fn record_error_occurrence(&self, error: PlatformError, category: ErrorCategory) {
        let occurrence = ErrorOccurrence {
            error,
            category,
            timestamp: Instant::now(),
            recovery_attempted: true,
            recovery_successful: false,
        };
        
        let mut history = self.error_history.lock().unwrap();
        history.push(occurrence);
        
        // Keep only recent errors (last 100)
        if history.len() > 100 {
            history.remove(0);
        }
    }
    
    /// Mark recovery as successful
    fn mark_recovery_successful(&self, error: &PlatformError) {
        let mut history = self.error_history.lock().unwrap();
        if let Some(last_occurrence) = history.last_mut() {
            if std::mem::discriminant(&last_occurrence.error) == std::mem::discriminant(error) {
                last_occurrence.recovery_successful = true;
            }
        }
    }
    
    /// Mark recovery as failed
    fn mark_recovery_failed(&self, error: &PlatformError) {
        let mut history = self.error_history.lock().unwrap();
        if let Some(last_occurrence) = history.last_mut() {
            if std::mem::discriminant(&last_occurrence.error) == std::mem::discriminant(error) {
                last_occurrence.recovery_successful = false;
            }
        }
    }
    
    /// Get recovery statistics
    pub fn get_recovery_statistics(&self) -> RecoveryStatistics {
        let history = self.error_history.lock().unwrap();
        let total_errors = history.len();
        let successful_recoveries = history.iter().filter(|e| e.recovery_successful).count();
        
        let mut category_stats = HashMap::new();
        for occurrence in history.iter() {
            let entry = category_stats.entry(occurrence.category.clone()).or_insert((0, 0));
            entry.0 += 1; // Total count
            if occurrence.recovery_successful {
                entry.1 += 1; // Successful count
            }
        }
        
        RecoveryStatistics {
            total_errors,
            successful_recoveries,
            recovery_rate: if total_errors > 0 {
                successful_recoveries as f32 / total_errors as f32
            } else {
                0.0
            },
            category_statistics: category_stats,
        }
    }
}

/// Result of error recovery attempt
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub success: bool,
    pub action_taken: String,
    pub user_guidance: Option<String>,
    pub functionality_impact: FunctionalityImpact,
}

/// Impact on functionality after recovery
#[derive(Debug, Clone)]
pub enum FunctionalityImpact {
    None,
    FeatureDisabled(String),
    PerformanceReduced,
    QualityReduced,
    DeviceChanged,
    RequiresUserAction,
    TemporaryDisruption,
}

/// Recovery statistics for monitoring
#[derive(Debug)]
pub struct RecoveryStatistics {
    pub total_errors: usize,
    pub successful_recoveries: usize,
    pub recovery_rate: f32,
    pub category_statistics: HashMap<ErrorCategory, (usize, usize)>, // (total, successful)
}

impl Default for ErrorRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_categorization() {
        let manager = ErrorRecoveryManager::new();
        
        let browser_error = PlatformError::BrowserDetectionFailed("Test".to_string());
        assert_eq!(manager.categorize_error(&browser_error), ErrorCategory::BrowserCompatibility);
        
        let permission_error = PlatformError::PermissionDenied("Test".to_string());
        assert_eq!(manager.categorize_error(&permission_error), ErrorCategory::PermissionDenied);
        
        let device_error = PlatformError::DeviceCapabilityError("device access failed".to_string());
        assert_eq!(manager.categorize_error(&device_error), ErrorCategory::DeviceAccess);
    }
    
    #[test]
    fn test_recovery_statistics() {
        let manager = ErrorRecoveryManager::new();
        
        // Test initial state
        let stats = manager.get_recovery_statistics();
        assert_eq!(stats.total_errors, 0);
        assert_eq!(stats.recovery_rate, 0.0);
        
        // Add some test errors
        manager.record_error_occurrence(
            PlatformError::BrowserDetectionFailed("Test".to_string()),
            ErrorCategory::BrowserCompatibility
        );
        
        let stats = manager.get_recovery_statistics();
        assert_eq!(stats.total_errors, 1);
    }
} 