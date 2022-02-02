use std::os::unix::net::UnixStream;
use std::io::Write;

use clap::Parser;

use firelight::args::ServerArgs;

macro_rules! skip_fail {
    ($res:expr) => {
        match $res {
            Ok(val) => val,
            Err(e) => {
                println!("Invalid value: {}", e);
                continue;
            }
        }
    };
}

/// A shell for interactive debugging.
fn main() -> anyhow::Result<()> {
    let args = ServerArgs::parse();
    let socket = UnixStream::connect(args.daemon_socket)?;
    let mut device = firelight::Handle::new(socket, args.strands);
    let mut input = String::new();
    let mut control = firelight::Control::default();
    loop {
        print!("firelight> ");
        std::io::stdout().flush().unwrap();
        input.clear();
        std::io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        if input.ends_with('\n') {
            input.pop();
        }
        let split = input.split("=");
        let vec : Vec<&str> = split.collect();
        if vec.len() != 2 {
            println!("expected input in the form of key=value");
            continue;
        }
        let (key, value) = (vec[0], vec[1]);
        match key {
            "help" => println!("Valid commands are: brightness=NUM, h=FLOAT, s=FLOAT, effect=STRING, on=BOOL"),
            "brightness" => control.brightness = skip_fail!(value.parse::<u8>()),
            "h" => control.color_hs.0 = skip_fail!(value.parse::<f32>()),
            "s" => control.color_hs.1 = skip_fail!(value.parse::<f32>()),
            "effect" => control.effect = skip_fail!(firelight::Effect::from_string(value)),
            "on" => control.on = skip_fail!(value.parse::<bool>()),
            _ => { println!("unknown key {}", key); continue; },
        }
        device.control(control);
    }
}
