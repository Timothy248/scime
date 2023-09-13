use std::ops::{AddAssign, SubAssign};
use std::{collections::HashMap, thread};
use std::sync::{Arc, RwLock, Mutex, atomic::AtomicUsize};

pub struct Scatter<S, T> {
    area: usize,
    queue_limit: usize,
    
    eaters: Arc<RwLock<usize>>,

    function: Arc<fn(T) -> Option<S>>,
    current_id: AtomicUsize,

    data: Arc<Mutex<Vec<(usize, T)>>>,
    results: Arc<RwLock<HashMap<usize, S>>>
}

impl<S: 'static + Send + Sync, T: 'static + Send> Scatter<S, T> {
    
    pub fn new(area: usize, queue_limit: usize, function: fn(T)->Option<S>) -> Self { 
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

    fn eat(&self) { if self.get_eaters() < self.area { self.dispatch_eater() } }

    fn dispatch_eater(&self) {
        let data = Arc::clone(&self.data);
        let results = Arc::clone(&self.results);
        let function = Arc::clone(&self.function);
        let eaters = Arc::clone(&self.eaters);

        let mut lock = self.eaters.write().unwrap();
        lock.add_assign(1); drop(lock);

        thread::spawn(move || {
            let mut has_data = true;

            while has_data {
                let mut lock = data.lock().unwrap();
                let _data = lock.pop(); drop(lock);

                match _data {
                    Some((cur_id, cur_data)) => {
                        let result = function(cur_data);
                        match result {
                            Some(value) => { results.write().unwrap().insert(cur_id, value); },
                            None => ()
                        };
                    },
                    None => has_data = false
                };
            }

            eaters.write().unwrap().sub_assign(1);
        });
    }

    pub fn feed(&self, data: T) -> Option<usize> {
        let mut lock = self.data.lock().unwrap();
        if lock.len() >= self.queue_limit { return None }

        let id = self.current_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        lock.push((id, data)); drop(lock);
        
        self.eat();

        return Some(id);
    }

    pub fn get_eaters(&self) -> usize{ *self.eaters.read().unwrap() }
    pub fn drain_results(&self) -> HashMap<usize, S> { self.results.write().unwrap().drain().collect() }
    pub fn get_queue_length(&self) -> usize { self.data.lock().unwrap().len() }

}