use std::ops::{AddAssign, SubAssign};
use std::{collections::HashMap, thread} ;
use std::sync::{Arc, RwLock, Mutex, atomic::AtomicUsize};

pub struct Scatter<S, T> {
    area: usize, // max thread count
    function: Arc<fn(T) -> Option<S>>,
    results: Arc<RwLock<HashMap<usize, S>>>,
    current_id: AtomicUsize,
    eaters: Arc<RwLock<usize>>,
    data: Arc<Mutex<Vec<(usize, T)>>>,
    queue_limit: usize,
}

impl<S: 'static + Send + Sync, T: 'static + Send> Scatter<S, T> {
    pub fn new(area: usize, queue_limit: usize, function: fn(T)->Option<S>) -> Self { // new: provide function, arguments
        let _area = if area == 0 { usize::MAX } else { area };
        let _queue_limit = if queue_limit == 0 { usize::MAX } else { queue_limit };

        Scatter { area: _area, queue_limit: _queue_limit,
            function: Arc::new(function), 
            results: Arc::new(RwLock::new(HashMap::new())),
            current_id: AtomicUsize::new(0),
            eaters: Arc::new(RwLock::new(0)),
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // if active threads < max threads create new thread, thread gets data itself
    // else thread on finish checks queue for new data 
    //    if new data repeats cycle
    //    else exits decreases counter of active threads

    fn eat(&self) {
        if self.get_eaters() < self.area { self.dispatch_eater() }
    }

    fn dispatch_eater(&self) {
        let data = Arc::clone(&self.data);
        let results = Arc::clone(&self.results);
        let function = Arc::clone(&self.function);
        let eaters = Arc::clone(&self.eaters);

        let mut lock = self.eaters.write().unwrap();
        lock.add_assign(1);
        drop(lock);
        thread::spawn(move || {
            let mut has_data = true;

            while has_data {
                let mut lock = data.lock().unwrap();
                let d = lock.pop();
                drop(lock);
                match d {
                    Some((cur_id, cur_data)) => {
                        let result = function(cur_data);
                        match result {
                            Some(value) => results.write().unwrap().insert(cur_id, value),
                            None => None
                        };
                    },
                    None => {
                        has_data = false;
                    }
                };
            }

            eaters.write().unwrap().sub_assign(1);
        });
    }

    pub fn feed(&self, data: T) -> Option<usize> { // feed data get data processing id
        let mut lock = self.data.lock().unwrap();
        if lock.len() >= self.queue_limit { 
            drop(lock);
            return None
        }

        let id = self.current_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        lock.push((id, data));
        drop(lock);
        self.eat();
        return Some(id);
    }

    pub fn get_eaters(&self) -> usize{ *self.eaters.read().unwrap() }
    pub fn drain_results(&self) -> HashMap<usize, S> { self.results.write().unwrap().drain().collect() }
    pub fn get_queue_length(&self) -> usize { self.data.lock().unwrap().len() }

}

