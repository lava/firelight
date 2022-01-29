// TODO: re-evaluate pub

use std::sync::mpsc;

use noise::NoiseFn;
use noise::Perlin;

use std::time::Duration;

use crate::control_msg::ControlMsg;
use crate::control_msg::Control;
use crate::control_msg::Effect;

pub(crate) struct ControlThreadData {
    pub rx: mpsc::Receiver<ControlMsg>,

    pub hw: ws281x::handle::Handle,
    pub strands: Vec<usize>,

    // the last received control msg
    pub state: Control,
}

pub(crate) fn light_control_thread(mut data: ControlThreadData) -> () {
    let mut t = 0.0;
    let delta = 0.01;
    loop {
        t += delta;
        let off = LedColor::from_u32_rgb(0x0);
        let colors = match data.state.effect {
            Effect::Static => render_static(t, &data.strands),
            Effect::Fire => render_fire(t, &data.strands),
        };
        for (i, led) in data.hw.channel_mut(0).leds_mut().iter_mut().enumerate() {
            let mut color = if data.state.on { colors[i] } else { off };
            if data.state.on {
                color.naive_scale(data.state.brightness)
            }
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


fn render_fire(t: f64, strands: &Vec<usize>) -> Vec<LedColor> {
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
        for _ in 0..num {
            result.push(LedColor::from_u32_rgb(0x0));
        }
        for _ in num..*strand {
            result.push(LedColor::from_u32_rgb(0xffffff));
        }
    }
    return result;
}

fn render_static(_t: f64, strands: &Vec<usize>) -> Vec<LedColor> {
        // let mut cumsum = Vec::new();
        // for strand in &strands {
        //     let prev = cumsum.last().copied().unwrap_or(0);
        //     cumsum.push(prev + strand);
        // }
        let on = LedColor::from_u32_rgb(0xffffff);
        let mut result = Vec::new();
        for strand in strands {
            for _ in 0..*strand {
                result.push(on);
            }
        }
        return result;
}
