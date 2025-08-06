#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, Document, Element, HtmlElement, HtmlSelectElement, HtmlInputElement, EventTarget};

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(target_arch = "wasm32")]
use crate::common::dev_log;
#[cfg(target_arch = "wasm32")]
use crate::shared_types::{TuningSystem, MidiNote, Scale, increment_midi_note, decrement_midi_note};
#[cfg(target_arch = "wasm32")]
use crate::app_config::COLOR_SCHEME;
#[cfg(target_arch = "wasm32")]
use crate::web::utils::{rgba_to_css, rgb_to_css};

// Global state for current root note - initialized to A3 (57)
#[cfg(target_arch = "wasm32")]
static CURRENT_ROOT_NOTE: AtomicU8 = AtomicU8::new(57);

// Global state for root note audio enabled
#[cfg(target_arch = "wasm32")]
static CURRENT_ROOT_NOTE_AUDIO_ENABLED: AtomicU8 = AtomicU8::new(0); // 0 = false, 1 = true

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

    // Create style constants using color scheme
    let label_style = format!(
        "color: {}; font-family: inherit; font-size: 14px; font-weight: 500;",
        rgb_to_css(COLOR_SCHEME.text)
    );
    
    let button_style = format!(
        "padding: 6px 12px; font-size: 14px; font-weight: bold; cursor: pointer; \
         background: linear-gradient(135deg, {}, {}); color: {}; \
         border: 1px solid {}; border-radius: 6px; transition: all 0.2s ease;",
        rgb_to_css(COLOR_SCHEME.surface),
        rgba_to_css(COLOR_SCHEME.surface, 0.8),
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.text, 0.2)
    );
    
    let select_style = format!(
        "padding: 8px 12px; background: linear-gradient(135deg, {}, {}); \
         color: {}; border: 1px solid {}; border-radius: 6px; cursor: pointer;",
        rgb_to_css(COLOR_SCHEME.surface),
        rgba_to_css(COLOR_SCHEME.surface, 0.8),
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.text, 0.2)
    );
    
    let display_style = format!(
        "color: {}; font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', monospace; \
         font-size: 16px; font-weight: 600; background: {}; padding: 8px 12px; \
         border: 1px solid {}; border-radius: 4px;",
        rgb_to_css(COLOR_SCHEME.text),
        rgba_to_css(COLOR_SCHEME.text, 0.1),
        rgba_to_css(COLOR_SCHEME.text, 0.15)
    );

    // Create container div
    let Ok(container) = document.create_element("div") else {
        dev_log!("Failed to create container div");
        return;
    };
    
    container.set_id("main-scene-ui-container");
    
    container.set_attribute("style", 
        "display: flex; \
         gap: 24px; \
         align-items: center; \
         font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;").ok();

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
    let label_style = format!(
        "color: {}; \
         font-family: inherit; \
         font-size: 14px; \
         font-weight: 500; \
         margin-right: 4px;",
        rgb_to_css(COLOR_SCHEME.text)
    );
    root_note_label.set_attribute("style", &label_style).ok();

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

    // Create scale controls container
    let Ok(scale_container) = document.create_element("div") else {
        dev_log!("Failed to create scale container");
        return;
    };
    scale_container.set_attribute("style", "display: flex; align-items: center; gap: 10px;").ok();

    // Create scale label
    let Ok(scale_label) = document.create_element("span") else {
        dev_log!("Failed to create scale label");
        return;
    };
    scale_label.set_text_content(Some("Scale:"));
    scale_label.set_attribute("style", 
        "color: #ffffff; \
         font-family: inherit; \
         font-size: 14px; \
         font-weight: 500; \
         margin-right: 4px;").ok();

    // Create scale dropdown
    let Ok(scale_select) = document.create_element("select") else {
        dev_log!("Failed to create scale select");
        return;
    };
    scale_select.set_id("scale-select");
    scale_select.set_attribute("style", 
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

    // Create Chromatic option (first in enum, so default)
    let Ok(chromatic_option) = document.create_element("option") else {
        dev_log!("Failed to create chromatic option");
        return;
    };
    chromatic_option.set_attribute("value", "chromatic").ok();
    chromatic_option.set_attribute("selected", "true").ok();
    chromatic_option.set_text_content(Some("Chromatic"));

    // Create Major option
    let Ok(major_option) = document.create_element("option") else {
        dev_log!("Failed to create major option");
        return;
    };
    major_option.set_attribute("value", "major").ok();
    major_option.set_text_content(Some("Major"));

    // Create Minor option
    let Ok(minor_option) = document.create_element("option") else {
        dev_log!("Failed to create minor option");
        return;
    };
    minor_option.set_attribute("value", "minor").ok();
    minor_option.set_text_content(Some("Minor"));

    // Create Major Pentatonic option
    let Ok(major_pentatonic_option) = document.create_element("option") else {
        dev_log!("Failed to create major pentatonic option");
        return;
    };
    major_pentatonic_option.set_attribute("value", "major_pentatonic").ok();
    major_pentatonic_option.set_text_content(Some("Major Pentatonic"));

    // Create Minor Pentatonic option
    let Ok(minor_pentatonic_option) = document.create_element("option") else {
        dev_log!("Failed to create minor pentatonic option");
        return;
    };
    minor_pentatonic_option.set_attribute("value", "minor_pentatonic").ok();
    minor_pentatonic_option.set_text_content(Some("Minor Pentatonic"));

    // Assemble scale dropdown (order matches enum)
    scale_select.append_child(&chromatic_option).ok();
    scale_select.append_child(&major_option).ok();
    scale_select.append_child(&minor_option).ok();
    scale_select.append_child(&major_pentatonic_option).ok();
    scale_select.append_child(&minor_pentatonic_option).ok();

    // Assemble scale controls
    scale_container.append_child(&scale_label).ok();
    scale_container.append_child(&scale_select).ok();

    // Assemble main container
    container.append_child(&root_note_container).ok();
    container.append_child(&tuning_container).ok();
    container.append_child(&scale_container).ok();

    // Create root note audio controls
    #[cfg(target_arch = "wasm32")]
    {
        // Create root note audio container
        let Ok(root_note_audio_container) = document.create_element("div") else {
            dev_log!("Failed to create root note audio container");
            return;
        };
        root_note_audio_container.set_attribute("style", "display: flex; align-items: center; gap: 10px;").ok();

        // Create root note audio label
        let Ok(root_note_audio_label) = document.create_element("span") else {
            dev_log!("Failed to create root note audio label");
            return;
        };
        root_note_audio_label.set_text_content(Some("Root Note Audio:"));
        root_note_audio_label.set_attribute("style", 
            "color: #ffffff; \
             font-family: inherit; \
             font-size: 14px; \
             font-weight: 500; \
             margin-right: 4px;").ok();

        // Create root note audio checkbox
        let Ok(root_note_audio_checkbox) = document.create_element("input") else {
            dev_log!("Failed to create root note audio checkbox");
            return;
        };
        let root_note_audio_checkbox = root_note_audio_checkbox.dyn_into::<HtmlInputElement>().unwrap();
        root_note_audio_checkbox.set_type("checkbox");
        root_note_audio_checkbox.set_id("root-note-audio-checkbox");
        root_note_audio_checkbox.set_attribute("style", 
            "width: 20px; \
             height: 20px; \
             cursor: pointer; \
             background: linear-gradient(135deg, #4a4a4a, #3a3a3a); \
             color: #ffffff; \
             border: 1px solid rgba(255, 255, 255, 0.2); \
             border-radius: 4px; \
             transition: all 0.2s ease;").ok();
        
        // Checkbox will be synced with engine state via sync_ui_with_model_data

        // Assemble root note audio controls
        root_note_audio_container.append_child(&root_note_audio_label).ok();
        root_note_audio_container.append_child(&root_note_audio_checkbox).ok();

        // Append to main container
        container.append_child(&root_note_audio_container).ok();
    }

    // Append container to menu bar
    let menu_bar = document.get_element_by_id("menu-bar");
    
    if let Some(menu_bar) = menu_bar {
        if let Err(err) = menu_bar.append_child(&container) {
            dev_log!("Failed to append UI container to menu bar: {:?}", err);
        }
    } else {
        // Fallback to body if menu-bar doesn't exist
        if let Some(body) = document.body() {
            if let Err(err) = body.append_child(&container) {
                dev_log!("Failed to append UI container to body: {:?}", err);
            }
        }
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
            let current_root_note = CURRENT_ROOT_NOTE.load(Ordering::Relaxed);
            if let Some(new_root) = increment_midi_note(current_root_note) {
                if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                    presenter_mut.on_root_note_adjusted(new_root);
                    
                    // Also update root note audio frequency if it's currently enabled
                    if let Some(current_window) = web_sys::window() {
                        if let Some(document) = current_window.document() {
                            if let Some(checkbox_element) = document.get_element_by_id("root-note-audio-checkbox") {
                                if let Some(html_checkbox) = checkbox_element.dyn_ref::<HtmlInputElement>() {
                                    if html_checkbox.checked() {
                                        presenter_mut.on_root_note_audio_configured(true, new_root);
                                    }
                                }
                            }
                        }
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
            let current_root_note = CURRENT_ROOT_NOTE.load(Ordering::Relaxed);
            if let Some(new_root) = decrement_midi_note(current_root_note) {
                if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                    presenter_mut.on_root_note_adjusted(new_root);
                    
                    // Also update root note audio frequency if it's currently enabled
                    if let Some(current_window) = web_sys::window() {
                        if let Some(document) = current_window.document() {
                            if let Some(checkbox_element) = document.get_element_by_id("root-note-audio-checkbox") {
                                if let Some(html_checkbox) = checkbox_element.dyn_ref::<HtmlInputElement>() {
                                    if html_checkbox.checked() {
                                        presenter_mut.on_root_note_audio_configured(true, new_root);
                                    }
                                }
                            }
                        }
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
            if let Some(current_window) = web_sys::window() {
                if let Some(document) = current_window.document() {
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

    // Set up scale dropdown event listener
    if let Some(scale_select) = document.get_element_by_id("scale-select") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(current_window) = web_sys::window() {
                if let Some(document) = current_window.document() {
                    if let Some(select_element) = document.get_element_by_id("scale-select") {
                if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
                    let value = html_select.value();
                    let scale = match value.as_str() {
                        "chromatic" => Scale::Chromatic,
                        "major" => Scale::Major,
                        "minor" => Scale::Minor,
                        "major_pentatonic" => Scale::MajorPentatonic,
                        "minor_pentatonic" => Scale::MinorPentatonic,
                        _ => {
                            dev_log!("Unknown scale value: {}", value);
                            return;
                        }
                    };
                        presenter_clone.borrow_mut().on_scale_changed(scale);
                    }
                }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = scale_select.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add change listener to scale dropdown: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find scale-select dropdown");
    }

    // Set up root note audio checkbox event listener
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(checkbox) = document.get_element_by_id("root-note-audio-checkbox") {
            let presenter_clone = presenter.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                if let Some(current_window) = web_sys::window() {
                    if let Some(document) = current_window.document() {
                        if let Some(checkbox_element) = document.get_element_by_id("root-note-audio-checkbox") {
                            if let Some(html_checkbox) = checkbox_element.dyn_ref::<HtmlInputElement>() {
                                let enabled = html_checkbox.checked();
                                let current_root_note = CURRENT_ROOT_NOTE.load(Ordering::Relaxed);
                                presenter_clone.borrow_mut().on_root_note_audio_configured(enabled, current_root_note);
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);

            if let Some(event_target) = checkbox.dyn_ref::<EventTarget>() {
                if let Err(err) = event_target.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref()) {
                    dev_log!("Failed to add change listener to root note audio checkbox: {:?}", err);
                }
            }
            closure.forget();
        } else {
            dev_log!("Failed to find root-note-audio-checkbox");
        }
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
/// * `scale` - The current scale from the presenter
/// * `root_note_audio_enabled` - The current root note audio state
#[cfg(target_arch = "wasm32")]
pub fn sync_ui_with_presenter_state(model_data: &crate::shared_types::ModelUpdateResult) {
    let Some(window) = window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    // Update stored root note state
    CURRENT_ROOT_NOTE.store(model_data.root_note, Ordering::Relaxed);

    // Update root note display
    if let Some(display) = document.get_element_by_id("root-note-display") {
        let formatted_note = format_midi_note(model_data.root_note);
        display.set_text_content(Some(&formatted_note));
    }

    // Update tuning system dropdown selection
    if let Some(select_element) = document.get_element_by_id("tuning-system-select") {
        if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
            let value = match model_data.tuning_system {
                TuningSystem::EqualTemperament => "equal",
                TuningSystem::JustIntonation => "just",
            };
            html_select.set_value(value);
        }
    }

    // Update scale dropdown selection
    if let Some(select_element) = document.get_element_by_id("scale-select") {
        if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
            let value = match model_data.scale {
                Scale::Chromatic => "chromatic",
                Scale::Major => "major",
                Scale::Minor => "minor",
                Scale::MajorPentatonic => "major_pentatonic",
                Scale::MinorPentatonic => "minor_pentatonic",
            };
            html_select.set_value(value);
        }
    }

    // Update root note audio checkbox state
    CURRENT_ROOT_NOTE_AUDIO_ENABLED.store(if model_data.root_note_audio_enabled { 1 } else { 0 }, Ordering::Relaxed);
    
    if let Some(checkbox_element) = document.get_element_by_id("root-note-audio-checkbox") {
        if let Some(html_checkbox) = checkbox_element.dyn_ref::<HtmlInputElement>() {
            html_checkbox.set_checked(model_data.root_note_audio_enabled);
        }
    }
}


#[cfg(not(target_arch = "wasm32"))]
pub fn setup_event_listeners(_presenter: std::rc::Rc<std::cell::RefCell<crate::presentation::Presenter>>) {
    // No-op for non-WASM targets
}

#[cfg(not(target_arch = "wasm32"))]
pub fn sync_ui_with_presenter_state(_model_data: &crate::shared_types::ModelUpdateResult) {
    // No-op for non-WASM targets
}