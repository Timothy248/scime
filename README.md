# scime

## Usage:
```rust
use scime::Scatter;

// area - thread count
// queue_limit - amount of overflowing data allowed
let scatter = Scatter::new(10, 10, |data| {
  /* task to scatter */
});

let id = scatter.feed(data);

/* results are available when the threads have processed given data */
let result = scatter.drain_results().get(id);
```
