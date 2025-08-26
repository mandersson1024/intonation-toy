use crate::engine::platform::PlatformValidationResult;

pub fn handle_platform_validation_error(result: PlatformValidationResult) {
    match result {
        PlatformValidationResult::AllSupported => (),
        PlatformValidationResult::MobileDevice => {
            #[cfg(target_arch = "wasm32")]
            {
                crate::web::error_message_box::show_error(&crate::common::shared_types::Error::MobileDeviceNotSupported);
            }
            return;
        }
        PlatformValidationResult::MissingCriticalApis(missing_apis) => {
            let api_list: Vec<String> = missing_apis.iter().map(|api| api.to_string()).collect();
            #[cfg(target_arch = "wasm32")]
            {
                let missing_apis_str = api_list.join(", ");
                crate::web::error_message_box::show_error_with_params(&crate::common::shared_types::Error::BrowserApiNotSupported, &[&missing_apis_str]);
            }
        }
    }
}
