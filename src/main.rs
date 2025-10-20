use brightnessctl_rs::{
    config, get_default_backlight_device, get_handler, max_handler, read_all_brightness_devices,
    set_handler,
};
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

    // FIXME: if there is no config, we should make a sane default, like putting the backlight device name
    // in there
    let devices = read_all_brightness_devices();
    let default_device = get_default_backlight_device(&devices);
    let config = config::load_config();

    let config_value = match config {
        Ok(c) => {
            println!("{:?}", c);
            c.default_device
        }
        Err(e) => {
            eprintln!("{}", e);
            default_device.unwrap().device_name.clone()
        }
    };

    let device_name = args.device.unwrap_or(config_value);
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
