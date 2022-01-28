fn main() {
    let strands = vec![39, 31, 38, 20];
    let mut device = firelight::Handle::new(5, 0, 18, strands);
    println!("press enter to toggle light status");
    let mut input = String::new();
    let mut brightness: u8 = 128;
    loop {
        std::io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        device.toggle();
        brightness += 20;
        device.adjust_brightness(brightness);
    }
}
