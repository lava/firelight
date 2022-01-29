extern crate ws281x;

use std::sync::mpsc;
// use anyhow::anyhow;

pub struct Control {
    pub on: bool,
    brightness: u8,
    // TODO:
    //color_rgb: (u8, u8, u8);
}

pub struct Handle {
    thread: Option<std::thread::JoinHandle<()>>,
    tx: mpsc::Sender<ControlMsg>,

    // Remember the last-sent state so we can offer
    // convenience methods to toggle on/off, adjust
    // brightness, etc.
    state: Control,
}

enum ControlMsg {
    Shutdown,
    External(Control),
}

struct ControlThreadData {
    rx: mpsc::Receiver<ControlMsg>,

    hw: ws281x::handle::Handle,
    strands_cum: Vec<usize>,

    // the last received control msg
    state: Control,
}

impl Control {
  fn default() -> Control {
    return Control {on: false, brightness: 255};
  }
}


impl Handle {
    /// Note that this can currently only run on a supported Raspberry Pi model,
    /// because it needs to know the correct offsets for video core memory and
    /// peripheral memory. To run on other devices, the C headers in the `rust-ws281x`
    /// dependency must be patched to include the relevant definitions for the
    /// new hardware platform.
    ///
    /// Arguments:
    ///   rpi_channel: The PWM channel to which the LED strip is connected. Usually 0.
    ///   rpi_dma: The DMA number to be used. This identifies the memory block used by the
    ///            DMA controller. Can be any number 0-15 that is *not* concurrently used
    ///            by another process or hardware on the same device.
    ///   rpi_pin: The pin to which the LED strip is attached. Will usually be one of the
    ///            PWM pins 12,18 for channel PWM0 or 13,19 for channel PWM1.
    pub fn new(
        rpi_dma: i32,
        rpi_channel: usize,
        rpi_pin: i32,
        strands: Vec<usize>,
    ) -> Handle {
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

    // fully set state
    pub fn control(&mut self, control: Control) {
        let _ = self.tx.send(ControlMsg::External(control));
    }

    // convenience functions
    pub fn toggle(&mut self) {
        let toggled = Control{on: !self.state.on, brightness: self.state.brightness};
        self.control(toggled);
    }

    pub fn set_brightness(&mut self, brightness: u8) {
        let adjusted = Control{on: self.state.on, brightness: brightness};    
        self.control(adjusted);
    }

    pub fn adjust_brightness(&mut self, brightness_delta: u8) {
        let adjusted = Control{on: self.state.on, brightness: self.state.brightness + brightness_delta};    
        self.control(adjusted);
    }

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

#[derive(Clone, Copy)]
struct LedColor {
    // r, g, b
    data: [u8; 3],
}

impl LedColor {
    // Initialize from a u32 that looks like 0x00RRGGBB.
    fn from_u32_rgb(x: u32) -> LedColor {
        return LedColor{data: [((x >> 16) & 0xff) as u8, ((x >> 8) & 0xff) as u8, (x & 0xff) as u8]};
    }

    // Render as 0x00RRGGBB.
    fn to_u32_rgb(&self) -> u32 {
        return ((self.data[0] as u32) << 16) | ((self.data[1] as u32) << 8) | (self.data[0] as u32);
    }

    fn naive_scale(&mut self, scale: u8) {
        let scale32 = scale as u32;
        self.data[0] = (self.data[0] as u32 * (scale32 + 1) / 256) as u8;
        self.data[1] = (self.data[1] as u32 * (scale32 + 1) / 256) as u8;
        self.data[2] = (self.data[1] as u32 * (scale32 + 1) / 256) as u8;
    }
}

fn light_control_thread(mut data: ControlThreadData) -> () {
    loop {
        let on = LedColor::from_u32_rgb(0xffffff);
        let off = LedColor::from_u32_rgb(0x0);
        let colors_on = vec![on, on, on, on];
        let colors_off = vec![off, off, off, off];
        let mut colors = if data.state.on { colors_on } else { colors_off };
        for color in &mut colors {
            color.naive_scale(data.state.brightness);
        }
        let mut idx = 0;
        for (i, led) in data.hw.channel_mut(0).leds_mut().iter_mut().enumerate() {
            while idx < data.strands_cum.len() && i >= data.strands_cum[idx] {
                idx += 1;
            }
            if idx >= colors.len() {
                break;
            }
            *led = colors[idx].to_u32_rgb();
        }

        data.hw.render().unwrap();
        data.hw.wait().unwrap();

        let msg = data.rx.recv().unwrap();
        match msg {
            ControlMsg::Shutdown => break,
            ControlMsg::External(control) => data.state = control,
        }
    }
}
