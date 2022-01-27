use std::thread;
use std::time::Duration;

extern crate ws281x;


fn main() {
	let mut handler = ws281x::handle::new()
		.dma(5)
		.channel(0, ws281x::channel::new()
			.pin(18)
			.count(128)
			.brightness(55)
			.build().unwrap())
		.build().unwrap();


    // strand identification pattern
    let strands = vec![39, 31, 38, 20];
    let mut cumsum = Vec::new();
    for strand in strands {
       let prev = cumsum.last().copied().unwrap_or(0);
       cumsum.push(prev + strand);
    }

    let mut idx = 0;
    let colors = vec![0xffffff, 0xff0000, 0x00ff00, 0x0000ff];
	for (i, led) in handler.channel_mut(0).leds_mut().iter_mut().enumerate() {
       while idx < cumsum.len() && i >= cumsum[idx] {
            idx += 1;
       }
       if idx >= colors.len() {
           break;
       }
       *led = colors[idx];
    }

    handler.render().unwrap();
	handler.wait().unwrap();


    /*
    // dynamic blink pattern
	let mut check = 0;
	loop {
		for (i, led) in handler.channel_mut(0).leds_mut().iter_mut().enumerate() {
			if i % 2 == check {
				*led = 0
			}
			else {
				*led = 0xffffff;
			}
		}

		handler.render().unwrap();
		handler.wait().unwrap();

		thread::sleep(Duration::from_millis(500));
		check = if check == 0 { 1 } else { 0 };
	}
    */
}
