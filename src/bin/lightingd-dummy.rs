// A debug version of lightingd that can be run without root
// and that only prints the received control messages.

fn main() {
    let vec = vec![4, 30, 100];
    firelight::render_fire(0.1, &vec);
}
