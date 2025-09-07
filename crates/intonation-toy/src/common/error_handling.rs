#![cfg(target_arch = "wasm32")]

use crate::engine::platform::PlatformValidationResult;
use crate::common::shared_types::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    None,
    Recoverable,
    Fatal,
}

/// Handle runtime errors and return the highest severity level encountered
pub fn handle_runtime_errors(errors: &[Error]) -> ErrorSeverity {
    if let Some(error) = errors.iter().next() {
        match error {
            Error::MobileDeviceNotSupported => {
                crate::web::error_message_box::show_error(&Error::MobileDeviceNotSupported);
                return ErrorSeverity::Fatal;
            }
            Error::BrowserApiNotSupported => {
                return ErrorSeverity::Fatal;
            }
            Error::BrowserError => {
                crate::web::error_message_box::show_error(&Error::BrowserError);
                return ErrorSeverity::Fatal;
            }
            Error::MicrophonePermissionDenied => {
                return ErrorSeverity::Fatal;
            }
            Error::MicrophoneNotAvailable => {
                crate::web::error_message_box::show_error(&Error::MicrophoneNotAvailable);
                return ErrorSeverity::Fatal;
            }
            Error::ProcessingError(msg) => {
                crate::common::error_log!("🔥 PROCESSING ERROR: {}", msg);
                return ErrorSeverity::Recoverable;
            }
        };
    }

    ErrorSeverity::None
}

pub fn handle_platform_validation_error(result: PlatformValidationResult) {
    match result {
        PlatformValidationResult::AllSupported => (),
        PlatformValidationResult::MobileDevice => {
            crate::web::error_message_box::show_error(&crate::common::shared_types::Error::MobileDeviceNotSupported);
        }
        PlatformValidationResult::MissingCriticalApis(missing_apis) => {
            let api_list: Vec<String> = missing_apis.iter().map(|api| api.to_string()).collect();
            let missing_apis_str = api_list.join(", ");
            crate::web::error_message_box::show_error_with_params(&crate::common::shared_types::Error::BrowserApiNotSupported, &[&missing_apis_str]);
        }
    }
}
