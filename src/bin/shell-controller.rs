fn main() {
    let strands = vec![39, 31, 38, 20];
    let mut device = firelight::Handle::new(5, 0, 18, strands);
    let mut state = false;
    println!("press enter to toggle light status");
    let mut input = String::new();
    loop {
        std::io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        state = !state;
        device.control(firelight::Control { on: state });
    }
}
