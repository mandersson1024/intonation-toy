#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, Document, Element, HtmlElement, HtmlSelectElement, EventTarget};

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;

#[cfg(target_arch = "wasm32")]
use crate::common::dev_log;
#[cfg(target_arch = "wasm32")]
use crate::shared_types::{TuningSystem, MidiNote, increment_midi_note, decrement_midi_note};

/// Format a MIDI note number as a string (e.g., 60 -> "C4")
#[cfg(target_arch = "wasm32")]
fn format_midi_note(midi_note: MidiNote) -> String {
    let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    let note_index = (midi_note % 12) as usize;
    let octave = (midi_note as i16 / 12) - 1;
    format!("{}{}", note_names[note_index], octave)
}

#[cfg(target_arch = "wasm32")]
pub fn setup_main_scene_ui() {
    let Some(window) = window() else {
        dev_log!("Failed to get window");
        return;
    };

    let Some(document) = window.document() else {
        dev_log!("Failed to get document");
        return;
    };

    // Create container div
    let Ok(container) = document.create_element("div") else {
        dev_log!("Failed to create container div");
        return;
    };
    
    container.set_id("main-scene-ui-container");
    
    container.set_attribute("style", 
        "position: fixed; \
         top: 20px; \
         left: 50%; \
         transform: translateX(-50%); \
         display: flex; \
         gap: 24px; \
         align-items: center; \
         background: linear-gradient(135deg, rgba(30, 30, 30, 0.95), rgba(50, 50, 50, 0.95)); \
         padding: 16px 24px; \
         border-radius: 12px; \
         border: 1px solid rgba(255, 255, 255, 0.1); \
         box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3); \
         backdrop-filter: blur(10px); \
         font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; \
         z-index: 1000;").ok();

    // Create root note controls container
    let Ok(root_note_container) = document.create_element("div") else {
        dev_log!("Failed to create root note container");
        return;
    };
    root_note_container.set_attribute("style", "display: flex; align-items: center; gap: 10px;").ok();

    // Create root note label
    let Ok(root_note_label) = document.create_element("span") else {
        dev_log!("Failed to create root note label");
        return;
    };
    root_note_label.set_text_content(Some("Root Note:"));
    root_note_label.set_attribute("style", 
        "color: #ffffff; \
         font-family: inherit; \
         font-size: 14px; \
         font-weight: 500; \
         margin-right: 4px;").ok();

    // Create minus button
    let Ok(minus_button) = document.create_element("button") else {
        dev_log!("Failed to create minus button");
        return;
    };
    minus_button.set_id("root-note-minus");
    minus_button.set_text_content(Some("-"));
    minus_button.set_attribute("style", 
        "width: 32px; \
         height: 32px; \
         font-size: 18px; \
         font-weight: bold; \
         cursor: pointer; \
         background: linear-gradient(135deg, #4a4a4a, #3a3a3a); \
         color: #ffffff; \
         border: 1px solid rgba(255, 255, 255, 0.2); \
         border-radius: 6px; \
         transition: all 0.2s ease; \
         display: flex; \
         align-items: center; \
         justify-content: center;").ok();

    // Create root note display
    let Ok(root_note_display) = document.create_element("span") else {
        dev_log!("Failed to create root note display");
        return;
    };
    root_note_display.set_id("root-note-display");
    root_note_display.set_text_content(Some("A3")); // Default root note is 57 (A3)
    root_note_display.set_attribute("style", 
        "color: #ffffff; \
         font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', monospace; \
         font-size: 16px; \
         font-weight: 600; \
         min-width: 48px; \
         text-align: center; \
         background: rgba(255, 255, 255, 0.1); \
         padding: 6px 12px; \
         border-radius: 6px; \
         border: 1px solid rgba(255, 255, 255, 0.15);").ok();

    // Create plus button
    let Ok(plus_button) = document.create_element("button") else {
        dev_log!("Failed to create plus button");
        return;
    };
    plus_button.set_id("root-note-plus");
    plus_button.set_text_content(Some("+"));
    plus_button.set_attribute("style", 
        "width: 32px; \
         height: 32px; \
         font-size: 18px; \
         font-weight: bold; \
         cursor: pointer; \
         background: linear-gradient(135deg, #4a4a4a, #3a3a3a); \
         color: #ffffff; \
         border: 1px solid rgba(255, 255, 255, 0.2); \
         border-radius: 6px; \
         transition: all 0.2s ease; \
         display: flex; \
         align-items: center; \
         justify-content: center;").ok();

    // Assemble root note controls
    root_note_container.append_child(&root_note_label).ok();
    root_note_container.append_child(&minus_button).ok();
    root_note_container.append_child(&root_note_display).ok();
    root_note_container.append_child(&plus_button).ok();

    // Create tuning system controls container
    let Ok(tuning_container) = document.create_element("div") else {
        dev_log!("Failed to create tuning container");
        return;
    };
    tuning_container.set_attribute("style", "display: flex; align-items: center; gap: 10px;").ok();

    // Create tuning system label
    let Ok(tuning_label) = document.create_element("span") else {
        dev_log!("Failed to create tuning label");
        return;
    };
    tuning_label.set_text_content(Some("Tuning:"));
    tuning_label.set_attribute("style", 
        "color: #ffffff; \
         font-family: inherit; \
         font-size: 14px; \
         font-weight: 500; \
         margin-right: 4px;").ok();

    // Create tuning system dropdown
    let Ok(tuning_select) = document.create_element("select") else {
        dev_log!("Failed to create tuning select");
        return;
    };
    tuning_select.set_id("tuning-system-select");
    tuning_select.set_attribute("style", 
        "padding: 8px 12px; \
         background: linear-gradient(135deg, #4a4a4a, #3a3a3a); \
         color: #ffffff; \
         border: 1px solid rgba(255, 255, 255, 0.2); \
         border-radius: 6px; \
         cursor: pointer; \
         font-size: 14px; \
         font-family: inherit; \
         min-width: 160px; \
         transition: all 0.2s ease;").ok();

    // Create Equal Temperament option
    let Ok(equal_option) = document.create_element("option") else {
        dev_log!("Failed to create equal temperament option");
        return;
    };
    equal_option.set_attribute("value", "equal").ok();
    equal_option.set_text_content(Some("Equal Temperament"));

    // Create Just Intonation option
    let Ok(just_option) = document.create_element("option") else {
        dev_log!("Failed to create just intonation option");
        return;
    };
    just_option.set_attribute("value", "just").ok();
    just_option.set_text_content(Some("Just Intonation"));

    // Assemble tuning system dropdown
    tuning_select.append_child(&equal_option).ok();
    tuning_select.append_child(&just_option).ok();

    // Assemble tuning controls
    tuning_container.append_child(&tuning_label).ok();
    tuning_container.append_child(&tuning_select).ok();

    // Assemble main container
    container.append_child(&root_note_container).ok();
    container.append_child(&tuning_container).ok();

    // Append to body
    let Some(body) = document.body() else {
        dev_log!("Failed to get document body");
        return;
    };

    if let Err(err) = body.append_child(&container) {
        dev_log!("Failed to append UI container to body: {:?}", err);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn cleanup_main_scene_ui() {
    let Some(window) = window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    // Remove the main container
    if let Some(container) = document.get_element_by_id("main-scene-ui-container") {
        container.remove();
    }
}

#[cfg(target_arch = "wasm32")]
pub fn setup_event_listeners(presenter: Rc<RefCell<crate::presentation::Presenter>>) {
    let Some(window) = window() else {
        dev_log!("Failed to get window for event listeners");
        return;
    };

    let Some(document) = window.document() else {
        dev_log!("Failed to get document for event listeners");
        return;
    };

    // Set up plus button event listener
    if let Some(plus_button) = document.get_element_by_id("root-note-plus") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Ok(presenter_ref) = presenter_clone.try_borrow() {
                let current_root = presenter_ref.get_root_note();
                drop(presenter_ref); // Release the borrow
                
                if let Some(new_root) = increment_midi_note(current_root) {
                    if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                        presenter_mut.on_root_note_adjusted(new_root);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = plus_button.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add click listener to plus button: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find root-note-plus button");
    }

    // Set up minus button event listener
    if let Some(minus_button) = document.get_element_by_id("root-note-minus") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Ok(presenter_ref) = presenter_clone.try_borrow() {
                let current_root = presenter_ref.get_root_note();
                drop(presenter_ref); // Release the borrow
                
                if let Some(new_root) = decrement_midi_note(current_root) {
                    if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                        presenter_mut.on_root_note_adjusted(new_root);
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = minus_button.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add click listener to minus button: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find root-note-minus button");
    }

    // Set up tuning system dropdown event listener
    if let Some(tuning_select) = document.get_element_by_id("tuning-system-select") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(select_element) = document.get_element_by_id("tuning-system-select") {
                if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
                    let value = html_select.value();
                    let tuning_system = match value.as_str() {
                        "equal" => TuningSystem::EqualTemperament,
                        "just" => TuningSystem::JustIntonation,
                        _ => {
                            dev_log!("Unknown tuning system value: {}", value);
                            return;
                        }
                    };
                    presenter_clone.borrow_mut().on_tuning_system_changed(tuning_system);
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = tuning_select.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add change listener to tuning dropdown: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find tuning-system-select dropdown");
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_main_scene_ui() {
    // No-op for non-WASM targets
}

#[cfg(not(target_arch = "wasm32"))]
pub fn cleanup_main_scene_ui() {
    // No-op for non-WASM targets
}

/// Synchronize UI elements with current presenter state
/// 
/// This function updates the UI to reflect the current root note and tuning system
/// values from the presenter, ensuring the UI stays in sync when values change
/// from sources other than direct user interaction.
/// 
/// # Arguments
/// 
/// * `root_note` - The current root note from the presenter
/// * `tuning_system` - The current tuning system from the presenter
#[cfg(target_arch = "wasm32")]
pub fn sync_ui_with_presenter_state(root_note: MidiNote, tuning_system: TuningSystem) {
    let Some(window) = window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    // Update root note display
    if let Some(display) = document.get_element_by_id("root-note-display") {
        let formatted_note = format_midi_note(root_note);
        display.set_text_content(Some(&formatted_note));
    }

    // Update tuning system dropdown selection
    if let Some(select_element) = document.get_element_by_id("tuning-system-select") {
        if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
            let value = match tuning_system {
                TuningSystem::EqualTemperament => "equal",
                TuningSystem::JustIntonation => "just",
            };
            html_select.set_value(value);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_event_listeners(_presenter: std::rc::Rc<std::cell::RefCell<crate::presentation::Presenter>>) {
    // No-op for non-WASM targets
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_ui_with_presenter_state(_root_note: crate::shared_types::MidiNote, _tuning_system: crate::shared_types::TuningSystem) {
    // No-op for non-WASM targets
}