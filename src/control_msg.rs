#[derive(Copy, Clone, Debug)]
pub enum Effect {
    StaticLight,
    Fire,
}

/// Sent by clients.
/// Used to control the state of the renderer.
#[derive(Copy, Clone, Debug)]
pub struct Control {
    pub on: bool,
    pub brightness: u8,
    pub effect: Effect,
    // TODO:
    //color_rgb: (u8, u8, u8);
}

// TODO: re-evaluate publicness when introducing daemon
pub(crate) enum ControlMsg {
    Shutdown,
    External(Control),
}

impl Control {
    pub fn default() -> Control {
        return Control {
            on: false,
            brightness: 255,
            effect: Effect::StaticLight,
        };
    }
}
