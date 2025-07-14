use griphd::vm::core;

fn main() {
    let input = match std::fs::read("/tmp/rfid_input.bin") {
        Ok(i) => i,
        Err(_) => return,
    };
    println!("Input: {:?}", input);
    core::handle_payload(&input);
    let addr = core::send_flag as *const ();
    println!("{:#x}", addr as usize);
}
