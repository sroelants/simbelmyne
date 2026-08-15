### Baseline
_Rust development build (`cargo run`)_ (5 ply deep)

Executed in   32.34 secs    fish           external
   usr time   32.15 secs  229.00 micros   32.15 secs
   sys time    0.02 secs  171.00 micros    0.02 secs
   
### Release build
_Rust release build (`cargo build --release`)_ (5 ply deep)

Executed in    1.62 secs    fish           external
   usr time    1.62 secs  338.00 micros    1.62 secs
   sys time    0.00 secs  112.00 micros    0.00 secs

### Single codegen unit
_Don't split up compilation into multiple codegen units_ (5 ply deep)
https://nnethercote.github.io/perf-book/build-configuration.html#codegen-units

Executed in    1.25 secs    fish           external
   usr time    1.24 secs  299.00 micros    1.24 secs
   sys time    0.00 secs  119.00 micros    0.00 secs
### Link time optimization (LTO)
https://nnethercote.github.io/perf-book/build-configuration.html#link-time-optimization

Executed in    1.14 secs    fish           external
   usr time    1.14 secs  276.00 micros    1.14 secs
   sys time    0.00 secs  134.00 micros    0.00 secs
   
### Native instructions (SIMD et al)
`RUSTFLAGS="-C target-cpu=native" cargo build --release`

Executed in    1.10 secs    fish           external
   usr time    1.10 secs  287.00 micros    1.10 secs
   sys time    0.00 secs  141.00 micros    0.00 secs
   
### Custom allocator
Using `jemalloc` instead of the system's allocator
https://nnethercote.github.io/perf-book/build-configuration.html#alternative-allocators

Executed in    1.03 secs    fish           external
   usr time    1.03 secs  293.00 micros    1.03 secs
   sys time    0.00 secs  144.00 micros    0.00 secs
   
   
### Inline the `directions()`
Rather than allocating a bunch of vectors on each call
Executed in   24.76 secs    fish           external
   usr time   24.73 secs  658.00 micros   24.73 secs
   sys time    0.02 secs  165.00 micros    0.02 secs
