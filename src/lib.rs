use std::ops::{AddAssign, SubAssign};
use std::sync::atomic::AtomicUsize;
use std::thread;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub struct Scatter<S: 'static + Send, T: 'static + Send> {
    area: usize, // max thread count
    function: fn(T) -> S,
    pub values: Arc<RwLock<HashMap<usize, S>>>,
    current_id: AtomicUsize,
    threads: Arc<RwLock<usize>>
}

impl<S: 'static + Send + std::marker::Sync, T: 'static + Send> Scatter<S, T> {
    pub fn new(area: usize, function: fn(T)->S) -> Self { // new: provide function, arguments
        Scatter { area, function, 
            values: Arc::new(RwLock::new(HashMap::new())),
            current_id: AtomicUsize::new(0),
            threads: Arc::new(RwLock::new(0))
        }
    } 

    pub fn feed(&mut self, data: T) -> usize { // feed argument then scatter it
        while self.active_threads().ge(&self.area) { thread::sleep(Duration::from_millis(1)); }
        
        let id = self.current_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let values = Arc::clone(&self.values);
        
        let fun = self.function;
        self.threads.write().expect("RwLock poisoned").add_assign(1);
        let threads = Arc::clone(&self.threads);
        thread::spawn(move || {
            let result = fun(data);
            values.write().expect("RwLock poisoned").insert(id, result);
            threads.write().expect("RwLock poisoned").sub_assign(1);
        });
        return id
    }

    pub fn active_threads(&self) -> usize{ *self.threads.read().unwrap() }

}

// feed returns id once function is evaluated hashmap[id] contains value

#[test]
fn run() {
    let mut scatter = Scatter::new(5, |data: (&str, u16)| {
        let (ip, port) = data;
        match std::net::TcpStream::connect_timeout(&format!("{}:{}", ip, port).parse().unwrap(), std::time::Duration::from_millis(1000)) {
            Ok(_) => true,
            Err(_) => false
        }
    });

    for _ in 0..20 {
        let id = scatter.feed(("92.204.50.150", 80));
        println!("{}", id);
    }

    println!("{:?}", scatter.values);

    thread::sleep(Duration::from_secs(1));

    println!("{:?}", scatter.values);

    println!();
}

