#![allow(dead_code)]
extern crate unix_socket;

extern crate rust_tessel;
use rust_tessel::TesselPort;
use rust_tessel::Action;

// Register addresses
const VALID_CHIP_ID: u8 =         42;
const OUT_X_MSB: u8 =             0x01;
const CHIP_ID_REG: u8 =           0x0D;
const XYZ_DATA_CFG:u8 =           0x0E;
const I2C_ADDRESS: u8 =           0x1D;
const CTRL_REG1: u8 =             0x2A;
const CTRL_REG4: u8 =             0x2D;

// Default acceleromter values
const NUM_OUTPUT_RATES: usize =   8;
const DEFAULT_OUTPUT_RATE: f32 =  12.5;
const NUM_SCALE_RANGES: usize =   3;
const DEFAULT_SCALE_RANGE: u8 =   2;

// The Accelerometer itself
pub struct Accelerometer {
  port: TesselPort,
  output_rate: f32,
  scale_range: u8,
}

impl Accelerometer {

  // Creates a new Accelerometer 
  pub fn new(port: TesselPort) -> Accelerometer {
    let mut accelerometer = Accelerometer {
      port : port,
      output_rate : DEFAULT_OUTPUT_RATE,
      scale_range : DEFAULT_SCALE_RANGE,
    };

    match accelerometer.get_chip_id() {
      Ok(_) => accelerometer,
      Err(e) => panic!(e),
    }
  }

  // Sets the MMA8452 to active mode. Needs to be in this mode to output data
  pub fn mode_active(&mut self) {
    // get the current mode
    let reg_current_state = self.read_register(CTRL_REG1);
    // write the current state, panic if it failed to read
    match reg_current_state {
      Ok(_) => {
        self.port.run(&mut [
          Action::start(0x0 | I2C_ADDRESS << 1),
          Action::tx(&[CTRL_REG1, reg_current_state.unwrap() | 0x01]),
          Action::stop(),
        ]).unwrap();
      },
      Err(e) => panic!(e),
    }
  }

  // Sets the MMA8452 to standby mode. Needs to be in this mode to change resgisters
  pub fn mode_standby(&mut self) {
    // get the current mode
    let reg_current_state = self.read_register(CTRL_REG1);
    // write the current state, panic if it failed to read
    match reg_current_state {
      Ok(_) => {
        self.port.run(&mut [
          Action::start(0x0 | I2C_ADDRESS << 1),
          Action::tx(&[CTRL_REG1, reg_current_state.unwrap() & !(0x01)]),
          Action::stop(),
        ]).unwrap();
      },
      Err(e) => panic!(e),
    }
  }

  // Write a single byte to the register
  pub fn write_register(&mut self, addr: u8, data: u8) -> Result<(), &'static str> {
    // write the byte to the register
    let res = self.port.run(&mut [
      Action::enable_i2c(),
      Action::start(0x0 | addr << 1),
      Action::tx(&[data]),
      Action::stop(),
    ]);
    // if there was an error report it, otherwise return nuffing
    match res {
      Ok(_) => Ok(()),
      Err(_) => Err("Unable to write to register"),
    }
  }

  // Read a single byte
  pub fn read_register(&mut self, addr: u8) -> Result<u8, &'static str> {
    let mut read_data = [0]; 
    // read from the provided register addr
    let res = self.port.run(&mut [
      Action::enable_i2c(),
      Action::start(0x0 | I2C_ADDRESS << 1),
      Action::tx(&[addr]),
      Action::start(0x1 | I2C_ADDRESS << 1),
      Action::rx(&mut read_data),
      Action::stop(),
    ]);
    // if there was an error report it, otherwise return the read data
    match res {
      Ok(_) => Ok(read_data[0]),
      Err(_) => Err("Unable to read register"),
    }
  }

  // Gets the acceleration from the device, outputs as array [x, y, z]
  pub fn get_acceleration(&mut self, accel: &mut[u16]) {
    let mut accel_raw = [0;6];

    self.port.run(&mut [
      Action::start(0x0 | I2C_ADDRESS << 1),
      Action::tx(&[OUT_X_MSB]),
      Action::start(0x1 | I2C_ADDRESS << 1),
      Action::rx(&mut accel_raw),
      Action::stop(),
    ]).unwrap();

    for i in 0..3 {
      let mut g_count: u16 = (((accel_raw[i*2] as u16) << 8) | (accel_raw[(i*2)+1]) as u16) as u16;
      g_count = g_count >> 4;

      if accel_raw[i*2] > 0x7F {
        g_count = 1 + 0xFFF - g_count;
      }

      g_count / ((1<<12)/(2*2*(self.scale_range as u16)));

      accel[i] = g_count;
    }
  }

  // Get the id of the chip
  pub fn get_chip_id(&mut self) -> Result<u8, &'static str>  {
    let mut chip_id = [0]; 

    let res = self.port.run(&mut [
      Action::enable_i2c(),
      Action::start(0x0 | I2C_ADDRESS << 1),
      Action::tx(&[CHIP_ID_REG]),
      Action::start(0x1 | I2C_ADDRESS << 1),
      Action::rx(&mut chip_id),
      Action::stop(),
    ]);

    match res {
      Ok(_) => Ok(chip_id[0]),
      Err(_) => Err("Unable to read chip id"),
    }
  }

  // Gets the available interrupt rates in Hz
  pub fn available_output_rates(&mut self) -> [f32; NUM_OUTPUT_RATES] {
    [800.0, 400.0, 200.0, 100.0, 50.0, 12.5, 6.25, 1.56]
  }

  // Gets the available accelerometer ranges (in units of Gs)
  pub fn available_scale_ranges(&mut self) -> [u8; NUM_SCALE_RANGES] {
    [2, 4, 8]
  }

  // Gets the closest rate to the one provided
  pub fn get_closest_output_rate(&mut self, requested_rate: f32) -> f32 {
    // if the requested rate is less than or equal to zero then return 0
    if requested_rate <= 0.0 { return 0.0 }
    // get the available output rates
    let available: [f32; NUM_OUTPUT_RATES] = self.available_output_rates();
    // get the first available rate less than or equal to requested
    for i in 0..NUM_OUTPUT_RATES {
      if available[i] <= requested_rate {
        return available[i]
      }
    }
    // if the rate is less than the smallest available return the smallest available
    available[NUM_OUTPUT_RATES-1]
  }

  // Sets the output rate
  pub fn set_output_rate(&mut self, hz: f32) -> Result<(), &'static str> {

    // set the accelerometer to be in standby
    self.mode_standby();

    // set our rate to the closest available rate to the one provided
    self.output_rate = self.get_closest_output_rate(hz);

    // get the index of the closest available rate
    let idx = self.available_output_rates().iter().position(|x| *x == self.output_rate).unwrap();

    // read the register containing the output rate
    let mut reg_current_state = try!(self.read_register(CTRL_REG1));

    // clear the three bits of output rate control (0b11000111 = 199)
    reg_current_state &= 199;

    // move the binary rep into place (bits 3:5)
    if idx != 0 { reg_current_state |= (idx as u8) << 3; }

    // write the new value to the output rate register
    try!(self.write_register(CTRL_REG1, reg_current_state));

    // set the accelerometer back to active
    self.mode_active();

    // everything went fine return okay result
    Ok(())
  }

  // Sets the scale range
  pub fn set_scale_range(&mut self, mut scale_range: u8) -> Result<(), &'static str> {

    // make sure scale range is maxed out at 8
    if scale_range > 8 { scale_range = 8; }

    // set our scale range to the provided one
    self.scale_range = scale_range;

    // neat trick, see page 22. 00 = 2G, 01 = 4G, 10 = 8G
    scale_range >>= 2;

    // set the accelerometer to be in standby
    self.mode_standby();

    // write the new value to the scale range register
    try!(self.write_register(XYZ_DATA_CFG, scale_range));

    // set the accelerometer back to active
    self.mode_active();

    // everything went fine return okay result
    Ok(())
  }

}


#[cfg(test)]
mod test {

  use super::*;

  #[test]
  fn test_new() {
    // no need to test socket connection here (read register does this)
    // let port = super::tessel::TesselPort::new(&super::std::old_path::Path::new("/var/run/tessel/port_a"));
    // let accel = Accelerometer::new(port);
    // // make sure the instance vars are set corectly
    // assert_eq!(accel.output_rate, super::DEFAULT_OUTPUT_RATE);
    // assert_eq!(accel.scale_range, super::DEFAULT_SCALE_RANGE);
  }

  #[test]
  fn test_mode_active() {
    // let port = super::tessel::TesselPort::new(&super::std::old_path::Path::new("/var/run/tessel/port_a"));
    // let mut accel = Accelerometer::new(port);
    // // make sure that setting mode to active does not panic
    // accel.mode_active();
  }

  #[test]
  fn test_mode_standby() {
    // let port = super::tessel::TesselPort::new(&super::std::old_path::Path::new("/var/run/tessel/port_a"));
    // let mut accel = Accelerometer::new(port);
    // // make sure that setting mode to standby does not panic
    // accel.mode_standby();
  }

  #[test]
  fn test_write_and_read_register() {
    // let port = super::tessel::TesselPort::new(&super::std::old_path::Path::new("/var/run/tessel/port_a"));
    // let mut accel = Accelerometer::new(port);
    // let mut res_read = accel.read_register(super::CTRL_REG1);
    // TODO: write test
  }

  #[test]
  fn test_get_acceleration() {
    // assert!(false);
  }

  #[test]
  fn test_get_chip_id() {
    // assert!(false);
  }

  #[test]
  fn test_get_closest_output_rate() {
    // assert!(false);
  }

  #[test]
  fn test_set_output_rate() {
    // assert!(false);
  }

  #[test]
  fn test_set_scale_range() {
    // assert!(false);
  }

}

