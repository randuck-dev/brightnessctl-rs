use std::{
    cmp::{max, min},
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

    #[arg(short, long, global = true)]
    device: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Get,
    Set { value: String },
    Max,
    List,
}

fn read_all_brightness_devices() -> Vec<BacklightDevice> {
    let mut backlight_devices: Vec<BacklightDevice> = get_subdirectories("/sys/class/backlight/")
        .unwrap()
        .iter()
        .map(|x| build_device(BacklightType::Backlight, x.clone()).unwrap())
        .collect();

    let mut led_devices: Vec<BacklightDevice> = get_subdirectories("/sys/class/leds/")
        .unwrap()
        .iter()
        .map(|x| build_device(BacklightType::Leds, x.clone()).unwrap())
        .collect();

    let mut x: Vec<BacklightDevice> = vec![];

    x.append(&mut backlight_devices);
    x.append(&mut led_devices);

    x
}

fn main() {
    let args = Cli::parse();

    let devices = read_all_brightness_devices();
    let device_name = args.device.unwrap_or_else(|| String::from("amdgpu_bl1"));
    let default_device = devices
        .iter()
        .clone()
        .filter(|x| x.device_name.eq(&device_name))
        .last()
        .unwrap();

    match args.command {
        Commands::Get {} => {
            get_handler(default_device);
        }
        Commands::Set { value } => {
            set_handler(default_device, value);
        }
        Commands::Max => {
            max_handler(default_device);
        }
        Commands::List => {
            println!("{:?}", devices)
        }
    }
}

#[derive(Debug)]
struct Brightness(u32);

#[derive(Debug)]
enum BacklightType {
    Backlight,
    Leds,
}

#[derive(Debug)]
struct BacklightDevice {
    device_name: String,
    max_brightness: u32,
    brightness: Brightness,
    backlight_type: BacklightType,
}

fn write_brightness_value_for_device(device: &BacklightDevice, value: Brightness) {
    let resolved_brightness = min(max(0, value.0), device.max_brightness);
    println!("Setting to: {}", resolved_brightness);

    let class_sub_path = match device.backlight_type {
        BacklightType::Backlight => "backlight",
        BacklightType::Leds => "leds",
    };
    let root = format!("/sys/class/{}/", class_sub_path);
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

fn max_handler(backlight_device: &BacklightDevice) {
    println!("{}", backlight_device.max_brightness);
}

fn get_handler(device: &BacklightDevice) {
    println!("{:?}", device.brightness.0);
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

fn build_device(class: BacklightType, device_name: String) -> Result<BacklightDevice, DeviceError> {
    let class_sub_path = match class {
        BacklightType::Backlight => "backlight",
        BacklightType::Leds => "leds",
    };
    let root = format!("/sys/class/{}/", class_sub_path);
    let path = Path::new(root.as_str()).join(&device_name);
    let brightness = read_u32_from_file(path.join("brightness"));
    let max_brightness = read_u32_from_file(path.join("max_brightness"));

    Ok(BacklightDevice {
        device_name,
        max_brightness,
        brightness: Brightness(brightness),
        backlight_type: class,
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

fn read_device(class: BacklightType, device_name: String) -> BacklightDevice {
    let device = build_device(class, device_name);
    device.unwrap()
}

fn set_handler(device: &BacklightDevice, desired_brightness: String) {
    let b = parse_brightness(desired_brightness.as_str());

    match b {
        Some(e) => {
            println!("{:?}", e);

            let as_data = e.0.parse::<f32>().unwrap_or_default();

            println!("Found device details: {:?}", device);

            // FIXME: the behavior should be based on the operand. If the operand is set, then we
            // should perform arithmentics relative to the current brightness
            let max_brightness = device.max_brightness;
            let d = match e.1 {
                Some(_) => max_brightness as f32 * (as_data / 100.0),
                None => as_data,
            } as u32;

            let applied_operator = match e.2 {
                Some(e) => match e.as_str() {
                    "+" => device.brightness.0 + d,
                    "-" => device.brightness.0 - d,
                    _ => d,
                },
                None => d,
            };

            write_brightness_value_for_device(&device, Brightness(applied_operator));
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
