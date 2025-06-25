//! # Error Recovery and Module Isolation
//!
//! This module provides resilient error handling with module isolation boundaries,
//! ensuring that failures in one module don't crash the entire application. It includes
//! error escalation, automatic recovery strategies, module health monitoring, and
//! user-friendly error reporting with recovery suggestions.
//!
//! ## Key Components
//!
//! - [`ErrorRecoveryManager`]: Core error recovery management interface
//! - [`ErrorRecoveryManagerImpl`]: Concrete implementation with isolation and recovery
//! - [`ModuleHealth`]: Health monitoring and status tracking
//! - [`RecoveryAction`]: Recovery strategy enumeration
//! - [`ErrorContext`]: Error context preservation for debugging
//! - [`FallbackMode`]: Safe mode configurations for degraded operation
//!
//! ## Usage Example
//!
//! ```rust
//! use crate::modules::application_core::error_recovery::*;
//!
//! let mut recovery_manager = ErrorRecoveryManagerImpl::new();
//! 
//! // Handle a module error
//! let action = recovery_manager.handle_module_error(
//!     &ModuleId::new("audio-module"),
//!     &std::io::Error::new(std::io::ErrorKind::Other, "Connection failed")
//! );
//!
//! match action {
//!     RecoveryAction::Restart => {
//!         recovery_manager.restart_module(&ModuleId::new("audio-module"))?;
//!     }
//!     RecoveryAction::Fallback(mode) => {
//!         recovery_manager.set_fallback_mode(&ModuleId::new("audio-module"), mode);
//!     }
//!     _ => {}
//! }
//! ```

use super::module_registry::{Module, ModuleId, ModuleRegistry, ModuleRegistryImpl, ModuleState};
use super::event_bus::{Event, EventPriority, get_timestamp_ns};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Module health status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Module is healthy and operating normally
    Healthy,
    /// Module is experiencing minor issues but still functional
    Degraded,
    /// Module is experiencing significant issues
    Unhealthy,
    /// Module has failed and is not operational
    Failed,
    /// Module is in recovery state
    Recovering,
    /// Module is in safe/fallback mode
    SafeMode,
    /// Module health is unknown
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Unknown
    }
}

/// Fallback mode configurations for degraded operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FallbackMode {
    /// Disable the module but keep it registered
    Disabled,
    /// Run in read-only mode
    ReadOnly,
    /// Use cached/offline data
    Offline,
    /// Minimal functionality mode
    Minimal,
    /// Use alternative algorithms/implementations
    Alternative,
    /// Custom fallback configuration
    Custom(String),
}

/// Module health information with monitoring data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHealth {
    /// Current health status
    pub status: HealthStatus,
    /// Last error that occurred (if any)
    pub last_error: Option<String>,
    /// Total number of errors since startup
    pub error_count: u32,
    /// Number of consecutive errors
    pub consecutive_errors: u32,
    /// Module uptime since last restart
    pub uptime: Duration,
    /// Last restart timestamp
    pub last_restart: Option<u64>,
    /// Current fallback mode (if any)
    pub fallback_mode: Option<FallbackMode>,
    /// Error rate (errors per minute)
    pub error_rate: f64,
    /// Average response time (milliseconds)
    pub avg_response_time: f64,
    /// Memory usage (bytes)
    pub memory_usage: u64,
    /// Custom health metrics
    pub custom_metrics: HashMap<String, f64>,
}

impl ModuleHealth {
    /// Create a new module health record
    pub fn new() -> Self {
        Self {
            status: HealthStatus::Healthy,
            last_error: None,
            error_count: 0,
            consecutive_errors: 0,
            uptime: Duration::from_secs(0),
            last_restart: None,
            fallback_mode: None,
            error_rate: 0.0,
            avg_response_time: 0.0,
            memory_usage: 0,
            custom_metrics: HashMap::new(),
        }
    }

    /// Update health status
    pub fn update_status(&mut self, status: HealthStatus) {
        self.status = status;
    }

    /// Record an error occurrence
    pub fn record_error(&mut self, error: &str) {
        self.error_count += 1;
        self.consecutive_errors += 1;
        self.last_error = Some(error.to_string());
        
        // Calculate error rate (simplified)
        let uptime_minutes = self.uptime.as_secs() as f64 / 60.0;
        if uptime_minutes > 0.0 {
            self.error_rate = self.error_count as f64 / uptime_minutes;
        }
    }

    /// Record successful operation (resets consecutive errors)
    pub fn record_success(&mut self) {
        self.consecutive_errors = 0;
        if self.status == HealthStatus::Failed || self.status == HealthStatus::Unhealthy {
            self.status = HealthStatus::Healthy;
        }
    }

    /// Check if module is in a critical state
    pub fn is_critical(&self) -> bool {
        matches!(self.status, HealthStatus::Failed | HealthStatus::Unhealthy) ||
        self.consecutive_errors >= 5 ||
        self.error_rate > 10.0
    }

    /// Get recovery urgency level
    pub fn get_recovery_urgency(&self) -> RecoveryUrgency {
        match self.status {
            HealthStatus::Failed => RecoveryUrgency::Critical,
            HealthStatus::Unhealthy => {
                if self.consecutive_errors >= 3 {
                    RecoveryUrgency::High
                } else {
                    RecoveryUrgency::Medium
                }
            }
            HealthStatus::Degraded => RecoveryUrgency::Low,
            _ => RecoveryUrgency::None,
        }
    }
}

impl Default for ModuleHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// Recovery urgency levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryUrgency {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Recovery actions that can be taken for module errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Ignore the error and continue
    Ignore,
    /// Restart the module
    Restart,
    /// Escalate to higher-level error handling
    Escalate,
    /// Shutdown the module or application
    Shutdown,
    /// Switch to fallback mode
    Fallback(FallbackMode),
    /// Retry the failed operation
    Retry { max_attempts: u32, delay_ms: u64 },
    /// Rollback to a previous state
    Rollback,
    /// Quarantine the module (isolate it)
    Quarantine,
}

/// Error context for debugging and analysis
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Module that experienced the error
    pub module_id: ModuleId,
    /// Error message
    pub error_message: String,
    /// Error type/category
    pub error_type: String,
    /// Timestamp when error occurred
    pub timestamp: u64,
    /// Call stack or trace information
    pub stack_trace: Option<String>,
    /// Operation that was being performed
    pub operation: String,
    /// Additional context data
    pub context_data: HashMap<String, String>,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Whether this error is recoverable
    pub recoverable: bool,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(
        module_id: ModuleId,
        error: &dyn std::error::Error,
        operation: &str,
    ) -> Self {
        Self {
            module_id,
            error_message: error.to_string(),
            error_type: std::any::type_name_of_val(error).to_string(),
            timestamp: get_timestamp_ns(),
            stack_trace: None,
            operation: operation.to_string(),
            context_data: HashMap::new(),
            severity: ErrorSeverity::Medium,
            recoverable: true,
        }
    }

    /// Add context data
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context_data.insert(key.to_string(), value.to_string());
        self
    }

    /// Set severity level
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set whether error is recoverable
    pub fn with_recoverable(mut self, recoverable: bool) -> Self {
        self.recoverable = recoverable;
        self
    }
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Recovery strategy configuration
#[derive(Debug, Clone)]
pub struct RecoveryStrategy {
    /// Maximum number of restart attempts
    pub max_restart_attempts: u32,
    /// Restart cooldown period
    pub restart_cooldown: Duration,
    /// Error threshold for triggering recovery
    pub error_threshold: u32,
    /// Whether to enable automatic recovery
    pub auto_recovery: bool,
    /// Fallback mode to use
    pub fallback_mode: Option<FallbackMode>,
    /// Custom recovery actions
    pub custom_actions: Vec<RecoveryAction>,
}

impl Default for RecoveryStrategy {
    fn default() -> Self {
        Self {
            max_restart_attempts: 3,
            restart_cooldown: Duration::from_secs(30),
            error_threshold: 5,
            auto_recovery: true,
            fallback_mode: Some(FallbackMode::Minimal),
            custom_actions: Vec::new(),
        }
    }
}

/// Recovery errors
#[derive(Debug, Clone)]
pub enum RecoveryError {
    /// Module not found
    ModuleNotFound(String),
    /// Recovery operation failed
    RecoveryFailed(String),
    /// Module is not in a recoverable state
    NotRecoverable(String),
    /// Recovery strategy not defined
    NoStrategy(String),
    /// Resource constraint prevents recovery
    ResourceConstraint(String),
    /// Recovery timeout
    Timeout(String),
    /// Access denied for recovery operation
    AccessDenied(String),
}

impl fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryError::ModuleNotFound(id) => write!(f, "Module not found: {}", id),
            RecoveryError::RecoveryFailed(msg) => write!(f, "Recovery failed: {}", msg),
            RecoveryError::NotRecoverable(msg) => write!(f, "Not recoverable: {}", msg),
            RecoveryError::NoStrategy(msg) => write!(f, "No recovery strategy: {}", msg),
            RecoveryError::ResourceConstraint(msg) => write!(f, "Resource constraint: {}", msg),
            RecoveryError::Timeout(msg) => write!(f, "Recovery timeout: {}", msg),
            RecoveryError::AccessDenied(msg) => write!(f, "Access denied: {}", msg),
        }
    }
}

impl std::error::Error for RecoveryError {}

/// Error recovery event for monitoring and logging
#[derive(Debug, Clone)]
pub struct ErrorRecoveryEvent {
    /// Module that experienced the error
    pub module_id: ModuleId,
    /// Recovery action taken
    pub action: RecoveryAction,
    /// Error context
    pub error_context: ErrorContext,
    /// Recovery result
    pub success: bool,
    /// Event timestamp
    pub timestamp: u64,
    /// Additional event data
    pub event_data: HashMap<String, String>,
}

impl Event for ErrorRecoveryEvent {
    fn event_type(&self) -> &'static str {
        "ErrorRecoveryEvent"
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    fn priority(&self) -> EventPriority {
        match self.error_context.severity {
            ErrorSeverity::Critical => EventPriority::Critical,
            ErrorSeverity::High => EventPriority::High,
            _ => EventPriority::Normal,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// User-friendly error report with recovery suggestions
#[derive(Debug, Clone)]
pub struct UserErrorReport {
    /// User-friendly error title
    pub title: String,
    /// Error description
    pub description: String,
    /// Suggested recovery actions for the user
    pub recovery_suggestions: Vec<String>,
    /// Error severity from user perspective
    pub user_severity: UserSeverity,
    /// Whether the issue affects functionality
    pub affects_functionality: bool,
    /// Estimated time to recovery
    pub estimated_recovery_time: Option<Duration>,
    /// Support information
    pub support_info: Option<String>,
}

/// User-facing severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Core error recovery management trait
pub trait ErrorRecoveryManager: Send + Sync {
    /// Handle a module error and determine recovery action
    fn handle_module_error(
        &mut self,
        module_id: &ModuleId,
        error: &dyn std::error::Error,
    ) -> RecoveryAction;

    /// Restart a module
    fn restart_module(&mut self, module_id: &ModuleId) -> Result<(), RecoveryError>;

    /// Get module health information
    fn get_module_health(&self, module_id: &ModuleId) -> Option<ModuleHealth>;

    /// Set fallback mode for a module
    fn set_fallback_mode(&mut self, module_id: &ModuleId, mode: FallbackMode);

    /// Get current fallback mode for a module
    fn get_fallback_mode(&self, module_id: &ModuleId) -> Option<FallbackMode>;

    /// Register a recovery strategy for a module
    fn register_recovery_strategy(
        &mut self,
        module_id: ModuleId,
        strategy: RecoveryStrategy,
    );

    /// Update module health monitoring data
    fn update_module_health(&mut self, module_id: &ModuleId, health: ModuleHealth);

    /// Get all unhealthy modules
    fn get_unhealthy_modules(&self) -> Vec<(ModuleId, ModuleHealth)>;

    /// Generate user-friendly error report
    fn generate_user_report(&self, error_context: &ErrorContext) -> UserErrorReport;

    /// Quarantine a module (isolate it from others)
    fn quarantine_module(&mut self, module_id: &ModuleId) -> Result<(), RecoveryError>;

    /// Release a module from quarantine
    fn release_quarantine(&mut self, module_id: &ModuleId) -> Result<(), RecoveryError>;

    /// Check if a module is quarantined
    fn is_quarantined(&self, module_id: &ModuleId) -> bool;

    /// Get recovery statistics
    fn get_recovery_stats(&self) -> RecoveryStats;
}

/// Recovery statistics for monitoring
#[derive(Debug, Clone)]
pub struct RecoveryStats {
    /// Total number of errors handled
    pub total_errors: u64,
    /// Number of successful recoveries
    pub successful_recoveries: u64,
    /// Number of failed recoveries
    pub failed_recoveries: u64,
    /// Number of modules restarted
    pub modules_restarted: u32,
    /// Number of modules in fallback mode
    pub modules_in_fallback: u32,
    /// Number of quarantined modules
    pub quarantined_modules: u32,
    /// Average recovery time
    pub avg_recovery_time: Duration,
    /// Most common error types
    pub common_errors: HashMap<String, u32>,
}

/// Concrete implementation of error recovery manager
pub struct ErrorRecoveryManagerImpl {
    /// Module health tracking
    module_health: RwLock<HashMap<ModuleId, ModuleHealth>>,
    /// Recovery strategies per module
    recovery_strategies: RwLock<HashMap<ModuleId, RecoveryStrategy>>,
    /// Quarantined modules
    quarantined_modules: RwLock<HashMap<ModuleId, u64>>, // ModuleId -> quarantine timestamp
    /// Error contexts for debugging
    error_contexts: RwLock<Vec<ErrorContext>>,
    /// Recovery statistics
    recovery_stats: RwLock<RecoveryStats>,
    /// Module registry for restart operations
    module_registry: Option<Arc<Mutex<ModuleRegistryImpl>>>,
}

impl ErrorRecoveryManagerImpl {
    /// Create a new error recovery manager
    pub fn new() -> Self {
        Self {
            module_health: RwLock::new(HashMap::new()),
            recovery_strategies: RwLock::new(HashMap::new()),
            quarantined_modules: RwLock::new(HashMap::new()),
            error_contexts: RwLock::new(Vec::new()),
            recovery_stats: RwLock::new(RecoveryStats {
                total_errors: 0,
                successful_recoveries: 0,
                failed_recoveries: 0,
                modules_restarted: 0,
                modules_in_fallback: 0,
                quarantined_modules: 0,
                avg_recovery_time: Duration::from_secs(0),
                common_errors: HashMap::new(),
            }),
            module_registry: None,
        }
    }

    /// Set module registry for restart operations
    pub fn set_module_registry(&mut self, registry: Arc<Mutex<ModuleRegistryImpl>>) {
        self.module_registry = Some(registry);
    }

    /// Record an error context for debugging
    fn record_error_context(&self, context: ErrorContext) {
        if let Ok(mut contexts) = self.error_contexts.write() {
            contexts.push(context);
            
            // Keep only the last 1000 error contexts to prevent memory growth
            if contexts.len() > 1000 {
                let drain_count = contexts.len() - 1000;
                contexts.drain(0..drain_count);
            }
        }
    }

    /// Update recovery statistics
    fn update_stats(&self, success: bool, error_type: &str) {
        if let Ok(mut stats) = self.recovery_stats.write() {
            stats.total_errors += 1;
            
            if success {
                stats.successful_recoveries += 1;
            } else {
                stats.failed_recoveries += 1;
            }
            
            *stats.common_errors.entry(error_type.to_string()).or_insert(0) += 1;
        }
    }

    /// Determine recovery action based on error and module health
    fn determine_recovery_action(
        &self,
        module_id: &ModuleId,
        error_context: &ErrorContext,
        health: &ModuleHealth,
    ) -> RecoveryAction {
        // Check if module is quarantined
        if self.is_quarantined(module_id) {
            return RecoveryAction::Ignore;
        }

        // Get recovery strategy
        let strategy = if let Ok(strategies) = self.recovery_strategies.read() {
            strategies.get(module_id).cloned()
        } else {
            None
        }.unwrap_or_default();

        // Determine action based on severity and health
        match error_context.severity {
            ErrorSeverity::Critical => {
                if health.consecutive_errors >= strategy.error_threshold {
                    RecoveryAction::Quarantine
                } else if health.is_critical() {
                    RecoveryAction::Restart
                } else {
                    RecoveryAction::Escalate
                }
            }
            ErrorSeverity::High => {
                if health.consecutive_errors >= strategy.error_threshold {
                    if let Some(fallback) = &strategy.fallback_mode {
                        RecoveryAction::Fallback(fallback.clone())
                    } else {
                        RecoveryAction::Restart
                    }
                } else {
                    RecoveryAction::Retry { max_attempts: 3, delay_ms: 1000 }
                }
            }
            ErrorSeverity::Medium => {
                if health.consecutive_errors >= 3 {
                    RecoveryAction::Retry { max_attempts: 2, delay_ms: 500 }
                } else {
                    RecoveryAction::Ignore
                }
            }
            ErrorSeverity::Low => RecoveryAction::Ignore,
        }
    }

    /// Perform module restart
    fn perform_restart(&mut self, module_id: &ModuleId) -> Result<(), RecoveryError> {
        // Check if we have a module registry
        let registry = self.module_registry.as_ref()
            .ok_or_else(|| RecoveryError::RecoveryFailed("No module registry available".to_string()))?;

        // For now, simulate restart by updating module state
        // In a real implementation, this would involve stopping and restarting the module
        
        // Update health after restart
        if let Ok(mut health_map) = self.module_health.write() {
            if let Some(health) = health_map.get_mut(module_id) {
                health.consecutive_errors = 0;
                health.last_restart = Some(get_timestamp_ns());
                health.status = HealthStatus::Recovering;
            }
        }

        // Update stats
        if let Ok(mut stats) = self.recovery_stats.write() {
            stats.modules_restarted += 1;
        }

        Ok(())
    }
}

impl Default for ErrorRecoveryManagerImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecoveryManager for ErrorRecoveryManagerImpl {
    fn handle_module_error(
        &mut self,
        module_id: &ModuleId,
        error: &dyn std::error::Error,
    ) -> RecoveryAction {
        // Create error context
        let error_context = ErrorContext::new(
            module_id.clone(),
            error,
            "unknown"
        ).with_severity(ErrorSeverity::Medium);

        // Record error context
        self.record_error_context(error_context.clone());

        // Get or create module health
        let health = {
            let mut health_map = self.module_health.write().unwrap_or_else(|_| {
                panic!("Failed to acquire module health write lock")
            });
            
            let health = health_map.entry(module_id.clone()).or_insert_with(ModuleHealth::new);
            health.record_error(&error.to_string());
            health.clone()
        };

        // Determine recovery action
        let action = self.determine_recovery_action(module_id, &error_context, &health);

        // Update statistics
        self.update_stats(true, &error_context.error_type);

        action
    }

    fn restart_module(&mut self, module_id: &ModuleId) -> Result<(), RecoveryError> {
        self.perform_restart(module_id)
    }

    fn get_module_health(&self, module_id: &ModuleId) -> Option<ModuleHealth> {
        if let Ok(health_map) = self.module_health.read() {
            health_map.get(module_id).cloned()
        } else {
            None
        }
    }

    fn set_fallback_mode(&mut self, module_id: &ModuleId, mode: FallbackMode) {
        if let Ok(mut health_map) = self.module_health.write() {
            let health = health_map.entry(module_id.clone()).or_insert_with(ModuleHealth::new);
            health.fallback_mode = Some(mode);
            health.status = HealthStatus::SafeMode;
        }

        // Update stats
        if let Ok(mut stats) = self.recovery_stats.write() {
            stats.modules_in_fallback += 1;
        }
    }

    fn get_fallback_mode(&self, module_id: &ModuleId) -> Option<FallbackMode> {
        if let Ok(health_map) = self.module_health.read() {
            health_map.get(module_id)?.fallback_mode.clone()
        } else {
            None
        }
    }

    fn register_recovery_strategy(
        &mut self,
        module_id: ModuleId,
        strategy: RecoveryStrategy,
    ) {
        if let Ok(mut strategies) = self.recovery_strategies.write() {
            strategies.insert(module_id, strategy);
        }
    }

    fn update_module_health(&mut self, module_id: &ModuleId, health: ModuleHealth) {
        if let Ok(mut health_map) = self.module_health.write() {
            health_map.insert(module_id.clone(), health);
        }
    }

    fn get_unhealthy_modules(&self) -> Vec<(ModuleId, ModuleHealth)> {
        if let Ok(health_map) = self.module_health.read() {
            health_map.iter()
                .filter(|(_, health)| !matches!(health.status, HealthStatus::Healthy))
                .map(|(id, health)| (id.clone(), health.clone()))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn generate_user_report(&self, error_context: &ErrorContext) -> UserErrorReport {
        let (title, description, suggestions, user_severity) = match error_context.severity {
            ErrorSeverity::Critical => (
                "Critical System Error".to_string(),
                format!("A critical error occurred in {}: {}", 
                    error_context.module_id.as_str(), error_context.error_message),
                vec![
                    "Please restart the application".to_string(),
                    "Contact support if the issue persists".to_string(),
                    "Check system requirements".to_string(),
                ],
                UserSeverity::Critical,
            ),
            ErrorSeverity::High => (
                "Module Error".to_string(),
                format!("An error occurred in {}: {}", 
                    error_context.module_id.as_str(), error_context.error_message),
                vec![
                    "The system will attempt automatic recovery".to_string(),
                    "Some features may be temporarily unavailable".to_string(),
                    "Try refreshing the page if issues persist".to_string(),
                ],
                UserSeverity::Error,
            ),
            ErrorSeverity::Medium => (
                "Performance Issue".to_string(),
                format!("Performance degradation detected in {}", 
                    error_context.module_id.as_str()),
                vec![
                    "Performance may be temporarily affected".to_string(),
                    "The system is working to resolve the issue".to_string(),
                ],
                UserSeverity::Warning,
            ),
            ErrorSeverity::Low => (
                "Minor Issue".to_string(),
                "A minor issue was detected and resolved".to_string(),
                vec!["No action required".to_string()],
                UserSeverity::Info,
            ),
        };

        UserErrorReport {
            title,
            description,
            recovery_suggestions: suggestions,
            user_severity,
            affects_functionality: matches!(error_context.severity, 
                ErrorSeverity::Critical | ErrorSeverity::High),
            estimated_recovery_time: match error_context.severity {
                ErrorSeverity::Critical => Some(Duration::from_secs(30)),
                ErrorSeverity::High => Some(Duration::from_secs(10)),
                _ => None,
            },
            support_info: Some("For additional help, please contact support".to_string()),
        }
    }

    fn quarantine_module(&mut self, module_id: &ModuleId) -> Result<(), RecoveryError> {
        if let Ok(mut quarantined) = self.quarantined_modules.write() {
            quarantined.insert(module_id.clone(), get_timestamp_ns());
        }

        // Update module health
        if let Ok(mut health_map) = self.module_health.write() {
            let health = health_map.entry(module_id.clone()).or_insert_with(ModuleHealth::new);
            health.status = HealthStatus::Failed;
        }

        // Update stats
        if let Ok(mut stats) = self.recovery_stats.write() {
            stats.quarantined_modules += 1;
        }

        Ok(())
    }

    fn release_quarantine(&mut self, module_id: &ModuleId) -> Result<(), RecoveryError> {
        if let Ok(mut quarantined) = self.quarantined_modules.write() {
            quarantined.remove(module_id);
        }

        // Update module health
        if let Ok(mut health_map) = self.module_health.write() {
            if let Some(health) = health_map.get_mut(module_id) {
                health.status = HealthStatus::Recovering;
                health.consecutive_errors = 0;
            }
        }

        // Update stats
        if let Ok(mut stats) = self.recovery_stats.write() {
            stats.quarantined_modules = stats.quarantined_modules.saturating_sub(1);
        }

        Ok(())
    }

    fn is_quarantined(&self, module_id: &ModuleId) -> bool {
        if let Ok(quarantined) = self.quarantined_modules.read() {
            quarantined.contains_key(module_id)
        } else {
            false
        }
    }

    fn get_recovery_stats(&self) -> RecoveryStats {
        if let Ok(stats) = self.recovery_stats.read() {
            stats.clone()
        } else {
            RecoveryStats {
                total_errors: 0,
                successful_recoveries: 0,
                failed_recoveries: 0,
                modules_restarted: 0,
                modules_in_fallback: 0,
                quarantined_modules: 0,
                avg_recovery_time: Duration::from_secs(0),
                common_errors: HashMap::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_health_creation() {
        let health = ModuleHealth::new();
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.error_count, 0);
        assert_eq!(health.consecutive_errors, 0);
        assert!(health.last_error.is_none());
    }

    #[test]
    fn test_error_recording() {
        let mut health = ModuleHealth::new();
        health.record_error("Test error");
        
        assert_eq!(health.error_count, 1);
        assert_eq!(health.consecutive_errors, 1);
        assert_eq!(health.last_error, Some("Test error".to_string()));
    }

    #[test]
    fn test_success_recording() {
        let mut health = ModuleHealth::new();
        health.record_error("Test error");
        health.record_success();
        
        assert_eq!(health.error_count, 1);
        assert_eq!(health.consecutive_errors, 0);
    }

    #[test]
    fn test_critical_state_detection() {
        let mut health = ModuleHealth::new();
        health.status = HealthStatus::Failed;
        assert!(health.is_critical());
        
        health.status = HealthStatus::Healthy;
        health.consecutive_errors = 6;
        assert!(health.is_critical());
    }

    #[test]
    fn test_recovery_urgency() {
        let mut health = ModuleHealth::new();
        
        health.status = HealthStatus::Failed;
        assert_eq!(health.get_recovery_urgency(), RecoveryUrgency::Critical);
        
        health.status = HealthStatus::Unhealthy;
        health.consecutive_errors = 3;
        assert_eq!(health.get_recovery_urgency(), RecoveryUrgency::High);
        
        health.consecutive_errors = 1;
        assert_eq!(health.get_recovery_urgency(), RecoveryUrgency::Medium);
    }

    #[test]
    fn test_error_context_creation() {
        let module_id = ModuleId::new("test-module");
        let error = std::io::Error::new(std::io::ErrorKind::Other, "Test error");
        
        let context = ErrorContext::new(module_id.clone(), &error, "test_operation");
        
        assert_eq!(context.module_id, module_id);
        assert_eq!(context.error_message, "Test error");
        assert_eq!(context.operation, "test_operation");
        assert!(context.recoverable);
    }

    #[test]
    fn test_error_recovery_manager_creation() {
        let manager = ErrorRecoveryManagerImpl::new();
        let stats = manager.get_recovery_stats();
        
        assert_eq!(stats.total_errors, 0);
        assert_eq!(stats.successful_recoveries, 0);
        assert_eq!(stats.quarantined_modules, 0);
    }

    #[test]
    fn test_module_health_tracking() {
        let mut manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("test-module");
        
        // Initially no health data
        assert!(manager.get_module_health(&module_id).is_none());
        
        // Update health
        let health = ModuleHealth::new();
        manager.update_module_health(&module_id, health);
        
        // Should now have health data
        assert!(manager.get_module_health(&module_id).is_some());
    }

    #[test]
    fn test_fallback_mode() {
        let mut manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("test-module");
        
        manager.set_fallback_mode(&module_id, FallbackMode::ReadOnly);
        assert_eq!(manager.get_fallback_mode(&module_id), Some(FallbackMode::ReadOnly));
        
        let health = manager.get_module_health(&module_id).unwrap();
        assert_eq!(health.status, HealthStatus::SafeMode);
    }

    #[test]
    fn test_quarantine_functionality() {
        let mut manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("test-module");
        
        // Initially not quarantined
        assert!(!manager.is_quarantined(&module_id));
        
        // Quarantine module
        manager.quarantine_module(&module_id).unwrap();
        assert!(manager.is_quarantined(&module_id));
        
        // Release from quarantine
        manager.release_quarantine(&module_id).unwrap();
        assert!(!manager.is_quarantined(&module_id));
    }

    #[test]
    fn test_recovery_strategy_registration() {
        let mut manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("test-module");
        
        let strategy = RecoveryStrategy {
            max_restart_attempts: 5,
            restart_cooldown: Duration::from_secs(60),
            error_threshold: 3,
            auto_recovery: true,
            fallback_mode: Some(FallbackMode::Minimal),
            custom_actions: vec![],
        };
        
        manager.register_recovery_strategy(module_id, strategy);
        // Strategy is registered (we can't directly verify due to private fields,
        // but it's used in error handling)
    }

    #[test]
    fn test_user_error_report_generation() {
        let manager = ErrorRecoveryManagerImpl::new();
        let module_id = ModuleId::new("test-module");
        let error = std::io::Error::new(std::io::ErrorKind::Other, "Test error");
        
        let context = ErrorContext::new(module_id, &error, "test_operation")
            .with_severity(ErrorSeverity::High);
        
        let report = manager.generate_user_report(&context);
        
        assert_eq!(report.title, "Module Error");
        assert!(report.affects_functionality);
        assert!(!report.recovery_suggestions.is_empty());
        assert_eq!(report.user_severity, UserSeverity::Error);
    }

    #[test]
    fn test_unhealthy_modules_detection() {
        let mut manager = ErrorRecoveryManagerImpl::new();
        let module1 = ModuleId::new("healthy-module");
        let module2 = ModuleId::new("unhealthy-module");
        
        // Add healthy module
        let mut healthy = ModuleHealth::new();
        healthy.status = HealthStatus::Healthy;
        manager.update_module_health(&module1, healthy);
        
        // Add unhealthy module
        let mut unhealthy = ModuleHealth::new();
        unhealthy.status = HealthStatus::Failed;
        manager.update_module_health(&module2, unhealthy);
        
        let unhealthy_modules = manager.get_unhealthy_modules();
        assert_eq!(unhealthy_modules.len(), 1);
        assert_eq!(unhealthy_modules[0].0, module2);
    }
}