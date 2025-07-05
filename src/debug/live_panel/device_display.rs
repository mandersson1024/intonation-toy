// Device Display - Audio device list rendering utilities
//
// This module provides utilities for displaying audio device information in the live panel.
// It handles formatting and visualization of input/output devices with their properties.

use yew::prelude::*;
use crate::audio::AudioDevices;

/// Device display utilities
pub struct DeviceDisplay;

impl DeviceDisplay {
    /// Render complete device list
    pub fn render_device_list(devices: &AudioDevices) -> Html {
        html! {
            <div class="device-list">
                {Self::render_device_section("Input Devices", &devices.input_devices)}
                {Self::render_device_section("Output Devices", &devices.output_devices)}
            </div>
        }
    }

    /// Render a section of devices (input or output)
    fn render_device_section(title: &str, devices: &[(String, String)]) -> Html {
        html! {
            <div class="device-section">
                <h5 class="device-section-title">{title}</h5>
                <div class="device-items">
                    {if devices.is_empty() {
                        html! {
                            <div class="device-placeholder">
                                {"No devices available"}
                            </div>
                        }
                    } else {
                        html! {
                            <>
                                {for devices.iter().map(|device| Self::render_device_item(device))}
                            </>
                        }
                    }}
                </div>
            </div>
        }
    }

    /// Render a single device item
    fn render_device_item(device: &(String, String)) -> Html {
        html! {
            <div class="device-item">
                <div class="device-info">
                    <span class="device-name">{&device.1}</span>
                    <span class="device-id">{&device.0}</span>
                </div>
                <div class="device-status">
                    {Self::render_device_status()}
                </div>
            </div>
        }
    }

    /// Render device status indicator
    fn render_device_status() -> Html {
        html! {
            <span class="device-status-indicator device-available">
                {"Available"}
            </span>
        }
    }

    /// Render device summary (device counts)
    pub fn render_device_summary(devices: &AudioDevices) -> Html {
        html! {
            <div class="device-summary">
                <div class="device-count">
                    <span class="count-label">{"Input Devices:"}</span>
                    <span class="count-value">{devices.input_devices.len()}</span>
                </div>
                <div class="device-count">
                    <span class="count-label">{"Output Devices:"}</span>
                    <span class="count-value">{devices.output_devices.len()}</span>
                </div>
            </div>
        }
    }

    /// Render device capabilities (if available)
    pub fn render_device_capabilities(device: &(String, String)) -> Html {
        html! {
            <div class="device-capabilities">
                <div class="capability-item">
                    <span class="capability-label">{"ID:"}</span>
                    <span class="capability-value">{&device.0}</span>
                </div>
                <div class="capability-item">
                    <span class="capability-label">{"Label:"}</span>
                    <span class="capability-value">{&device.1}</span>
                </div>
                <div class="capability-item">
                    <span class="capability-label">{"Default:"}</span>
                    <span class="capability-value">{"Unknown"}</span>
                </div>
            </div>
        }
    }
}