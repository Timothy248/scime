use std::ops::{AddAssign, SubAssign};
use std::thread;
use std::sync::{Arc, RwLock, Mutex};

pub struct Scatter<S, T, I> {
    area: usize,
    eaters: Arc<RwLock<usize>>,

    function: Arc<fn(T) -> Option<S>>,
    iterator: Arc<Mutex<I>>,
    results: Arc<RwLock<Vec<S>>>
}

impl<S: 'static + Send + Sync, T: 'static + Send, I: 'static + Send + Iterator<Item = T>> Scatter<S, T, I> {
    
    pub fn new(iterator: I, area: usize, function: fn(T)->Option<S>) -> Self where I: Iterator<Item = T> { 
        let _area = if area == 0 { usize::MAX } else { area };

        let scatter = Scatter { area: _area,
            function: Arc::new(function), 
            results: Arc::new(RwLock::new(Vec::new())),
            eaters: Arc::new(RwLock::new(0)),
            iterator: Arc::new(Mutex::new(iterator)),
        };
        scatter.eat();
        return scatter
    }

    fn eat(&self) { for _ in 0..self.area {self.dispatch_eater()} } 

    fn dispatch_eater(&self) {
        let iterator = Arc::clone(&self.iterator);
        let results = Arc::clone(&self.results);
        let function = Arc::clone(&self.function);
        let eaters = Arc::clone(&self.eaters);

        let mut lock = self.eaters.write().unwrap();
        lock.add_assign(1); drop(lock);

        thread::spawn(move || {
            let mut has_data = true;

            while has_data {
                let mut lock = iterator.lock().unwrap();
                let _data = lock.next(); drop(lock);

                match _data {
                    Some(cur_data) => {
                        let result = function(cur_data);
                        match result {
                            Some(value) => { results.write().unwrap().push(value) },
                            None => ()
                        };
                    },
                    None => has_data = false
                };
            }

            eaters.write().unwrap().sub_assign(1);
        });
    }

    pub fn get_eaters(&self) -> usize{ *self.eaters.read().unwrap() }
    pub fn drain_results(&self) -> Vec<S> { self.results.write().unwrap().drain(..).collect() }

}
