# scime
Scime : distribute tasks autonomous

## Usage:
```rust
use scime::Scatter;

let scatter = Scatter::new(10, 10, |data| {
  /* task to scatter */
});

let id = scatter.feed(data);

/* results are available when the threads have processed all data */
let result = scatter.drain_results().get(id);
```
