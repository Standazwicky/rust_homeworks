use std::time::SystemTime;

fn main() {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
     Ok(n) => println!("Hello! It's been {} seconds since 1970-01-01 00:00:00 UTC",n.as_secs()),
     Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}
