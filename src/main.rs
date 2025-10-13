use brightnessctl_rs::{get_handler, max_handler, read_all_brightness_devices, set_handler};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "brightnessctl")]
#[command(about = "Brightness regulation for your laptop screen", long_about = None)]
struct Cli {
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

fn main() {
    let args = Cli::parse();

    let devices = read_all_brightness_devices();
    let device_name = args.device.unwrap_or_else(|| String::from("amdgpu_bl1"));
    let default_device_option =
        Iterator::last(devices.iter().filter(|x| x.device_name.eq(&device_name)));

    match default_device_option {
        Some(default_device) => match args.command {
            Commands::Get => {
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
        },
        None => {
            println!("No matching device found");
        }
    }
}
