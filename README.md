# accel-mma84-rust
A library to use a Tessel accelerometer module with Rust

### Setup

If you want to test this module yourself, or write your own module driver in Rust [start here](https://github.com/tessel/rust-tessel).

### Install

1) 	Update your `Cargo.toml` to include a new dependency like the following:
```toml
[dependencies]
rust_accel_mma84='0.0.1'
```
2)	Make sure to run `Cargo build` to update your dependencies.

### Example
```rust
// get the tessel and accelerometer crate
use std::path::Path;
extern crate rust_tessel;
extern crate rust_accel_mma84;

fn main() {
  // initialize the accelerometer
  let port = rust_tessel::TesselPort::new(&Path::new("/var/run/tessel/port_a"));
  let mut accel = rust_accel_mma84::Accelerometer::new(port);
  accel.mode_active();

  // stream accelerometer data
  let mut vals = [0;3];
  loop {
    accel.get_acceleration(&mut vals);
    println!("x: {0:<4} y: {1:<4} z: {2:<4}", vals[0], vals[1], vals[2]);
  }
}
```

### Methods 

&#x20;<a href="#api-accel-new" name="api-accel-new">#</a> accel<b>.new</b>( port: TesselPort )

Creates a new accelerometer modules on the provided port.

&#x20;<a href="#api-accel-get_acceleration" name="api-accel-get_acceleration">#</a> accel<b>.get_acceleration</b>( vals: &mut[u16] )

Gets the acceleration from the device, fills the provided vals array with [x, y, z]. 

&#x20;<a href="#api-accel-available_output_rates" name="api-accel-available_output_rates">#</a> accel<b>.available_output_rates</b>()

Gets the available interrupt rates in Hz.

&#x20;<a href="#api-accel-set_output_rate" name="api-accel-set_output_rate">#</a> accel<b>.set_output_rate</b>( rateInHz: f32 )

Sets the output rate of the data (1.56-800 Hz).

&#x20;<a href="#api-accel-available_scale_ranges" name="api-accel-available_scale_ranges">#</a> accel<b>.available_scale_ranges</b>()

Gets the available accelerometer ranges in units of Gs.

&#x20;<a href="#api-accel-set_scale_range" name="api-accel-set_scale_range">#</a> accel<b>.set_scale_range</b>( scaleRange: u8 )

Sets the accelerometer to read up to 2, 4, or 8 Gs of acceleration (smaller range = better precision).

### Licensing

MIT or Apache 2.0, at your option
