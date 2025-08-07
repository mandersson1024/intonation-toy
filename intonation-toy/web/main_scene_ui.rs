#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::{window, Document, HtmlElement, HtmlSelectElement, EventTarget};

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
use crate::web::styling;

// Global state for current root note - initialized to A3 (57)
#[cfg(target_arch = "wasm32")]
static CURRENT_ROOT_NOTE: AtomicU8 = AtomicU8::new(57);

// Global state for root note audio enabled
#[cfg(target_arch = "wasm32")]
static CURRENT_ROOT_NOTE_AUDIO_ENABLED: AtomicU8 = AtomicU8::new(0); // 0 = false, 1 = true

// Debouncing for tuning fork clicks to prevent double-triggering
#[cfg(target_arch = "wasm32")]
static LAST_TUNING_FORK_CLICK_TIME: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Format a MIDI note number as a string (e.g., 60 -> "C4")
#[cfg(target_arch = "wasm32")]
fn format_midi_note(midi_note: MidiNote) -> String {
    crate::shared_types::midi_note_to_name(midi_note)
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

    // No need for local style constants - will use centralized styling functions

    // Create header for sidebar
    let Ok(header) = document.create_element("h1") else {
        dev_log!("Failed to create header");
        return;
    };
    header.set_text_content(Some("Intonation Toy"));
    header.set_attribute("style", &styling::get_sidebar_header_style()).ok();

    // Create container div
    let Ok(container) = document.create_element("div") else {
        dev_log!("Failed to create container div");
        return;
    };
    
    container.set_id("main-scene-ui-container");
    
    container.set_attribute("style", &styling::get_container_style()).ok();

    // Create root note controls container
    let Ok(root_note_container) = document.create_element("div") else {
        dev_log!("Failed to create root note container");
        return;
    };
    root_note_container.set_attribute("style", "display: flex; align-items: center; gap: 8px;").ok();

    // Create root note label
    let Ok(root_note_label) = document.create_element("span") else {
        dev_log!("Failed to create root note label");
        return;
    };
    root_note_label.set_text_content(Some("Root Note:"));
    root_note_label.set_attribute("style", &styling::get_label_style()).ok();

    // Create minus button
    let Ok(minus_button) = document.create_element("button") else {
        dev_log!("Failed to create minus button");
        return;
    };
    minus_button.set_id("root-note-minus");
    minus_button.set_text_content(Some("-"));
    minus_button.set_attribute("style", &styling::get_small_button_style()).ok();

    // Create root note display
    let Ok(root_note_display) = document.create_element("span") else {
        dev_log!("Failed to create root note display");
        return;
    };
    root_note_display.set_id("root-note-display");
    root_note_display.set_text_content(Some("A3")); // Default root note is 57 (A3)
    root_note_display.set_attribute("style", &styling::get_root_note_display_style()).ok();

    // Create plus button
    let Ok(plus_button) = document.create_element("button") else {
        dev_log!("Failed to create plus button");
        return;
    };
    plus_button.set_id("root-note-plus");
    plus_button.set_text_content(Some("+"));
    plus_button.set_attribute("style", &styling::get_small_button_style()).ok();

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
    tuning_container.set_attribute("style", "display: flex; align-items: center; gap: 8px;").ok();

    // Create tuning system label
    let Ok(tuning_label) = document.create_element("span") else {
        dev_log!("Failed to create tuning label");
        return;
    };
    tuning_label.set_text_content(Some("Tuning:"));
    tuning_label.set_attribute("style", &styling::get_label_style()).ok();

    // Create tuning system dropdown
    let Ok(tuning_select) = document.create_element("select") else {
        dev_log!("Failed to create tuning select");
        return;
    };
    tuning_select.set_id("tuning-system-select");
    tuning_select.set_attribute("style", &styling::get_select_style()).ok();

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
    scale_container.set_attribute("style", "display: flex; align-items: center; gap: 8px;").ok();

    // Create scale label
    let Ok(scale_label) = document.create_element("span") else {
        dev_log!("Failed to create scale label");
        return;
    };
    scale_label.set_text_content(Some("Scale:"));
    scale_label.set_attribute("style", &styling::get_label_style()).ok();

    // Create scale dropdown
    let Ok(scale_select) = document.create_element("select") else {
        dev_log!("Failed to create scale select");
        return;
    };
    scale_select.set_id("scale-select");
    scale_select.set_attribute("style", &styling::get_select_style()).ok();

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
        root_note_audio_container.set_attribute("style", "display: flex; align-items: center; gap: 8px;").ok();

        // Create root note audio label
        let Ok(root_note_audio_label) = document.create_element("span") else {
            dev_log!("Failed to create root note audio label");
            return;
        };
        root_note_audio_label.set_text_content(Some("Root Note Audio:"));
        root_note_audio_label.set_attribute("style", &styling::get_label_style()).ok();

        // Create tuning fork icon
        let Ok(tuning_fork_icon) = document.create_element("div") else {
            dev_log!("Failed to create tuning fork icon");
            return;
        };
        tuning_fork_icon.set_id("root-note-audio-icon");
        tuning_fork_icon.set_class_name("tuning-fork-icon");
        
        let tuning_fork_svg = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 2v16"/>
                <path d="M8 2h8"/>
                <path d="M8 2v4c0 1.1.9 2 2 2h4c1.1 0 2-.9 2-2V2"/>
                <path d="M6 18c-2 0-2 2-2 2s0 2 2 2 2-2 2-2-0-2-2-2"/>
                <path d="M18 18c2 0 2 2 2 2s0 2-2 2-2-2-2-2 0-2 2-2"/>
            </svg>"#;
        tuning_fork_icon.set_inner_html(&tuning_fork_svg);
        
        // Icon will be synced with engine state via sync_ui_with_model_data

        // Assemble root note audio controls
        root_note_audio_container.append_child(&root_note_audio_label).ok();
        root_note_audio_container.append_child(&tuning_fork_icon).ok();

        // Append to main container
        container.append_child(&root_note_audio_container).ok();
    }

    // Append header and container to sidebar
    let sidebar = document.get_element_by_id("sidebar");
    
    if let Some(sidebar) = sidebar {
        // Append header first
        if let Err(err) = sidebar.append_child(&header) {
            dev_log!("Failed to append header to sidebar: {:?}", err);
        }
        // Then append the main container
        if let Err(err) = sidebar.append_child(&container) {
            dev_log!("Failed to append UI container to sidebar: {:?}", err);
        }
    } else {
        // Fallback to body if sidebar doesn't exist
        if let Some(body) = document.body() {
            if let Err(err) = body.append_child(&header) {
                dev_log!("Failed to append header to body: {:?}", err);
            }
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
                            if let Some(icon_element) = document.get_element_by_id("root-note-audio-icon") {
                                if let Some(html_element) = icon_element.dyn_ref::<HtmlElement>() {
                                    if html_element.class_name().contains("active") {
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
                            if let Some(icon_element) = document.get_element_by_id("root-note-audio-icon") {
                                if let Some(html_element) = icon_element.dyn_ref::<HtmlElement>() {
                                    if html_element.class_name().contains("active") {
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

    // Set up root note audio tuning fork icon event listener
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(icon) = document.get_element_by_id("root-note-audio-icon") {
            let presenter_clone = presenter.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
                event.prevent_default();
                event.stop_propagation();
                
                // Debounce to prevent double-clicks within 200ms
                let now = js_sys::Date::now() as u64;
                let last_click = LAST_TUNING_FORK_CLICK_TIME.load(Ordering::Relaxed);
                if now - last_click < 200 {
                    return;
                }
                LAST_TUNING_FORK_CLICK_TIME.store(now, Ordering::Relaxed);
                
                if let Some(current_window) = web_sys::window() {
                    if let Some(document) = current_window.document() {
                        if let Some(icon_element) = document.get_element_by_id("root-note-audio-icon") {
                            if let Some(html_element) = icon_element.dyn_ref::<HtmlElement>() {
                                // Toggle the active class
                                let current_classes = html_element.class_name();
                                let enabled = current_classes.contains("active");
                                if enabled {
                                    let new_classes = current_classes.replace(" active", "").replace("active ", "").replace("active", "");
                                    html_element.set_class_name(&new_classes);
                                } else {
                                    html_element.set_class_name(&format!("{} active", current_classes));
                                }
                                let new_enabled = !enabled;
                                let current_root_note = CURRENT_ROOT_NOTE.load(Ordering::Relaxed);
                                presenter_clone.borrow_mut().on_root_note_audio_configured(new_enabled, current_root_note);
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(_)>);

            if let Some(event_target) = icon.dyn_ref::<EventTarget>() {
                if let Err(err) = event_target.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref()) {
                    dev_log!("Failed to add click listener to root note audio icon: {:?}", err);
                }
            }
            closure.forget();
        } else {
            dev_log!("Failed to find root-note-audio-icon");
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

    // Update root note audio icon state
    CURRENT_ROOT_NOTE_AUDIO_ENABLED.store(if model_data.root_note_audio_enabled { 1 } else { 0 }, Ordering::Relaxed);
    
    if let Some(icon_element) = document.get_element_by_id("root-note-audio-icon") {
        if let Some(html_element) = icon_element.dyn_ref::<HtmlElement>() {
            let current_classes = html_element.class_name();
            if model_data.root_note_audio_enabled {
                if !current_classes.contains("active") {
                    html_element.set_class_name(&format!("{} active", current_classes));
                }
            } else {
                let new_classes = current_classes.replace(" active", "").replace("active ", "").replace("active", "");
                html_element.set_class_name(&new_classes);
            }
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