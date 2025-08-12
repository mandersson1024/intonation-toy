//! UI controller implementation for web browsers.
//! 
//! Provides canvas management, theme application, UI lifecycle management,
//! event handling, and other UI-related functionality using DOM APIs.
//! 
//! This implementation consolidates UI management functions previously
//! scattered throughout the web module into a single platform abstraction.

use crate::platform::traits::UiController;
use super::utils::rgb_to_css;
use wasm_bindgen::JsCast;
use crate::common::dev_log;
use wasm_bindgen::closure::Closure;
use web_sys::{HtmlSelectElement, HtmlInputElement, EventTarget};
use std::sync::atomic::{AtomicU8, AtomicU16, Ordering};


/// Web-based UI controller implementation.
/// 
/// Manages canvas sizing, theme styling, and other UI operations
/// through browser DOM APIs.
pub struct WebUiController;

impl UiController for WebUiController {
    fn resize_canvas() {
        resize_canvas_impl();
    }

    fn apply_theme_styles() {
        reapply_current_theme();
    }
    
    fn setup_ui() {
        setup_ui_impl();
    }
    
    fn cleanup_ui() {
        cleanup_ui_impl();
    }
    
    fn setup_event_listeners(presenter: std::rc::Rc<std::cell::RefCell<crate::presentation::Presenter>>) {
        setup_event_listeners_impl(presenter);
    }
    
    fn sync_ui_state(model_data: &crate::shared_types::ModelUpdateResult) {
        sync_ui_state_impl(model_data);
    }
    
    fn get_zoom_factor() -> f32 {
        get_zoom_factor_impl()
    }
}

impl WebUiController {
    /// Creates a new WebUiController instance.
    pub fn new() -> Self {
        WebUiController
    }
}

impl Default for WebUiController {
    fn default() -> Self {
        Self::new()
    }
}

/// Resize the 3D canvas to fit the available window space.
/// 
/// This function calculates the optimal canvas size based on:
/// - Window dimensions
/// - Sidebar width
/// - Canvas margins
/// - Zoom control space
/// 
/// The canvas maintains a square aspect ratio and respects minimum/maximum size constraints.
fn resize_canvas_impl() {
    use crate::common::dev_log;
    
    let window_obj = match web_sys::window() {
        Some(win) => win,
        None => {
            dev_log!("RESIZE: No window object available");
            return;
        }
    };
    
    let document = match window_obj.document() {
        Some(doc) => doc,
        None => {
            dev_log!("RESIZE: No document object available");
            return;
        }
    };
    
    // Get the canvas element
    let canvas = match document.get_element_by_id("three-d-canvas") {
        Some(elem) => match elem.dyn_into::<web_sys::HtmlCanvasElement>() {
            Ok(canvas) => canvas,
            Err(_) => {
                dev_log!("RESIZE: Element with id 'three-d-canvas' is not a canvas");
                return;
            }
        },
        None => {
            dev_log!("RESIZE: Canvas element 'three-d-canvas' not found");
            return;
        }
    };
    
    dev_log!("RESIZE: resize_canvas called");
    
    // Import constants from web styling module
    // TODO: These should be moved to platform configuration in future phases
    const SIDEBAR_WIDTH: i32 = 300;
    const CANVAS_MARGIN: i32 = 100;
    
    // Estimate zoom control width (padding + slider + margins)
    let zoom_control_width = 80; // Approximate width of zoom control
    let gap = 16; // Gap between canvas and zoom control
    
    // Calculate available space (subtract sidebar width, margins, zoom control, and gap)
    let available_width = window_obj.inner_width().unwrap().as_f64().unwrap() as i32 - SIDEBAR_WIDTH - (CANVAS_MARGIN * 2) - zoom_control_width - gap;
    let available_height = window_obj.inner_height().unwrap().as_f64().unwrap() as i32 - (CANVAS_MARGIN * 2);
    
    dev_log!("RESIZE: available {}x{}", available_width, available_height);
    
    // Take the smaller dimension to maintain square aspect ratio
    let mut canvas_size = std::cmp::min(available_width, available_height);
    canvas_size = std::cmp::min(canvas_size, crate::app_config::CANVAS_MAX_SIZE);
    canvas_size = std::cmp::max(canvas_size, crate::app_config::CANVAS_MIN_SIZE);
    
    // Scene wrapper width includes canvas + gap + zoom control
    let wrapper_width = canvas_size + gap + zoom_control_width;
    let wrapper_height = canvas_size;
    
    dev_log!("RESIZE: setting canvas size to {}px, wrapper size to {}x{}", canvas_size, wrapper_width, wrapper_height);
    
    // Get the scene wrapper element
    let scene_wrapper = match document.get_element_by_id("scene-wrapper") {
        Some(elem) => elem,
        None => {
            dev_log!("RESIZE: Scene wrapper element not found");
            return;
        }
    };
    
    // Set CSS positioning and sizing for scene wrapper
    if let Err(e) = scene_wrapper.set_attribute("style", &format!(
        "position: absolute; top: {}px; left: {}px; width: {}px; height: {}px; display: flex; flex-direction: row; align-items: center; gap: 16px;",
        CANVAS_MARGIN, CANVAS_MARGIN, wrapper_width, wrapper_height
    )) {
        dev_log!("RESIZE: Failed to set scene wrapper style: {:?}", e);
        return;
    }
    
    // Set canvas to specific size
    let style = canvas.style();
    if let Err(e) = style.set_property("width", &format!("{}px", canvas_size)) {
        dev_log!("RESIZE: Failed to set canvas width: {:?}", e);
    }
    if let Err(e) = style.set_property("height", &format!("{}px", canvas_size)) {
        dev_log!("RESIZE: Failed to set canvas height: {:?}", e);
    }
}

/// Reapply the current theme by updating CSS custom properties.
/// 
/// All styling is handled by static/style.css with CSS custom properties,
/// so only updating the CSS variables is needed for efficient theme switching.
/// This approach ensures theme changes are applied instantly across all UI elements.
fn reapply_current_theme() {
    update_css_variables();
}

/// Update CSS custom properties based on the current color scheme.
/// 
/// Sets CSS variables on the document root element that control theming
/// throughout the application. Variables include:
/// - --color-background
/// - --color-surface
/// - --color-primary
/// - --color-secondary
/// - --color-accent
/// - --color-text
/// - --color-muted
/// - --color-border
/// - --color-error
fn update_css_variables() {
    use crate::theme::get_current_color_scheme;
    use crate::common::dev_log;
    
    let color_scheme = get_current_color_scheme();
    
    // Apply to document.documentElement (html element) instead of :root selector
    let document = match web_sys::window() {
        Some(win) => match win.document() {
            Some(doc) => doc,
            None => {
                dev_log!("No document available for theme update");
                return;
            }
        },
        None => {
            dev_log!("No window available for theme update");
            return;
        }
    };
    
    if let Some(root) = document.document_element() {
        // Use set_attribute to set style properties directly on the element
        let style_str = format!(
            "--color-background: {}; --color-surface: {}; --color-primary: {}; --color-secondary: {}; --color-accent: {}; --color-text: {}; --color-muted: {}; --color-border: {}; --color-error: {};",
            rgb_to_css(color_scheme.background),
            rgb_to_css(color_scheme.surface),
            rgb_to_css(color_scheme.primary),
            rgb_to_css(color_scheme.secondary),
            rgb_to_css(color_scheme.accent),
            rgb_to_css(color_scheme.text),
            rgb_to_css(color_scheme.muted),
            rgb_to_css(color_scheme.border),
            rgb_to_css(color_scheme.error)
        );
        
        if root.set_attribute("style", &style_str).is_err() {
            dev_log!("Failed to set CSS variables on root element");
        } else {
            dev_log!("Successfully updated CSS custom properties");
        }
    }
}

// Static variables for UI state management (moved from web/main_scene_ui.rs)
static CURRENT_ROOT_NOTE: AtomicU8 = AtomicU8::new(crate::app_config::DEFAULT_ROOT_NOTE);
static CURRENT_TUNING_FORK_VOLUME_POSITION: AtomicU8 = AtomicU8::new(0);
static CURRENT_ZOOM_LEVEL: AtomicU16 = AtomicU16::new(0);

/// Sets up the main scene UI elements and event handlers
/// 
/// This function initializes UI components required for the main scene,
/// including setting default values and verifying essential HTML elements exist.
fn setup_ui_impl() {
    let Some(window) = web_sys::window() else {
        dev_log!("Failed to get window");
        return;
    };

    let Some(document) = window.document() else {
        dev_log!("Failed to get document");
        return;
    };

    // Verify that essential HTML elements exist and set initial values
    if let Some(root_note_display) = document.get_element_by_id("root-note-display") {
        let default_note_name = crate::shared_types::midi_note_to_name(crate::app_config::DEFAULT_ROOT_NOTE);
        root_note_display.set_text_content(Some(&default_note_name));
    } else {
        dev_log!("Warning: root-note-display element not found in HTML");
    }

    if let Some(volume_display) = document.get_element_by_id("tuning-fork-volume-display") {
        volume_display.set_text_content(Some(&slider_position_to_db_display(0.0)));
    } else {
        dev_log!("Warning: tuning-fork-volume-display element not found in HTML");
    }

    if let Some(volume_slider) = document.get_element_by_id("tuning-fork-volume") {
        if let Some(html_slider) = volume_slider.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value("0");
        }
    } else {
        dev_log!("Warning: tuning-fork-volume element not found in HTML");
    }

    // Initialize zoom control elements
    let default_zoom_position = zoom_factor_to_slider_position(crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT);
    CURRENT_ZOOM_LEVEL.store(default_zoom_position as u16, Ordering::Relaxed);
    
    if let Some(zoom_slider) = document.get_element_by_id("zoom-slider") {
        if let Some(html_slider) = zoom_slider.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value(&default_zoom_position.to_string());
        }
    } else {
        dev_log!("Warning: zoom-slider element not found in HTML");
    }
    
    if let Some(zoom_display) = document.get_element_by_id("zoom-display") {
        zoom_display.set_text_content(Some(&format_zoom_display(default_zoom_position)));
    } else {
        dev_log!("Warning: zoom-display element not found in HTML");
    }

    // Verify other essential elements exist
    if document.get_element_by_id("root-note-plus").is_none() {
        dev_log!("Warning: root-note-plus element not found in HTML");
    }
    if document.get_element_by_id("root-note-minus").is_none() {
        dev_log!("Warning: root-note-minus element not found in HTML");
    }
    if document.get_element_by_id("tuning-system-select").is_none() {
        dev_log!("Warning: tuning-system-select element not found in HTML");
    }
    if document.get_element_by_id("scale-select").is_none() {
        dev_log!("Warning: scale-select element not found in HTML");
    }
}

/// Cleans up main scene UI elements and removes event handlers
/// 
/// Currently no cleanup is needed since we're using static HTML elements
/// that remain in the DOM and can be reused.
fn cleanup_ui_impl() {
    // No cleanup needed since we're now using static HTML elements
    // The HTML elements remain in the DOM and can be reused
}

/// Sets up event listeners for UI interaction
fn setup_event_listeners_impl(presenter: std::rc::Rc<std::cell::RefCell<crate::presentation::Presenter>>) {
    let Some(window) = web_sys::window() else {
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
            if let Some(new_root) = crate::shared_types::increment_midi_note(current_root_note) {
                if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                    presenter_mut.on_root_note_adjusted(new_root);
                    
                    // Also update root note audio frequency
                    let position = CURRENT_TUNING_FORK_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
                    let amplitude = slider_position_to_amplitude(position);
                    presenter_mut.on_root_note_audio_configured(true, new_root, amplitude);
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
            if let Some(new_root) = crate::shared_types::decrement_midi_note(current_root_note) {
                if let Ok(mut presenter_mut) = presenter_clone.try_borrow_mut() {
                    presenter_mut.on_root_note_adjusted(new_root);
                    
                    // Also update root note audio frequency
                    let position = CURRENT_TUNING_FORK_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
                    let amplitude = slider_position_to_amplitude(position);
                    presenter_mut.on_root_note_audio_configured(true, new_root, amplitude);
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
                        "equal" => crate::shared_types::TuningSystem::EqualTemperament,
                        "just" => crate::shared_types::TuningSystem::JustIntonation,
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
                        "chromatic" => crate::shared_types::Scale::Chromatic,
                        "major" => crate::shared_types::Scale::Major,
                        "minor" => crate::shared_types::Scale::Minor,
                        "harmonic_minor" => crate::shared_types::Scale::HarmonicMinor,
                        "melodic_minor" => crate::shared_types::Scale::MelodicMinor,
                        "major_pentatonic" => crate::shared_types::Scale::MajorPentatonic,
                        "minor_pentatonic" => crate::shared_types::Scale::MinorPentatonic,
                        "blues" => crate::shared_types::Scale::Blues,
                        "dorian" => crate::shared_types::Scale::Dorian,
                        "phrygian" => crate::shared_types::Scale::Phrygian,
                        "lydian" => crate::shared_types::Scale::Lydian,
                        "mixolydian" => crate::shared_types::Scale::Mixolydian,
                        "locrian" => crate::shared_types::Scale::Locrian,
                        "whole_tone" => crate::shared_types::Scale::WholeTone,
                        "augmented" => crate::shared_types::Scale::Augmented,
                        "diminished_half_whole" => crate::shared_types::Scale::DiminishedHalfWhole,
                        "diminished_whole_half" => crate::shared_types::Scale::DiminishedWholeHalf,
                        "hungarian_minor" => crate::shared_types::Scale::HungarianMinor,
                        "neapolitan_minor" => crate::shared_types::Scale::NeapolitanMinor,
                        "neapolitan_major" => crate::shared_types::Scale::NeapolitanMajor,
                        "enigmatic" => crate::shared_types::Scale::Enigmatic,
                        "persian" => crate::shared_types::Scale::Persian,
                        "double_harmonic_major" => crate::shared_types::Scale::DoubleHarmonicMajor,
                        "altered" => crate::shared_types::Scale::Altered,
                        "bebop_major" => crate::shared_types::Scale::BebopMajor,
                        "bebop_dominant" => crate::shared_types::Scale::BebopDominant,
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

    // Set up tuning fork volume slider event listener
    if let Some(slider) = document.get_element_by_id("tuning-fork-volume") {
        let presenter_clone = presenter.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(current_window) = web_sys::window() {
                if let Some(document) = current_window.document() {
                    if let Some(slider_element) = document.get_element_by_id("tuning-fork-volume") {
                        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
                            if let Ok(position) = html_slider.value().parse::<f32>() {
                                CURRENT_TUNING_FORK_VOLUME_POSITION.store(position as u8, Ordering::Relaxed);
                                
                                // Convert position to amplitude
                                let amplitude = slider_position_to_amplitude(position);
                                
                                // Update volume display with dB value
                                if let Some(display_element) = document.get_element_by_id("tuning-fork-volume-display") {
                                    display_element.set_text_content(Some(&slider_position_to_db_display(position)));
                                }
                                
                                // Update audio
                                let current_root_note = CURRENT_ROOT_NOTE.load(Ordering::Relaxed);
                                presenter_clone.borrow_mut().on_root_note_audio_configured(true, current_root_note, amplitude);
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = slider.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add input listener to volume slider: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find tuning-fork-volume slider");
    }

    // Set up zoom slider event listener
    if let Some(slider) = document.get_element_by_id("zoom-slider") {
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(current_window) = web_sys::window() {
                if let Some(document) = current_window.document() {
                    if let Some(slider_element) = document.get_element_by_id("zoom-slider") {
                        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
                            if let Ok(position) = html_slider.value().parse::<f32>() {
                                // Store the position value
                                CURRENT_ZOOM_LEVEL.store(position as u16, Ordering::Relaxed);
                                
                                // Update zoom display with percentage
                                if let Some(display_element) = document.get_element_by_id("zoom-display") {
                                    display_element.set_text_content(Some(&format_zoom_display(position)));
                                }
                                
                                // Note: The actual zoom factor will be retrieved when needed
                                // using slider_position_to_zoom_factor(position)
                            }
                        }
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        if let Some(event_target) = slider.dyn_ref::<EventTarget>() {
            if let Err(err) = event_target.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref()) {
                dev_log!("Failed to add input listener to zoom slider: {:?}", err);
            }
        }
        closure.forget();
    } else {
        dev_log!("Failed to find zoom-slider");
    }
}

/// Synchronizes HTML UI state with model data
fn sync_ui_state_impl(model_data: &crate::shared_types::ModelUpdateResult) {
    let Some(window) = web_sys::window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    // Update stored root note state
    CURRENT_ROOT_NOTE.store(model_data.root_note, Ordering::Relaxed);

    // Update root note display
    if let Some(display) = document.get_element_by_id("root-note-display") {
        let formatted_note = crate::shared_types::midi_note_to_name(model_data.root_note);
        display.set_text_content(Some(&formatted_note));
    }

    // Update tuning system dropdown selection
    if let Some(select_element) = document.get_element_by_id("tuning-system-select") {
        if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
            let value = match model_data.tuning_system {
                crate::shared_types::TuningSystem::EqualTemperament => "equal",
                crate::shared_types::TuningSystem::JustIntonation => "just",
            };
            html_select.set_value(value);
        }
    }

    // Update scale dropdown selection
    if let Some(select_element) = document.get_element_by_id("scale-select") {
        if let Some(html_select) = select_element.dyn_ref::<HtmlSelectElement>() {
            let value = match model_data.scale {
                crate::shared_types::Scale::Chromatic => "chromatic",
                crate::shared_types::Scale::Major => "major",
                crate::shared_types::Scale::Minor => "minor",
                crate::shared_types::Scale::HarmonicMinor => "harmonic_minor",
                crate::shared_types::Scale::MelodicMinor => "melodic_minor",
                crate::shared_types::Scale::MajorPentatonic => "major_pentatonic",
                crate::shared_types::Scale::MinorPentatonic => "minor_pentatonic",
                crate::shared_types::Scale::Blues => "blues",
                crate::shared_types::Scale::Dorian => "dorian",
                crate::shared_types::Scale::Phrygian => "phrygian",
                crate::shared_types::Scale::Lydian => "lydian",
                crate::shared_types::Scale::Mixolydian => "mixolydian",
                crate::shared_types::Scale::Locrian => "locrian",
                crate::shared_types::Scale::WholeTone => "whole_tone",
                crate::shared_types::Scale::Augmented => "augmented",
                crate::shared_types::Scale::DiminishedHalfWhole => "diminished_half_whole",
                crate::shared_types::Scale::DiminishedWholeHalf => "diminished_whole_half",
                crate::shared_types::Scale::HungarianMinor => "hungarian_minor",
                crate::shared_types::Scale::NeapolitanMinor => "neapolitan_minor",
                crate::shared_types::Scale::NeapolitanMajor => "neapolitan_major",
                crate::shared_types::Scale::Enigmatic => "enigmatic",
                crate::shared_types::Scale::Persian => "persian",
                crate::shared_types::Scale::DoubleHarmonicMajor => "double_harmonic_major",
                crate::shared_types::Scale::Altered => "altered",
                crate::shared_types::Scale::BebopMajor => "bebop_major",
                crate::shared_types::Scale::BebopDominant => "bebop_dominant",
            };
            html_select.set_value(value);
        }
    }

    // Update volume slider and display
    let current_position = CURRENT_TUNING_FORK_VOLUME_POSITION.load(Ordering::Relaxed) as f32;
    if let Some(slider_element) = document.get_element_by_id("tuning-fork-volume") {
        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value(&current_position.to_string());
        }
    }
    if let Some(display_element) = document.get_element_by_id("tuning-fork-volume-display") {
        display_element.set_text_content(Some(&slider_position_to_db_display(current_position)));
    }

    // Update zoom slider and display
    let current_zoom_position = CURRENT_ZOOM_LEVEL.load(Ordering::Relaxed) as f32;
    if let Some(slider_element) = document.get_element_by_id("zoom-slider") {
        if let Some(html_slider) = slider_element.dyn_ref::<HtmlInputElement>() {
            html_slider.set_value(&current_zoom_position.to_string());
        }
    }
    if let Some(display_element) = document.get_element_by_id("zoom-display") {
        display_element.set_text_content(Some(&format_zoom_display(current_zoom_position)));
    }
}

/// Gets the current UI zoom factor
fn get_zoom_factor_impl() -> f32 {
    let position = CURRENT_ZOOM_LEVEL.load(Ordering::Relaxed) as f32;
    slider_position_to_zoom_factor(position)
}

/// Convert slider position (0-100) to amplitude (0.0-1.0) using dual-scale approach
/// - 0-20%: Linear scaling from 0.0 to 0.01 amplitude
/// - 20-100%: dB scaling from -40dB to 0dB
fn slider_position_to_amplitude(position: f32) -> f32 {
    if position <= 0.0 {
        0.0
    } else if position <= 20.0 {
        // Linear scaling: 0-20% maps to 0.0-0.01 amplitude
        position * 0.01 / 20.0
    } else {
        // dB scaling: 20-100% maps to -40dB to 0dB
        let db = -40.0 + (position - 20.0) * 40.0 / 80.0;
        10.0_f32.powf(db / 20.0)
    }
}

/// Convert slider position to dB display string
/// - 0%: Shows "-∞ dB"
/// - 0-20%: Calculates dB from amplitude
/// - 20-100%: Maps directly to dB scale
fn slider_position_to_db_display(position: f32) -> String {
    if position <= 0.0 {
        "-∞ dB".to_string()
    } else if position <= 20.0 {
        // Calculate dB from the amplitude in the linear range
        let amplitude = slider_position_to_amplitude(position);
        if amplitude > 0.0 {
            let db = 20.0 * amplitude.log10();
            format!("{:.0} dB", db)
        } else {
            "-∞ dB".to_string()
        }
    } else {
        // Direct dB mapping for 20-100% range
        let db = -40.0 + (position - 20.0) * 40.0 / 80.0;
        format!("{:.0} dB", db)
    }
}

/// Convert zoom slider position (0-1000) to zoom factor (0.95-2.85)
/// Position 0 maps to PITCH_VISUALIZATION_ZOOM_DEFAULT (0.95)
/// Position 1000 maps to PITCH_VISUALIZATION_ZOOM_MAX (2.85)
fn slider_position_to_zoom_factor(position: f32) -> f32 {
    let clamped = position.clamp(0.0, 1000.0);
    // Map 0-1000 to 0.95-2.85
    let normalized = clamped / 1000.0; // 0.0 to 1.0
    let zoom_range = crate::app_config::PITCH_VISUALIZATION_ZOOM_MAX - crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT;
    crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT + normalized * zoom_range
}

/// Convert zoom factor (0.95-2.85) to slider position (0-1000)
fn zoom_factor_to_slider_position(factor: f32) -> f32 {
    let zoom_range = crate::app_config::PITCH_VISUALIZATION_ZOOM_MAX - crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT;
    let normalized = (factor - crate::app_config::PITCH_VISUALIZATION_ZOOM_DEFAULT) / zoom_range;
    let position = normalized * 1000.0;
    position.clamp(0.0, 1000.0)
}

/// Format zoom position for display (e.g., "100%")
/// Position 0 shows as 100%, position 1000 shows as 250%
fn format_zoom_display(position: f32) -> String {
    // Map 0-1000 to 100%-250%
    let percentage = 100.0 + (position / 1000.0) * 150.0;
    format!("{:.0}%", percentage)
}