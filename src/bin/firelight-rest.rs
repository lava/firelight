#![allow(unreachable_code)]
#[macro_use]
extern crate rouille;
extern crate serde;

use clap::Parser;
use std::io;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;

use serde::Serialize;

use firelight::Control;
use firelight::args::ServerArgs;


#[derive(Serialize, Debug)]
struct StatusResponse {
    on: bool,
    brightness: u8,
    effect: String,
    color_hs: (f32, f32),
}

#[derive(Serialize, Debug)]
struct AboutResponse {
    version: String,
    instance_name: String,
}

impl StatusResponse {
    fn from_control(control: Control) -> StatusResponse {
        return StatusResponse {
            on: control.on,
            brightness: control.brightness,
            effect: control.effect.to_string(),
            color_hs: control.color_hs,
        };
    }
}

struct ServerState {
    last_state: Control,
    firelight: firelight::Handle,
}

impl ServerState {
    fn new(socket: UnixStream, strands: Vec<usize>) -> ServerState {
        let handle = firelight::Handle::new(socket, strands);
        return ServerState {
            last_state: Control::default(),
            firelight: handle,
        };
    }
}

fn main() -> anyhow::Result<()> {
    let firelight_version: &str = option_env!("CARGO_PKG_VERSION").unwrap_or("unknown");
    let args = ServerArgs::parse();
    let uds = UnixStream::connect(args.daemon_socket)?;
    print!("starting server listening on {}\n", args.bind);
    let server_state = Mutex::new(ServerState::new(uds, args.strands));

    rouille::start_server(args.bind, move |request| {
        rouille::log(&request, io::stdout(), || {
            router!(request,
                (GET) (/) => {
                    // TODO: Make a cool HTML control page.
                    return rouille::Response::redirect_302("/status");
                },

                (GET) (/status) => {
                    let state = try_or_400!(server_state.lock());
                    return rouille::Response::json(&StatusResponse::from_control(state.last_state));
                },

                (GET) (/about) => {
                    let about = AboutResponse {
                        version: firelight_version.to_string(),
                        instance_name: args.instance_name.clone(),
                    };
                    return rouille::Response::json(&about);
                },

                (POST) (/control) => {
                    let maybe_input = post_input!(request, {
                        on: String,
                        brightness: Option<u8>,
                        color_hs: Vec<f32>,  // h in [0.0,360.0], s in [0.0, 100.0]
                        effect: Option<String>,
                    });
                    let input = match maybe_input {
                        Ok(v) => v,
                        Err(e) => {println!("error {:?}", e); return rouille::Response::empty_400(); }
                    };
                    println!("got '/control' input {:?}", input);
                    {
                        let mut state = try_or_400!(server_state.lock());
                        let mut control = state.last_state;
                        if input.on == "True" {
                            control.on = true;
                        } else if input.on == "False" {
                            control.on = false;
                        } else {
                            return rouille::Response::text("invalid value for 'on'").with_status_code(400);
                        }
                        if let Some(brightness) = input.brightness {
                            control.brightness = brightness;
                        }
                        if let Some(effect) = input.effect {
                            println!("got effect {}", effect);
                            let maybe_effect = firelight::Effect::from_string(&effect);
                            match maybe_effect {
                                Ok(effect) => control.effect = effect,
                                Err(_) => return rouille::Response::empty_400(),
                            }
                        }
                        if !input.color_hs.is_empty() {
                            if input.color_hs.len() != 2 {
                                return rouille::Response::empty_400();
                            }
                            control.color_hs = (input.color_hs[0], input.color_hs[1]);
                        }
                        state.last_state = control;
                        state.firelight.control(control);
                    }
                    return rouille::Response::text("success");
                },

                _ => rouille::Response::empty_404()
            )
        })
    });
    return Ok(());
}
