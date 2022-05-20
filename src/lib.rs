use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

pub struct Scatter<T, S> {
    area: usize, // thread count
    function: fn(T) -> S,
}

impl<T, S> Scatter<T, S> {
    pub fn new(_area: usize, function: fn(T)->S) -> Self { // new: provide function, arguments

        let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();

        Scatter { _area, function }
    } 

    pub fn feed(&self, data: T) -> usize { // feed argument then scatter it


        return 1
    } 
}

// feed returns id once function is evaluated hashmap[id] contains value

#[test]
fn run() {
    let scatter = Scatter::new(5, |data: (&str, u16)| {
        let (ip, port) = data;
        match std::net::TcpStream::connect_timeout(&format!("{}:{}", ip, port).parse().unwrap(), std::time::Duration::from_millis(1000)) {
            Ok(_) => true,
            Err(_) => false
        }
    });

    let id = scatter.feed(("92.204.50.150", 80));
    println!("{}", id);
}

