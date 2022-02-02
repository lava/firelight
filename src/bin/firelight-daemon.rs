use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::ops::DerefMut;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::Arc;
use std::thread;

use clap::Parser;
use std::sync::Mutex;

use firelight::daemon;
use firelight::args::DaemonArgs;
use firelight::ledstrip::DeviceController;

struct DaemonState {
    hw: DeviceController,
}

fn handle_client(
    mut stream: UnixStream,
    state_mutex: Arc<Mutex<DaemonState>>,
) -> anyhow::Result<()> {
    let mut buffer: [u32; 256] = [0; 256];
    loop {
        let n = daemon::read_input(&mut stream, &mut buffer)?;
        println!("got {} colors", n);
        let maybe_state = state_mutex.lock();
        match maybe_state {
            Ok(mut state) => state.deref_mut().hw.apply(&buffer[0..n]),
            Err(e) => {
                println!("shared state is poisoned : {}", e);
                break;
            }
        }
        //
    }
    return Ok(());
}

fn main() -> anyhow::Result<()> {
    let args = DaemonArgs::parse();
    // It would be cleaner to delete this on shutdown using RAII,
    // but rust doesn't unwind after signals so that cannot work.
    if Path::new(&args.unix_socket).exists() {
        std::fs::remove_file(&args.unix_socket)?;
    }
    let handle = firelight::ledstrip::DeviceController::new(
        args.dma,
        args.channel,
        args.pin,
        args.leds_count,
    )?;
    let state = DaemonState { hw: handle };
    let shared_state = Arc::new(Mutex::new(state));
    let listener = UnixListener::bind(&args.unix_socket)?;
    std::fs::set_permissions(&args.unix_socket, Permissions::from_mode(0o666))?;
    println!("listening on {}", args.unix_socket);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let thread_state = shared_state.clone();
                /* connection succeeded */
                thread::spawn(move || handle_client(stream, thread_state));
            }
            Err(err) => {
                println!("couldn't accept client: {}", err);
                continue;
            }
        }
    }
    return Ok(());
}
