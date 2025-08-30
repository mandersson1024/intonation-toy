use web_sys::AudioContext;
use crate::common::dev_log;
use super::super::AudioError;
use super::AudioContextState;

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
    
    pub fn attach_existing_context(&mut self, context: AudioContext) -> Result<(), AudioError> {
        self.state = AudioContextState::Initializing;
        dev_log!("✓ AudioContext attached from external source");
        
        // Map the web_sys::AudioContextState to our AudioContextState
        let context_state = match context.state() {
            web_sys::AudioContextState::Running => AudioContextState::Running,
            web_sys::AudioContextState::Suspended => AudioContextState::Suspended,
            web_sys::AudioContextState::Closed => AudioContextState::Closed,
            _ => AudioContextState::Closed, // Default to closed for unknown states
        };
        
        self.context = Some(context);
        self.state = context_state;
        Ok(())
    }

}

