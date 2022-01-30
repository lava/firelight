use std::path::Path;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::thread;

use clap::Parser;

use firelight::daemon::DaemonArgs;
use firelight::daemon;

fn handle_client(mut stream: UnixStream) -> anyhow::Result<()> {
    let mut buffer : [u32; 256] = [0; 256];
    loop {
        let n = daemon::read_input(&mut stream, &mut buffer)?;
        if n == 0 {
            break;
        }
        println!("got {} colors", n);
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
