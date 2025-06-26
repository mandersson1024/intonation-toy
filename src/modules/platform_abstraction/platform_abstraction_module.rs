use super::*;
use crate::modules::application_core::{Module, ModuleId, TypedEventBus, Event, EventPriority};
use std::sync::{Arc, Mutex};
use std::any::Any;
use std::time::Instant;

/// Main Platform Abstraction Module coordinating all platform services
pub struct PlatformAbstractionModule {
    /// Module metadata
    module_id: ModuleId,
    module_name: String,
    module_version: String,
    
    /// Core platform components
    browser_compat: Option<Arc<dyn BrowserCompatibility>>,
    device_capabilities: Option<Arc<dyn DeviceCapabilityDetector>>,
    wasm_bridge: Option<Arc<dyn WasmBridge>>,
    optimization_engine: Option<Arc<dyn PlatformOptimizationEngine>>,
    error_recovery: Option<Arc<ErrorRecoveryManager>>,
    
    /// Event bus for platform events
    event_bus: Option<Arc<Mutex<TypedEventBus>>>,
    
    /// Module state
    initialized: bool,
    started: bool,
}

impl PlatformAbstractionModule {
    /// Create a new Platform Abstraction Module
    pub fn new() -> Self {
        Self {
            module_id: ModuleId::new("platform_abstraction"),
            module_name: "Platform Abstraction".to_string(),
            module_version: "1.0.0".to_string(),
            browser_compat: None,
            device_capabilities: None,
            wasm_bridge: None,
            optimization_engine: None,
            error_recovery: None,
            event_bus: None,
            initialized: false,
            started: false,
        }
    }
    
    /// Set the browser compatibility component
    pub fn with_browser_compatibility(mut self, browser_compat: Arc<dyn BrowserCompatibility>) -> Self {
        self.browser_compat = Some(browser_compat);
        self
    }
    
    /// Set the device capability detector
    pub fn with_device_capabilities(mut self, device_capabilities: Arc<dyn DeviceCapabilityDetector>) -> Self {
        self.device_capabilities = Some(device_capabilities);
        self
    }
    
    /// Set the WebAssembly bridge
    pub fn with_wasm_bridge(mut self, wasm_bridge: Arc<dyn WasmBridge>) -> Self {
        self.wasm_bridge = Some(wasm_bridge);
        self
    }
    
    /// Set the platform optimization engine
    pub fn with_optimization_engine(mut self, optimization_engine: Arc<dyn PlatformOptimizationEngine>) -> Self {
        self.optimization_engine = Some(optimization_engine);
        self
    }
    
    pub fn with_error_recovery(mut self, error_recovery: Arc<ErrorRecoveryManager>) -> Self {
        self.error_recovery = Some(error_recovery);
        self
    }
    
    /// Set the event bus
    pub fn with_event_bus(mut self, event_bus: Arc<Mutex<TypedEventBus>>) -> Self {
        self.event_bus = Some(event_bus);
        self
    }
    
    /// Get browser compatibility service
    pub fn browser_compatibility(&self) -> Option<&Arc<dyn BrowserCompatibility>> {
        self.browser_compat.as_ref()
    }
    
    /// Get device capability detector
    pub fn device_capabilities(&self) -> Option<&Arc<dyn DeviceCapabilityDetector>> {
        self.device_capabilities.as_ref()
    }
    
    /// Get WebAssembly bridge
    pub fn wasm_bridge(&self) -> Option<&Arc<dyn WasmBridge>> {
        self.wasm_bridge.as_ref()
    }
    
    /// Get platform optimization engine
    pub fn optimization_engine(&self) -> Option<&Arc<dyn PlatformOptimizationEngine>> {
        self.optimization_engine.as_ref()
    }
    
    /// Detect platform and publish ready event
    fn detect_and_publish_platform_ready(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(browser_compat), Some(device_capabilities)) = 
            (self.browser_compat.as_ref(), self.device_capabilities.as_ref()) {
            
            // Detect browser capabilities
            let browser_info = browser_compat.detect_browser()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            
            // Detect device capabilities
            let capabilities = device_capabilities.detect_all()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            
            // Apply platform optimizations if optimization engine is available
            if let Some(optimization_engine) = self.optimization_engine.as_ref() {
                optimization_engine.apply_optimizations(&browser_info, &capabilities)
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
            }
            
            // Publish platform ready event
            self.publish_platform_ready_event(&browser_info, &capabilities)?;
            
            // Log platform detection (using console for WASM compatibility)
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("Platform abstraction module detected platform: {} {}.{}.{}", 
                browser_info.browser_name,
                browser_info.version.major,
                browser_info.version.minor,
                browser_info.version.patch
            ).into());
            
            #[cfg(not(target_arch = "wasm32"))]
            println!("Platform abstraction module detected platform: {} {}.{}.{}", 
                browser_info.browser_name,
                browser_info.version.major,
                browser_info.version.minor,
                browser_info.version.patch
            );
        }
        
        Ok(())
    }
    
    /// Publish platform ready event
    fn publish_platform_ready_event(&self, browser_info: &BrowserInfo, capabilities: &DeviceCapabilities) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(event_bus) = self.event_bus.as_ref() {
            let event = PlatformReadyEvent {
                timestamp: Instant::now(),
                browser_info: browser_info.clone(),
                capabilities: capabilities.clone(),
                module_id: self.module_id.clone(),
            };
            
            let mut bus = event_bus.lock().unwrap();
            bus.publish(event)?;
            
            // Log event publishing (using console for WASM compatibility)
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("Published platform ready event for browser: {}", browser_info.browser_name).into());
            
            #[cfg(not(target_arch = "wasm32"))]
            println!("Published platform ready event for browser: {}", browser_info.browser_name);
        }
        Ok(())
    }
}

impl Default for PlatformAbstractionModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for PlatformAbstractionModule {
    fn module_id(&self) -> ModuleId {
        self.module_id.clone()
    }
    
    fn module_name(&self) -> &str {
        &self.module_name
    }
    
    fn module_version(&self) -> &str {
        &self.module_version
    }
    
    fn dependencies(&self) -> Vec<ModuleId> {
        vec![
            ModuleId::new("application_core"),
            ModuleId::new("audio_foundations"), // For device integration
        ]
    }
    
    fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.initialized {
            return Ok(());
        }
        
        // Log initialization (using console for WASM compatibility)
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("Initializing Platform Abstraction Module v{}", self.module_version).into());
        
        #[cfg(not(target_arch = "wasm32"))]
        println!("Initializing Platform Abstraction Module v{}", self.module_version);
        
        // Validate required components are present
        if self.browser_compat.is_none() {
            return Err("Browser compatibility component not set".into());
        }
        
        if self.device_capabilities.is_none() {
            return Err("Device capabilities component not set".into());
        }
        
        // Initialize platform detection and optimization
        self.detect_and_publish_platform_ready()?;
        
        self.initialized = true;
        // Log successful initialization
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Platform Abstraction Module initialized successfully".into());
        
        #[cfg(not(target_arch = "wasm32"))]
        println!("Platform Abstraction Module initialized successfully");
        
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.initialized {
            return Err("Module must be initialized before starting".into());
        }
        
        if self.started {
            return Ok(());
        }
        
        // Log module start
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Starting Platform Abstraction Module".into());
        
        #[cfg(not(target_arch = "wasm32"))]
        println!("Starting Platform Abstraction Module");
        
        // Start device capability monitoring if available
        if let Some(device_capabilities) = self.device_capabilities.as_ref() {
            device_capabilities.start_capability_monitoring()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        }
        
        self.started = true;
        // Log successful start
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Platform Abstraction Module started successfully".into());
        
        #[cfg(not(target_arch = "wasm32"))]
        println!("Platform Abstraction Module started successfully");
        
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.started {
            return Ok(());
        }
        
        // Log module stop
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Stopping Platform Abstraction Module".into());
        
        #[cfg(not(target_arch = "wasm32"))]
        println!("Stopping Platform Abstraction Module");
        
        // Stop any ongoing monitoring or services
        // TODO: Implement cleanup logic for device monitoring
        
        self.started = false;
        // Log successful stop
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Platform Abstraction Module stopped successfully".into());
        
        #[cfg(not(target_arch = "wasm32"))]
        println!("Platform Abstraction Module stopped successfully");
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.started {
            self.stop()?;
        }
        
        // Log module shutdown
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Shutting down Platform Abstraction Module".into());
        
        #[cfg(not(target_arch = "wasm32"))]
        println!("Shutting down Platform Abstraction Module");
        
        // Clear all components
        self.browser_compat = None;
        self.device_capabilities = None;
        self.wasm_bridge = None;
        self.optimization_engine = None;
        self.event_bus = None;
        
        self.initialized = false;
        // Log shutdown complete
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Platform Abstraction Module shutdown complete".into());
        
        #[cfg(not(target_arch = "wasm32"))]
        println!("Platform Abstraction Module shutdown complete");
        
        Ok(())
    }
}

// Note: ModuleAny is implemented automatically via blanket implementation 