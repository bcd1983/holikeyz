fn main() {
    eprintln!("This is a library crate. Use one of the binary targets:");
    eprintln!("  cargo run --bin elgato-cli");
    eprintln!("  cargo run --bin elgato-dbus-service");
    std::process::exit(1);
}
