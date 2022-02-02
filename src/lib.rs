// High-level overview:
//
// Protocol:                       ws2811                 domain socket              Control                    ???
// Library Concept:      hardware <--------> controller  <---------------> renderer <------------> server <------------> user
//
// Implementing Binary:                    firelight-daemon                 (lib)              firelight-rest         homeassistant
//                                                                                             debug-shell            actual human

pub mod daemon;
pub mod firelight_api;
pub mod ledstrip;
pub mod renderer;
pub mod args;

pub use firelight_api::*;
