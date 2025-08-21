#[derive(Debug, Clone, Default)]
pub struct AudioDevices {
    pub input_devices: Vec<(String, String)>,
    pub output_devices: Vec<(String, String)>,
}