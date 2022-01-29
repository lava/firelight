fn main() {
    let strands = vec![39, 31, 38, 20];
    let mut device = firelight::Handle::new(5, 0, 18, strands);
    device.set_brightness(128);
    println!("press enter to toggle light status");
    let mut input = String::new();
    let mut delta = 20;
    loop {
        std::io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        device.toggle();
        device.adjust_brightness(delta);
        if device.brightness() == 255 {
            delta = -20;
        }
        if device.brightness() == 0 {
            delta = 20;
        }
    }
}
