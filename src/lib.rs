// High-level overview:
//
// Protocol:                       ws2811                 domain socket              Control                 ???
// Library Concept:      hardware <--------> controller  <---------------> renderer <------------> client <------------> user
//
// Implementing Binary:                     lightingd                    firelight (lib)       firelight-rest         homeassistant
//                                                                                             firelight-shell        actual human

pub mod control_msg;
pub mod daemon;
pub mod firelight_api;
pub mod ledstrip;
pub mod renderer;

pub use firelight_api::*;
