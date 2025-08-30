use crate::engine::platform::PlatformValidationResult;
use crate::common::shared_types::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    None,
    Recoverable,
    Fatal,
}

/// Handle runtime errors and return the highest severity level encountered
pub fn handle_runtime_errors(errors: &Vec<Error>) -> ErrorSeverity {
    for error in errors {
        match error {
            Error::MobileDeviceNotSupported => {
                #[cfg(target_arch = "wasm32")]
                crate::web::error_message_box::show_error(&Error::MobileDeviceNotSupported);
                return ErrorSeverity::Fatal;
            }
            Error::BrowserApiNotSupported => {
                return ErrorSeverity::Fatal;
            }
            Error::BrowserError => {
                #[cfg(target_arch = "wasm32")]
                crate::web::error_message_box::show_error(&Error::BrowserError);
                return ErrorSeverity::Fatal;
            }
            Error::MicrophonePermissionDenied => {
                return ErrorSeverity::Fatal;
            }
            Error::MicrophoneNotAvailable => {
                #[cfg(target_arch = "wasm32")]
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
