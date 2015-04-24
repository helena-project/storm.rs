use hil;

///
/// Device driver for the TI TMP006 contactless temperature sensor
///

// These are passed in from the device tree
#[derive(Copy)]
pub struct TMP006Params;

// Define the temperature sensor device. This is valid as long as we have
// an I2C device that implements the I2C interface.
pub struct TMP006 <I2C: hil::i2c::I2C> {
	i2c:  I2C
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
			i2c: i2c_device
		}
	}

	/// Returns a temperature reading by doing all synchronous (blocking) i2c
	/// calls.
	/// This is not a good function to call as it sits in a loop asking the
	/// TMP006 sensor "do you have a reading yet?" via I2C.
	pub fn read_sync (&mut self) -> i16 {

		let mut buf: [u8; 3] = [0; 3];
		let mut config: u16;

		self.i2c.enable();

		// Start by enabling the sensor
		config = 0x7 << 12;
		buf[0] = TMP006Registers::Configuration as u8;
		buf[1] = ((config & 0xFF00) >> 8) as u8;
		buf[2] = (config & 0x00FF) as u8;
		self.i2c.write_sync(&buf);

		// Now wait until a sensor reading is ready
		loop {
			self.i2c.read_sync(&mut buf[0..2]);
			// Check the DRDY ready bit in the config register
			if (buf[1] & 0x80) == 0x80 {
				break;
			}
		}

		let mut sensor_voltage: i16;
		let mut die_temp: i16;

		// Now set the correct register pointer value so we can issue a read
		// to the sensor voltage register
		buf[0] = TMP006Registers::SensorVoltage as u8;
		self.i2c.write_sync(&buf[0..1]);

		// Now read the sensor reading
		self.i2c.read_sync(&mut buf[0..2]);
		sensor_voltage = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;

		// Now move the register pointer to the die temp register
		buf[0] = TMP006Registers::LocalTemperature as u8;
		self.i2c.write_sync(&buf[0..1]);

		// Now read the 14bit die temp
		self.i2c.read_sync(&mut buf[0..2]);
		die_temp = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;
		// Shift to the right to make it 14 bits (this should be a signed shift)
		// The die temp is is in 1/32 degrees C.
		die_temp = die_temp >> 2;

		// return
		die_temp
	}

}
