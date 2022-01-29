#![allow(unreachable_code)]
#[macro_use]
extern crate rouille;
extern crate serde;

use std::env;
use std::io;
use std::sync::Mutex;

use serde::Serialize;

use firelight::control_msg::Control;

#[derive(Serialize, Debug)]
struct StatusResponse {
    on: bool,
    brightness: u8,
}

impl StatusResponse {
    fn from_control(control: Control) -> StatusResponse {
        return StatusResponse{on: control.on, brightness: control.brightness};
    }
}

struct ServerState {
    last_state: Control,
    firelight: firelight::Handle, // [DEV-ONLY] This should still be replaced by a domain socket
}

impl ServerState {
    fn new() -> ServerState {
        let strands = vec![39, 31, 38, 20];
        let handle = firelight::Handle::new(5, 0, 18, strands);
        return ServerState{last_state: Control::default(), firelight: handle};
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let url = if args.len() > 1 {
        args[1].as_str()
    } else {
        "0.0.0.0:1313"
    };
    print!("starting server listening on {}\n", url);

    let server_state = Mutex::new(ServerState::new());

    rouille::start_server(url, move |request| {
        rouille::log(&request, io::stdout(), || {
            router!(request,
                (GET) (/) => {
                    return rouille::Response::redirect_302("/status");
                },

                (GET) (/status) => {
                    let state = try_or_400!(server_state.lock());
                    return rouille::Response::json(&StatusResponse::from_control(state.last_state));
                },

                (POST) (/control) => {
                    let input = try_or_400!(post_input!(request, {
                        on: String,
                        brightness: Option<u8>,
                        color_rgb: Option<String>,
                        transition: Option<i32>,
                    }));
                    println!("got '/control' input {:?}", input);
                    let mut control = firelight::control_msg::Control::default();
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
                    {
                        let mut state = try_or_400!(server_state.lock());
                        state.last_state = control;
                        state.firelight.control(control);
                    }
                    return rouille::Response::text("success");
                },

                _ => rouille::Response::empty_404()
            )
        })
    });
}
