# scime

## Usage:
```rust
use scime::data::Scatter;

// area: thread count
// queue_limit: amount of data in the queue allowed
let scatter = Scatter::new(10, 10, |data| {
  /* task to parallelize */
});

let id = scatter.feed(data);

// results are available as soon as the data has been processed
let result = scatter.drain_results().get(id);
```
