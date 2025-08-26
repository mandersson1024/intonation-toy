use web_sys::{AudioContext, AudioContextOptions};
use crate::common::dev_log;
use super::super::AudioError;
use super::AudioContextState;
use crate::app_config::STANDARD_SAMPLE_RATE;

pub struct AudioContextManager {
    context: Option<AudioContext>,
    state: AudioContextState,
}


impl Default for AudioContextManager {
    fn default() -> Self {
        Self {
            context: None,
            state: AudioContextState::Uninitialized,
        }
    }
}

impl AudioContextManager {
    pub fn state(&self) -> &AudioContextState {
        &self.state
    }
    
    pub fn initialize(&mut self) -> Result<(), AudioError> {
        self.state = AudioContextState::Initializing;
        let options = AudioContextOptions::new();
        options.set_sample_rate(STANDARD_SAMPLE_RATE as f32);
        
        let context = AudioContext::new_with_context_options(&options)
            .map_err(|e| {
                dev_log!("✗ Failed to create AudioContext: {:?}", e);
                self.state = AudioContextState::Closed;
                AudioError::StreamInitFailed(format!("Failed to create AudioContext: {:?}", e))
            })?;
            
        dev_log!("✓ AudioContext created");
        self.context = Some(context);
        self.state = AudioContextState::Running;
        Ok(())
    }
    
    pub fn close(&mut self) -> Result<(), AudioError> {
        if let Some(context) = &self.context {
            dev_log!("Closing AudioContext");
            let _ = context.close();
        }
        
        self.context = None;
        self.state = AudioContextState::Closed;
        Ok(())
    }
    
    
    pub fn get_context(&self) -> Option<&AudioContext> {
        self.context.as_ref()
    }
    
    
    pub fn is_running(&self) -> bool {
        matches!(self.state, AudioContextState::Running) &&
        self.context.as_ref()
            .is_some_and(|ctx| ctx.state() == web_sys::AudioContextState::Running)
    }
    
    
    

    


}

