use std::os::unix::net::UnixListener;
use anyhow::anyhow;
use clap::Parser;

use firelight::ledstrip::DeviceController;
use firelight::cmdline_args::DaemonArgs;

fn main() -> anyhow::Result<()> {
    let args = DaemonArgs::parse();
    let handle = firelight::ledstrip::DeviceController::new(args.dma, args.channel, args.pin, args.leds_count)?;
    let listener = UnixListener::bind(&args.unix_socket)?;
    println!("listening on {}", args.unix_socket);
    for stream in listener.incoming() {
    	
    }
    std::fs::remove_file(&args.unix_socket)?;
    return Ok(());
}
