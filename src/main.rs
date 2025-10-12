use std::{
    cmp::min,
    fmt::{self},
    fs::{self},
    path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use regex::Regex;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(name = "brightnessctl")]
#[command(about = "Brightness regulation for your laptop screen", long_about = None)]
struct Cli {
    /// Name of the person to greet
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Get,
    Set { value: String },
    Max,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Get => {
            get_handler();
        }
        Commands::Set { value } => {
            set_handler(value);
        }
        Commands::Max => {
            max_handler();
        }
    }
}

#[derive(Debug)]
struct Brightness(u32);

#[derive(Debug)]
struct BacklightDevice {
    device_name: String,
    max_brightness: u32,
    // this value is not necessarily the same as brightness
    actual_brightness: u32,
    brightness: Brightness,
}

fn write_brightness_value_for_device(device: &BacklightDevice, value: Brightness) {
    let resolved_brightness = min(value.0, device.max_brightness);
    println!("Setting to: {}", resolved_brightness);

    let root = "/sys/class/backlight/";
    let path = Path::new(root).join(&device.device_name).join("brightness");

    let write_result = fs::write(path, resolved_brightness.to_string());
    match write_result {
        Ok(()) => {}
        Err(err) => eprintln!(
            "Something went wrong when updating the brightness: {:?}",
            err
        ),
    }
}

fn max_handler() {
    let device = read_device(BrightnessClass::Backlight, String::from("amdgpu_bl1"));

    match device {
        BrightnessDevice::Backlight(d) => println!("{}", d.max_brightness),
    }
}

fn get_handler() {
    let raw_devices = get_subdirectories("/sys/class/backlight/");

    match raw_devices {
        Ok(value) => {
            for device in value {
                let parsed = build_device(device);

                match parsed {
                    Ok(d) => println!("{:?}", d.brightness.0),
                    Err(err) => println!("{:?}", err),
                }
            }
        }
        Err(err) => println!("{:?}", err),
    }
}

#[derive(Debug)]
enum DeviceError {
    BrightnessNotFound,
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceError::BrightnessNotFound => write!(f, "BrightnessNotFound"),
        }
    }
}

impl std::error::Error for DeviceError {}

fn build_device(device_name: String) -> Result<BacklightDevice, DeviceError> {
    let root = "/sys/class/backlight/";
    let path = Path::new(root).join(&device_name);
    let brightness = read_u32_from_file(path.join("brightness"));
    let max_brightness = read_u32_from_file(path.join("max_brightness"));
    let actual_brightness = read_u32_from_file(path.join("actual_brightness"));

    Ok(BacklightDevice {
        device_name,
        max_brightness,
        brightness: Brightness(brightness),
        actual_brightness,
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

enum BrightnessClass {
    Backlight,
}

enum BrightnessDevice {
    Backlight(BacklightDevice),
}

fn read_device(class: BrightnessClass, device_name: String) -> BrightnessDevice {
    let device = build_device(device_name);
    BrightnessDevice::Backlight(device.unwrap())
}

fn set_handler(desired_brightness: String) {
    let device = read_device(BrightnessClass::Backlight, String::from("amdgpu_bl1"));

    let b = parse_brightness(desired_brightness.as_str());

    match b {
        Some(e) => {
            println!("{:?}", e);

            let as_data = e.0.parse::<f32>().unwrap_or_default();

            match device {
                BrightnessDevice::Backlight(value) => {
                    println!("Found device details: {:?}", value);

                    // FIXME: the behavior should be based on the operand. If the operand is set, then we
                    // should perform arithmentics relative to the current brightness
                    let max_brightness = value.max_brightness;
                    let d = match e.1 {
                        Some(_) => max_brightness as f32 * (as_data / 100.0),
                        None => as_data,
                    } as u32;

                    let applied_operator = match e.2 {
                        Some(e) => match e.as_str() {
                            "+" => value.brightness.0 + d,
                            "-" => value.brightness.0 - d,
                            _ => d,
                        },
                        None => d,
                    };

                    write_brightness_value_for_device(&value, Brightness(applied_operator));
                }
            }
        }
        None => (),
    }
}

fn parse_brightness(input: &str) -> Option<(String, Option<String>, Option<String>)> {
    let re = Regex::new(r"(\d+)(%)?([+-])?").unwrap();

    if let Some(caps) = re.captures(input) {
        let number = caps.get(1)?.as_str().to_string();
        let percent = caps.get(2).map(|m| m.as_str().to_string());
        let modifier = caps.get(3).map(|m| m.as_str().to_string());

        Some((number, percent, modifier))
    } else {
        None
    }
}
