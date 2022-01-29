use anyhow::anyhow;

pub struct DeviceController {
    hw: ws281x::handle::Handle,
    channel: usize,
}

impl DeviceController {
    /// Note that this can currently only run on a supported Raspberry Pi model,
    /// because it needs to know the correct offsets for video core memory and
    /// peripheral memory. To run on other devices, the C headers in the `rust-ws281x`
    /// dependency must be patched to include the relevant definitions for the
    /// new hardware platform.
    ///
    /// Arguments:
    ///   rpi_dma:     The DMA number to be used. This identifies the memory block used by the
    ///                DMA controller. Can be any number 0-15 that is *not* concurrently used
    ///                by another process or hardware on the same device.
    ///   rpi_channel: The PWM channel to which the LED strip is connected. Usually 0 or 1.
    ///   rpi_pin:     The pin to which the LED strip is attached. Will usually be one of the
    ///                PWM pins 12,18 for channel PWM0 or 13,19 for channel PWM1.
    ///   leds_count:  How many LEDs the strip contains.
    pub fn new(
        rpi_dma: i32,
        rpi_channel: usize,
        rpi_pin: i32,
        leds_count: usize,
    ) -> anyhow::Result<DeviceController> {
        // The `rust-ws2811x` library has a built-in `brightness` parameter,
        // that's used to scale every color channel by `c = c * (brightness+1) / 256`.
        // We don't expose that to the user and instead set it to 255 to pass
        // through the exact rgb values that we put in, letting the outside take
        // care of handling color spaces, brightness etc.
        let hw_channel = ws281x::channel::new()
            .pin(rpi_pin)
            .count(leds_count)
            .brightness(255)
            .build()
            .map_err(|_e| anyhow!("failed to create channel"))?;

        let handler = ws281x::handle::new()
            .dma(rpi_dma)
            .channel(rpi_channel, hw_channel)
            .build()
            .map_err(|_e| anyhow!("failed to open device"))?;

        return Ok(DeviceController { hw: handler, channel: rpi_channel});
    }

    pub fn apply(&mut self, led_colors: &[u32]) {
        let bound = led_colors.len();
        for (i, led) in self.hw.channel_mut(self.channel).leds_mut().iter_mut().enumerate() {
            if i >= bound {
                break;
            }
            *led = led_colors[i];
        }
        self.hw.render().unwrap();
        self.hw.wait().unwrap();
    }
}
