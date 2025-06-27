//! # Memory Leak Prevention for Debug Event Subscriptions
//!
//! This module provides utilities to prevent memory leaks in debug component event subscriptions.
//! It includes subscription tracking, automatic cleanup, and monitoring capabilities.

use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use std::time::{Duration, Instant};
use crate::modules::developer_ui::hooks::use_event_subscription::EventSubscriptionHandle;

/// Memory leak prevention manager for debug event subscriptions
#[cfg(debug_assertions)]
pub struct MemoryLeakPreventionManager {
    active_subscriptions: HashMap<u64, SubscriptionInfo>,
    subscription_counter: u64,
    cleanup_threshold: Duration,
    last_cleanup: Instant,
    max_subscriptions: usize,
    leak_detection_enabled: bool,
}

/// Information about an active subscription for leak detection
#[cfg(debug_assertions)]
#[derive(Debug)]
struct SubscriptionInfo {
    subscription_id: u64,
    component_name: String,
    event_type: String,
    created_at: Instant,
    last_activity: Instant,
    reference_count: usize,
}

/// Subscription lifecycle tracker for automatic cleanup
#[cfg(debug_assertions)]
pub struct SubscriptionLifecycleTracker {
    manager: Rc<RefCell<MemoryLeakPreventionManager>>,
    subscription_id: u64,
    component_name: String,
}

#[cfg(debug_assertions)]
impl MemoryLeakPreventionManager {
    /// Create a new memory leak prevention manager
    pub fn new() -> Self {
        Self {
            active_subscriptions: HashMap::new(),
            subscription_counter: 0,
            cleanup_threshold: Duration::from_secs(30), // Clean up inactive subscriptions after 30 seconds
            last_cleanup: Instant::now(),
            max_subscriptions: 1000, // Maximum allowed active subscriptions
            leak_detection_enabled: true,
        }
    }

    /// Register a new subscription for tracking
    pub fn register_subscription(
        &mut self,
        component_name: String,
        event_type: String,
    ) -> Result<SubscriptionLifecycleTracker, Box<dyn std::error::Error>> {
        // Check if we're approaching subscription limit
        if self.active_subscriptions.len() >= self.max_subscriptions {
            self.force_cleanup();
            
            if self.active_subscriptions.len() >= self.max_subscriptions {
                return Err(format!(
                    "Maximum subscription limit reached: {}. Possible memory leak detected.",
                    self.max_subscriptions
                ).into());
            }
        }

        self.subscription_counter += 1;
        let subscription_id = self.subscription_counter;
        
        let subscription_info = SubscriptionInfo {
            subscription_id,
            component_name: component_name.clone(),
            event_type: event_type.clone(),
            created_at: Instant::now(),
            last_activity: Instant::now(),
            reference_count: 1,
        };

        self.active_subscriptions.insert(subscription_id, subscription_info);
        
        web_sys::console::log_1(&format!(
            "Registered subscription {} for component '{}' event type '{}'",
            subscription_id, component_name, event_type
        ).into());

        Ok(SubscriptionLifecycleTracker {
            manager: Rc::new(RefCell::new(MemoryLeakPreventionManager::new())), // Placeholder
            subscription_id,
            component_name,
        })
    }

    /// Unregister a subscription (automatic cleanup)
    pub fn unregister_subscription(&mut self, subscription_id: u64) {
        if let Some(info) = self.active_subscriptions.remove(&subscription_id) {
            web_sys::console::log_1(&format!(
                "Unregistered subscription {} for component '{}' (lifetime: {:?})",
                subscription_id, info.component_name, info.created_at.elapsed()
            ).into());
        } else {
            web_sys::console::warn_1(&format!("Attempted to unregister unknown subscription: {}", subscription_id).into());
        }
    }

    /// Update subscription activity timestamp
    pub fn update_subscription_activity(&mut self, subscription_id: u64) {
        if let Some(info) = self.active_subscriptions.get_mut(&subscription_id) {
            info.last_activity = Instant::now();
        }
    }

    /// Force cleanup of inactive subscriptions
    pub fn force_cleanup(&mut self) {
        let now = Instant::now();
        let threshold = self.cleanup_threshold;
        
        let before_count = self.active_subscriptions.len();
        
        self.active_subscriptions.retain(|&id, info| {
            let inactive_duration = now.duration_since(info.last_activity);
            if inactive_duration > threshold {
                web_sys::console::warn_1(&format!(
                    "Cleaned up inactive subscription {} for component '{}' (inactive for {:?})",
                    id, info.component_name, inactive_duration
                ).into());
                false
            } else {
                true
            }
        });
        
        let cleaned_count = before_count - self.active_subscriptions.len();
        if cleaned_count > 0 {
            web_sys::console::log_1(&format!("Cleaned up {} inactive subscriptions", cleaned_count).into());
        }
        
        self.last_cleanup = now;
    }

    /// Perform automatic cleanup if needed
    pub fn maybe_cleanup(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_cleanup) > Duration::from_secs(10) {
            self.force_cleanup();
        }
    }

    /// Check for potential memory leaks
    pub fn detect_memory_leaks(&self) -> Vec<MemoryLeakWarning> {
        if !self.leak_detection_enabled {
            return Vec::new();
        }

        let mut warnings = Vec::new();
        let now = Instant::now();

        // Check for long-lived subscriptions
        for (id, info) in &self.active_subscriptions {
            let lifetime = now.duration_since(info.created_at);
            if lifetime > Duration::from_secs(300) { // 5 minutes
                warnings.push(MemoryLeakWarning {
                    warning_type: MemoryLeakWarningType::LongLivedSubscription,
                    subscription_id: *id,
                    component_name: info.component_name.clone(),
                    details: format!("Subscription alive for {:?}", lifetime),
                });
            }
        }

        // Check for excessive subscriptions from single component
        let mut component_counts: HashMap<String, usize> = HashMap::new();
        for info in self.active_subscriptions.values() {
            *component_counts.entry(info.component_name.clone()).or_insert(0) += 1;
        }

        for (component_name, count) in component_counts {
            if count > 50 { // More than 50 subscriptions per component
                warnings.push(MemoryLeakWarning {
                    warning_type: MemoryLeakWarningType::ExcessiveSubscriptions,
                    subscription_id: 0,
                    component_name,
                    details: format!("{} active subscriptions", count),
                });
            }
        }

        warnings
    }

    /// Get current subscription statistics
    pub fn get_statistics(&self) -> SubscriptionStatistics {
        let mut component_breakdown: HashMap<String, usize> = HashMap::new();
        let mut event_type_breakdown: HashMap<String, usize> = HashMap::new();
        
        for info in self.active_subscriptions.values() {
            *component_breakdown.entry(info.component_name.clone()).or_insert(0) += 1;
            *event_type_breakdown.entry(info.event_type.clone()).or_insert(0) += 1;
        }

        SubscriptionStatistics {
            total_active_subscriptions: self.active_subscriptions.len(),
            total_created_subscriptions: self.subscription_counter,
            component_breakdown,
            event_type_breakdown,
            last_cleanup: self.last_cleanup,
            cleanup_threshold: self.cleanup_threshold,
        }
    }

    /// Enable or disable leak detection
    pub fn set_leak_detection_enabled(&mut self, enabled: bool) {
        self.leak_detection_enabled = enabled;
    }

    /// Set the maximum allowed subscriptions
    pub fn set_max_subscriptions(&mut self, max: usize) {
        self.max_subscriptions = max;
    }

    /// Set the cleanup threshold
    pub fn set_cleanup_threshold(&mut self, threshold: Duration) {
        self.cleanup_threshold = threshold;
    }
}

/// Memory leak warning information
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct MemoryLeakWarning {
    pub warning_type: MemoryLeakWarningType,
    pub subscription_id: u64,
    pub component_name: String,
    pub details: String,
}

/// Types of memory leak warnings
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub enum MemoryLeakWarningType {
    LongLivedSubscription,
    ExcessiveSubscriptions,
    InactiveSubscription,
    OrphanedSubscription,
}

/// Subscription statistics for monitoring
#[cfg(debug_assertions)]
#[derive(Debug, Clone)]
pub struct SubscriptionStatistics {
    pub total_active_subscriptions: usize,
    pub total_created_subscriptions: u64,
    pub component_breakdown: HashMap<String, usize>,
    pub event_type_breakdown: HashMap<String, usize>,
    pub last_cleanup: Instant,
    pub cleanup_threshold: Duration,
}

#[cfg(debug_assertions)]
impl Drop for SubscriptionLifecycleTracker {
    fn drop(&mut self) {
        // Automatic cleanup when the tracker is dropped
        web_sys::console::log_1(&format!(
            "SubscriptionLifecycleTracker dropped for component '{}', subscription {}",
            self.component_name, self.subscription_id
        ).into());
        
        // In a real implementation, we would notify the manager here
        // This is a placeholder for the automatic cleanup
    }
}

/// Global memory leak prevention manager instance
#[cfg(debug_assertions)]
static mut GLOBAL_LEAK_PREVENTION_MANAGER: Option<MemoryLeakPreventionManager> = None;

/// Get or create the global memory leak prevention manager
#[cfg(debug_assertions)]
pub fn get_global_leak_prevention_manager() -> &'static mut MemoryLeakPreventionManager {
    unsafe {
        if GLOBAL_LEAK_PREVENTION_MANAGER.is_none() {
            GLOBAL_LEAK_PREVENTION_MANAGER = Some(MemoryLeakPreventionManager::new());
        }
        GLOBAL_LEAK_PREVENTION_MANAGER.as_mut().unwrap()
    }
}

/// Utility function to create a memory-safe event subscription
#[cfg(debug_assertions)]
pub fn create_safe_subscription(
    component_name: String,
    event_type: String,
) -> Result<SubscriptionLifecycleTracker, Box<dyn std::error::Error>> {
    let manager = get_global_leak_prevention_manager();
    manager.register_subscription(component_name, event_type)
}

/// Utility function to run memory leak detection
#[cfg(debug_assertions)]
pub fn check_for_memory_leaks() -> Vec<MemoryLeakWarning> {
    let manager = get_global_leak_prevention_manager();
    manager.detect_memory_leaks()
}

/// Utility function to get subscription statistics
#[cfg(debug_assertions)]
pub fn get_subscription_statistics() -> SubscriptionStatistics {
    let manager = get_global_leak_prevention_manager();
    manager.get_statistics()
}

#[cfg(test)]
#[cfg(debug_assertions)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_registration() {
        let mut manager = MemoryLeakPreventionManager::new();
        
        let tracker = manager.register_subscription(
            "TestComponent".to_string(),
            "TestEvent".to_string(),
        );
        
        assert!(tracker.is_ok());
        assert_eq!(manager.active_subscriptions.len(), 1);
    }

    #[test]
    fn test_subscription_cleanup() {
        let mut manager = MemoryLeakPreventionManager::new();
        manager.set_cleanup_threshold(Duration::from_millis(1));
        
        let _tracker = manager.register_subscription(
            "TestComponent".to_string(),
            "TestEvent".to_string(),
        );
        
        // Wait for cleanup threshold
        std::thread::sleep(Duration::from_millis(2));
        
        manager.force_cleanup();
        assert_eq!(manager.active_subscriptions.len(), 0);
    }

    #[test]
    fn test_memory_leak_detection() {
        let mut manager = MemoryLeakPreventionManager::new();
        manager.set_max_subscriptions(2);
        
        // Create more subscriptions than the limit
        let _tracker1 = manager.register_subscription("Comp1".to_string(), "Event1".to_string());
        let _tracker2 = manager.register_subscription("Comp2".to_string(), "Event2".to_string());
        let tracker3 = manager.register_subscription("Comp3".to_string(), "Event3".to_string());
        
        assert!(tracker3.is_err());
    }

    #[test]
    fn test_subscription_statistics() {
        let mut manager = MemoryLeakPreventionManager::new();
        
        let _tracker1 = manager.register_subscription("Comp1".to_string(), "Event1".to_string());
        let _tracker2 = manager.register_subscription("Comp1".to_string(), "Event2".to_string());
        let _tracker3 = manager.register_subscription("Comp2".to_string(), "Event1".to_string());
        
        let stats = manager.get_statistics();
        assert_eq!(stats.total_active_subscriptions, 3);
        assert_eq!(stats.component_breakdown.get("Comp1"), Some(&2));
        assert_eq!(stats.component_breakdown.get("Comp2"), Some(&1));
        assert_eq!(stats.event_type_breakdown.get("Event1"), Some(&2));
        assert_eq!(stats.event_type_breakdown.get("Event2"), Some(&1));
    }
}