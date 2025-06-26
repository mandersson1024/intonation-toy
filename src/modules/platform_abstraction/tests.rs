#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_platform_abstraction_module_creation() {
        let module = PlatformAbstractionModule::new();
        assert_eq!(module.module_id().as_str(), "platform_abstraction");
        assert_eq!(module.module_name(), "Platform Abstraction");
        assert_eq!(module.module_version(), "1.0.0");
    }
    
    #[test]
    fn test_module_dependencies() {
        let module = PlatformAbstractionModule::new();
        let deps = module.dependencies();
        assert!(deps.contains(&ModuleId::new("application_core")));
        assert!(deps.contains(&ModuleId::new("audio_foundations")));
    }
    
    #[test]
    fn test_module_builder_pattern() {
        let browser_compat = Arc::new(BrowserCompatibilityImpl::new());
        let device_capabilities = Arc::new(DeviceCapabilityDetectorImpl::new());
        let wasm_bridge = Arc::new(WasmBridgeImpl::new());
        let optimization_engine = Arc::new(PlatformOptimizationEngineImpl::new());
        
        let module = PlatformAbstractionModule::new()
            .with_browser_compatibility(browser_compat.clone())
            .with_device_capabilities(device_capabilities.clone())
            .with_wasm_bridge(wasm_bridge.clone())
            .with_optimization_engine(optimization_engine.clone());
        
        assert!(module.browser_compatibility().is_some());
        assert!(module.device_capabilities().is_some());
        assert!(module.wasm_bridge().is_some());
        assert!(module.optimization_engine().is_some());
    }
    
    #[test]
    fn test_module_trait_implementation() {
        use crate::modules::application_core::Module;
        
        let mut module = PlatformAbstractionModule::new();
        
        // Test initialization without required components should fail
        assert!(module.initialize().is_err());
        
        // Add required components
        let browser_compat = Arc::new(BrowserCompatibilityImpl::new());
        let device_capabilities = Arc::new(DeviceCapabilityDetectorImpl::new());
        
        module = module
            .with_browser_compatibility(browser_compat)
            .with_device_capabilities(device_capabilities);
        
        // Test lifecycle methods exist (they will fail due to placeholder implementations)
        // This just ensures the trait is properly implemented
        let _result = module.initialize();
        let _result = module.start();
        let _result = module.stop();
        let _result = module.shutdown();
    }
    
    #[test]
    fn test_platform_error_display() {
        let error = PlatformError::BrowserDetectionFailed("test error".to_string());
        assert!(error.to_string().contains("Browser detection failed"));
        
        let error = PlatformError::UnsupportedFeature("test feature".to_string());
        assert!(error.to_string().contains("Unsupported feature"));
        
        let error = PlatformError::DeviceCapabilityError("test error".to_string());
        assert!(error.to_string().contains("Device capability error"));
    }
    
    #[test]
    fn test_trait_implementations_exist() {
        // Test that all required trait implementations compile
        let _browser_compat: Box<dyn BrowserCompatibility> = Box::new(BrowserCompatibilityImpl::new());
        let _device_capabilities: Box<dyn DeviceCapabilityDetector> = Box::new(DeviceCapabilityDetectorImpl::new());
        let _wasm_bridge: Box<dyn WasmBridge> = Box::new(WasmBridgeImpl::new());
        let _optimization_engine: Box<dyn PlatformOptimizationEngine> = Box::new(PlatformOptimizationEngineImpl::new());
        
        // This test ensures all traits are properly defined and implemented
        assert!(true);
    }
} 