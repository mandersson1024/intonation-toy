//! Application Data Module
//!
//! This module contains the central LiveData struct that holds observable data
//! for the entire application. Components can subscribe to changes in this data
//! to update their state reactively.

use observable_data::ObservableData;
use crate::audio::AudioPermission;

/// Central application data store containing all observable application state
pub struct LiveData {
    /// Microphone permission status - components can observe this for updates
    pub microphone_permission: ObservableData<AudioPermission>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use observable_data::DataSource;

    #[test]
    fn test_live_data_with_data_source() {
        // Create data source
        let mut permission_source = DataSource::new(AudioPermission::Uninitialized);
        
        // Create LiveData with observable from data source
        let live_data = LiveData {
            microphone_permission: permission_source.observer(),
        };
        
        // Test initial value
        assert_eq!(live_data.microphone_permission.get(), AudioPermission::Uninitialized);
        
        // Update via data source
        permission_source.set(AudioPermission::Granted);
        
        // Test updated value is visible through observable
        assert_eq!(live_data.microphone_permission.get(), AudioPermission::Granted);
        
        // Update again
        permission_source.set(AudioPermission::Denied);
        assert_eq!(live_data.microphone_permission.get(), AudioPermission::Denied);
    }
}
