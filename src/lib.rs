// High-level overview:
//
// Protocol:                       ws2811                 domain socket              Control                 ???
// Library Concept:      hardware <--------> controller  <---------------> renderer <------------> client <------------> user
//
// Implementing Binary:                     lightingd                    (firelight)          firelight-rest         homeassistant
//                                                                                            firelight-shell        actual human

pub mod control_msg;
pub mod controller;

// FIXME: move everything else into separate files

use std::sync::mpsc;
use std::time::Duration;

use noise::NoiseFn;
use noise::Perlin;

use crate::control_msg::Control; // FIXME - remove this!
use crate::control_msg::ControlMsg;

pub struct Handle {
    thread: Option<std::thread::JoinHandle<()>>,
    tx: mpsc::Sender<ControlMsg>,

    // Remember the last-sent state so we can offer
    // convenience methods to toggle on/off, adjust
    // brightness, etc.
    state: Control,
}

struct ControlThreadData {
    rx: mpsc::Receiver<ControlMsg>,

    hw: ws281x::handle::Handle,
    strands: Vec<usize>,
    strands_cum: Vec<usize>,

    // the last received control msg
    state: Control,
}

impl Handle {
    pub fn new(rpi_dma: i32, rpi_channel: usize, rpi_pin: i32, strands: Vec<usize>) -> Handle {
        let (tx, rx) = mpsc::channel();
        let join_handle = std::thread::spawn(move || {
            let mut total = 0;
            let mut cumsum = Vec::new();
            for strand in &strands {
                let prev = cumsum.last().copied().unwrap_or(0);
                cumsum.push(prev + strand);
                total += strand;
            }
            // The `rust-ws2811x` library has a built-in `brightness` parameter,
            // that's used to scale every color channel by `c = c * (brightness+1) / 256`.
            // We don't expose that to the user and instead set it to 255 to pass
            // through the exact rgb values that we put in, letting the user take
            // care of handling color spaces, brightness etc.
            let hw_channel = ws281x::channel::new()
                .pin(rpi_pin)
                .count(total)
                .brightness(255)
                .build()
                .unwrap();

            // FIXME: move handler/channel creation into
            // a separate `lightingd` binary.
            let handler = ws281x::handle::new()
                .dma(rpi_dma)
                .channel(rpi_channel, hw_channel)
                .build()
                .unwrap();

            let control_data = ControlThreadData {
                rx: rx,
                hw: handler,
                strands: strands,
                strands_cum: cumsum,
                state: Control::default(),
            };
            return light_control_thread(control_data);
        });

        return Handle {
            thread: Some(join_handle),
            tx: tx,
            state: Control::default(),
        };
    }

    /// Fully set state.
    pub fn control(&mut self, control: Control) {
        self.state = control;
        let _ = self.tx.send(ControlMsg::External(control));
    }

    // Convenience functions to partially change the state.

    /// Toggle the lamp on/off.
    pub fn toggle(&mut self) {
        let mut toggled = self.state;
        toggled.on = !self.state.on;
        self.control(toggled);
    }

    pub fn set_brightness(&mut self, brightness: u8) {
        let mut adjusted = self.state;
        adjusted.brightness = brightness;
        self.control(adjusted);
    }

    pub fn adjust_brightness(&mut self, brightness_delta: i32) {
        let brightness = if brightness_delta > 0 {
            self.state.brightness.saturating_add(brightness_delta as u8)
        } else {
            self.state
                .brightness
                .saturating_sub(-brightness_delta as u8)
        };
        let mut adjusted = self.state;
        adjusted.brightness = brightness;
        self.control(adjusted);
    }

    // Getters for the current state.

    pub fn brightness(&self) -> u8 {
        return self.state.brightness;
    }

    /// Destructor, joins the control thread.
    pub fn drop(&mut self) {
        let _ = self.tx.send(ControlMsg::Shutdown);
        // This is apparently a standard idiom known as the "option dance" [1]
        // [1]: https://users.rust-lang.org/t/spawn-threads-and-join-in-destructor/1613/9
        if let Some(handle) = self.thread.take() {
            match handle.join() {
                Ok(_) => (),
                Err(err) => println!("error while joining: {:#?}", err),
            }
        }
    }
} // impl handle

#[derive(Clone, Copy, Debug)]
pub struct LedColor {
    // r, g, b
    data: [u8; 3],
}

impl LedColor {
    // Initialize from a u32 that looks like 0x00RRGGBB.
    fn from_u32_rgb(x: u32) -> LedColor {
        return LedColor {
            data: [
                ((x >> 16) & 0xff) as u8,
                ((x >> 8) & 0xff) as u8,
                (x & 0xff) as u8,
            ],
        };
    }

    // Render as 0x00RRGGBB.
    fn to_u32_rgb(&self) -> u32 {
        return ((self.data[0] as u32) << 16)
            | ((self.data[1] as u32) << 8)
            | (self.data[0] as u32);
    }

    fn naive_scale(&mut self, scale: u8) {
        let scale32 = scale as u32;
        self.data[0] = (self.data[0] as u32 * (scale32 + 1) / 256) as u8;
        self.data[1] = (self.data[1] as u32 * (scale32 + 1) / 256) as u8;
        self.data[2] = (self.data[1] as u32 * (scale32 + 1) / 256) as u8;
    }
}

pub fn render_fire(t: f64, strands: &Vec<usize>) -> Vec<LedColor> {
    let perlin = Perlin::default();
    let mut noise = Vec::new();
    for (i, _) in strands.iter().enumerate() {
        // Scale return value from [-1, 1] to [0, 1]
        //noise.push((perlin.get([t, 0.0]) + 1.0) / 2.0);
        // TODO: independent noise for all strands
        noise.push((perlin.get([t, i as f64]) + 1.0) / 2.0);
    }
    //println!("noise: {:?}", noise);
    let mut result = Vec::new();
    for (i, strand) in strands.iter().enumerate() {
        let num = (noise[i] * (*strand as f64)) as usize;
        for x in 0..num {
            result.push(LedColor::from_u32_rgb(0x0));
        }
        for x in num..*strand {
            result.push(LedColor::from_u32_rgb(0xffffff));
        }
    }
    return result;
}

fn light_control_thread(mut data: ControlThreadData) -> () {
    let mut t = 0.0;
    let delta = 0.01;
    loop {
        t += delta;
        // let on = LedColor::from_u32_rgb(0xffffff);
        let off = LedColor::from_u32_rgb(0x0);
        // let colors_on = vec![on, on, on, on];
        // let colors_off = vec![off, off, off, off];
        // let mut colors = if data.state.on { colors_on } else { colors_off };
        // for color in &mut colors {
        //     color.naive_scale(data.state.brightness);
        // }
        let colors = render_fire(t, &data.strands);
        let mut idx = 0;
        for (i, led) in data.hw.channel_mut(0).leds_mut().iter_mut().enumerate() {
            let color = if data.state.on { colors[i] } else { off };
            // while idx < data.strands_cum.len() && i >= data.strands_cum[idx] {
            //     idx += 1;
            // }
            // if idx >= colors.len() {
            //     break;
            // }
            *led = color.to_u32_rgb();
        }

        //println!("rendering colors {:?}", colors);

        data.hw.render().unwrap();
        data.hw.wait().unwrap();

        // TODO: Use a separate timer thread for a stable clock pulse
        let msg = data.rx.recv_timeout(Duration::from_millis(1000 / 60));
        match msg {
            Ok(ControlMsg::Shutdown) => break,
            Ok(ControlMsg::External(control)) => data.state = control,
            Err(_) => continue,
        }
    }
}
