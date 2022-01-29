use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::thread;
use anyhow::anyhow;

use clap::Parser;

use firelight::ledstrip::DeviceController;
use firelight::cmdline_args::DaemonArgs;

struct DaemonState {
}

fn as_bytes(v: &mut [u32]) -> &mut [u8] {
    unsafe {
        let (_prefix, result, _suffix) = v.align_to_mut::<u8>();
        return result;
    }
}

fn handle_client(mut stream: UnixStream) -> anyhow::Result<()> {
    let mut buffer : [u32; 256] = [0; 256];
    loop {
        let n = stream.read(&mut as_bytes(&mut buffer)[..])?;
        println!("got {} bytes", n);
        if n <= 0 {
            break;
        }
    }
    return Ok(());
}

/// A debug version of lightingd that only prints the color pattern it would apply.
/// TODO: Open a window and draw results.
fn main() -> anyhow::Result<()> {
    let args = DaemonArgs::parse();
    // It would be cleaner to delete this on shutdown using RAII,
    // but rust doesn't unwind after signals.
    if Path::new(&args.unix_socket).exists() {
        std::fs::remove_file(&args.unix_socket)?;
    }
    let listener = UnixListener::bind(&args.unix_socket)?;
    println!("listening on {}", args.unix_socket);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("new client");
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                println!("couldn't accept client: {}", err);
                continue;
            }
        }
    }
    return Ok(());
}
