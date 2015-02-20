use hil;

use core::prelude::SliceExt;

///
/// Device driver for the TI TMP006 contactless temperature sensor
///

// These are passed in from the device tree
#[derive(Copy)]
pub struct TMP006Params {
	pub addr: u16
}

// Define the temperature sensor device. This is valid as long as we have
// an I2C device that implements the I2C interface.
pub struct TMP006 <I2C: hil::i2c::I2C> {
	i2c:  I2C,
	addr: u16 // I2C address
}

#[allow(dead_code)]
enum TMP006Registers {
	SensorVoltage    = 0x00,
	LocalTemperature = 0x01,
	Configuration    = 0x02,
	ManufacturerID   = 0xFE,
	DeviceID         = 0xFF
}


impl <I2C: hil::i2c::I2C> TMP006 <I2C> {

	pub fn new (i2c_device: I2C, params: TMP006Params) -> TMP006<I2C> {
		// return
		TMP006 {
			i2c: i2c_device,
			addr: params.addr
		}
	}

	/// Returns a temperature reading by doing all synchronous (blocking) i2c
	/// calls.
	pub fn read_sync (&mut self) -> i16 {

		let mut buf: [u8; 3] = [0; 3];
		let mut config: u16;

		self.i2c.enable();

		// Start by enabling the sensor
		config = 0x7 << 12;
		buf[0] = TMP006Registers::Configuration as u8;
		buf[1] = ((config & 0xFF00) >> 8) as u8;
		buf[2] = (config & 0x00FF) as u8;
		self.i2c.write_sync(self.addr, &buf);

		// Now wait until a sensor reading is ready
		loop {
			self.i2c.read_sync(self.addr, &mut buf[1..2]);
			Check the DRDY ready bit in the config register
			if (buf[1] & 0x80) == 0x80 {
				break;
			}
		}

		// Now set the correct register pointer value so we can issue a read
		// to the sensor voltage register
		buf[0] = TMP006Registers::SensorVoltage as u8;
		self.i2c.write_sync(self.addr, &buf[0..1]);

		// Now read the sensor reading
		self.i2c.read_sync(self.addr, &mut buf[0..2]);

		let ret: i16 = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;

		ret
	}

}
