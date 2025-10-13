use std::{
    cmp::{max, min},
    fs,
    path::{Path, PathBuf},
};

use regex::Regex;

#[derive(Debug)]
struct MaxBrightness(u32);

#[derive(Debug)]
struct Brightness(u32);

#[derive(Debug, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
enum BrightnessClass {
    Backlight,
    Leds,
}

#[derive(Debug)]
pub struct BrightnessDevice {
    pub device_name: String,
    max_brightness: MaxBrightness,
    brightness: Brightness,
    brightness_class: BrightnessClass,
}

#[derive(Debug)]
struct RawBrightnessConfig(String, Option<String>, Option<String>);

fn write_brightness_value_for_device(device: &BrightnessDevice, value: Brightness) {
    let resolved_brightness = min(max(0, value.0), device.max_brightness.0);
    println!("Setting to: {}", resolved_brightness);

    let root = format!("/sys/class/{}/", device.brightness_class);
    let path = Path::new(root.as_str())
        .join(&device.device_name)
        .join("brightness");

    let write_result = fs::write(path, resolved_brightness.to_string());
    match write_result {
        Ok(()) => {}
        Err(err) => eprintln!(
            "Something went wrong when updating the brightness: {:?}",
            err
        ),
    }
}

pub fn max_handler(backlight_device: &BrightnessDevice) {
    println!("{}", backlight_device.max_brightness.0);
}

pub fn get_handler(device: &BrightnessDevice) {
    println!("{:?}", device.brightness.0);
}

fn build_device(class: BrightnessClass, device_name: String) -> Result<BrightnessDevice, String> {
    let root = format!("/sys/class/{}/", class);
    let path = Path::new(root.as_str()).join(&device_name);
    let brightness = read_u32_from_file(path.join("brightness"));
    let max_brightness = read_u32_from_file(path.join("max_brightness"));

    Ok(BrightnessDevice {
        device_name,
        max_brightness: MaxBrightness(max_brightness),
        brightness: Brightness(brightness),
        brightness_class: class,
    })
}

fn read_u32_from_file(path: PathBuf) -> u32 {
    let file_content_result = fs::read_to_string(path.clone());
    let content_raw = file_content_result.unwrap_or_default();
    let trimmed = content_raw.trim();
    trimmed.parse::<u32>().unwrap()
}

fn get_subdirectories(path: &str) -> Result<Vec<String>, std::io::Error> {
    let mut subdirs = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    subdirs.push(name_str.to_string());
                }
            }
        }
    }

    Ok(subdirs)
}

pub fn set_handler(device: &BrightnessDevice, desired_brightness: String) {
    let brightness_config = parse_brightness(desired_brightness.as_str());

    if let Some(bc) = brightness_config {
        println!("{:?}", bc);

        let as_data = bc.0.parse::<f32>().unwrap_or_default();

        println!("Found device details: {:?}", device);

        // FIXME: the behavior should be based on the operand. If the operand is set, then we
        // should perform arithmentics relative to the current brightness
        let max_brightness = device.max_brightness.0;
        let d = match bc.1 {
            Some(_) => max_brightness as f32 * (as_data / 100.0),
            None => as_data,
        } as u32;

        let applied_operator = match bc.2 {
            Some(e) => match e.as_str() {
                "+" => device.brightness.0 + d,
                "-" => device.brightness.0 - d,
                _ => d,
            },
            None => d,
        };

        write_brightness_value_for_device(device, Brightness(applied_operator));
    }
}

fn parse_brightness(input: &str) -> Option<RawBrightnessConfig> {
    let re = Regex::new(r"(\d+)(%)?([+-])?").unwrap();

    if let Some(caps) = re.captures(input) {
        let number = caps.get(1)?.as_str().to_string();
        let percent = caps.get(2).map(|m| m.as_str().to_string());
        let modifier = caps.get(3).map(|m| m.as_str().to_string());

        Some(RawBrightnessConfig(number, percent, modifier))
    } else {
        None
    }
}

pub fn read_all_brightness_devices() -> Vec<BrightnessDevice> {
    let mut backlight_devices: Vec<BrightnessDevice> = get_subdirectories("/sys/class/backlight/")
        .unwrap()
        .iter()
        .map(|x| build_device(BrightnessClass::Backlight, x.clone()).unwrap())
        .collect();

    let mut led_devices: Vec<BrightnessDevice> = get_subdirectories("/sys/class/leds/")
        .unwrap()
        .iter()
        .map(|x| build_device(BrightnessClass::Leds, x.clone()).unwrap())
        .collect();

    let mut x: Vec<BrightnessDevice> = vec![];

    x.append(&mut backlight_devices);
    x.append(&mut led_devices);

    x
}
