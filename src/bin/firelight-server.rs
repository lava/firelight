#![allow(unreachable_code)]
#[macro_use]
extern crate rouille;

use std::env;
use std::io;
use std::sync::Mutex;

use serde::Serialize;

#[derive(Serialize, Debug)]
struct StatusResponse {
    on: bool,
}

struct ServerState {
    on: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let url = if args.len() > 1 {
        args[1].as_str()
    } else {
        "0.0.0.0:1313"
    };
    print!("starting server listening on {}\n", url);

    let server_state = Mutex::new(ServerState { on: false });

    rouille::start_server(url, move |request| {
        rouille::log(&request, io::stdout(), || {
            router!(request,
                (GET) (/) => {
                    return rouille::Response::redirect_302("/status");
                },

                (GET) (/status) => {
                    let is_on = {
                        let state = try_or_400!(server_state.lock());
                        (*state).on
                    };
                    return rouille::Response::json(&StatusResponse {on: is_on});
                },

                (POST) (/control) => {
                    let input = try_or_400!(post_input!(request, {
                        on: String,
                        brightness: Option<u8>,
                        color_rgb: Option<String>,
                        transition: Option<i32>,
                    }));
                    println!("got '/control' input {:?}", input);
                    {
                        let mut state = try_or_400!(server_state.lock());
                        if input.on == "True" {
                            state.on = true;
                        } else if input.on == "False" {
                            state.on = false;
                        } else {
                            return rouille::Response::text("invalid value for 'on'").with_status_code(400);
                        }
                    }
                    return rouille::Response::text("success");
                },

                _ => rouille::Response::empty_404()
            )
        })
    });
}
