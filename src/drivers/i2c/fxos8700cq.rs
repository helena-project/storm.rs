use hil;

///
/// Device driver for the Freescale FXOS8700CQ 6-Axis accelerometer
///

// These are passed in from the device tree
#[derive(Copy)]
pub struct FXOS8700CQParams {
	pub addr: u16
}

// Define the temperature sensor device. This is valid as long as we have
// an I2C device that implements the I2C interface.
pub struct FXOS8700CQ <I2C: 'static> {
// bradjc: not sure why this doesn't work:
// pub struct FXOS8700CQ <'i2clifetime, I2C: 'i2clifetime hil::i2c::I2C> {
	i2c:  &'static mut I2C,
	addr: u16 // I2C address
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
enum FXOS8700CQRegisters {
	STATUS           = 0x00,
	OUT_X_MSB        = 0x01,
	OUT_X_LSB        = 0x02,
	OUT_Y_MSB        = 0x03,
	OUT_Y_LSB        = 0x04,
	OUT_Z_MSB        = 0x05,
	OUT_Z_LSB        = 0x06,
	F_SETUP          = 0x09,
	TRIG_CFG         = 0x0A,
	SYSMOD           = 0x0B,
	INT_SOURCE       = 0x0C,
	WHO_AM_I         = 0x0D,
	XYZ_DATA_CFG     = 0x0E,
	HP_FILTER_CUTOFF = 0x0F,
	PL_STATUS        = 0x10,
	PL_CFG           = 0x11,
	PL_COUNT         = 0x12,
	PL_BF_ZCOMP      = 0x13,
	P_L_THS_REG      = 0x14,
	A_FFMT_CFG       = 0x15,
	A_FFMT_SRC       = 0x16,
	A_FFMT_THS       = 0x17,
	A_FFMT_COUNT     = 0x18,
	TRANSIENT_CFG    = 0x1D,
	TRANSIENT_SRC    = 0x1E,
	TRANSIENT_THS    = 0x1F,
	TRANSIENT_COUNT  = 0x20,
	PULSE_CFG        = 0x21,
	PULSE_SRC        = 0x22,
	PULSE_THSX       = 0x23,
	PULSE_THSY       = 0x24,
	PULSE_THSZ       = 0x25,
	PULSE_TMLT       = 0x26,
	PULSE_LTCY       = 0x27,
	PULSE_WIND       = 0x28,
	ASLP_COUNT       = 0x29,
	CTRL_REG1        = 0x2A,
	CTRL_REG2        = 0x2B,
	CTRL_REG3        = 0x2C,
	CTRL_REG4        = 0x2D,
	CTRL_REG5        = 0x2E,
	OFF_X            = 0x2F,
	OFF_Y            = 0x30,
	OFF_Z            = 0x31,
	M_DR_STATUS      = 0x32,
	M_OUT_X_MSB      = 0x33,
	M_OUT_X_LSB      = 0x34,
	M_OUT_Y_MSB      = 0x35,
	M_OUT_Y_LSB      = 0x36,
	M_OUT_Z_MSB      = 0x37,
	M_OUT_Z_LSB      = 0x38,
	CMP_OUT_X_MSB    = 0x39,
	CMP_OUT_X_LSB    = 0x3A,
	CMP_OUT_Y_MSB    = 0x3B,
	CMP_OUT_Y_LSB    = 0x3C,
	CMP_OUT_Z_MSB    = 0x3D,
	CMP_OUT_Z_LSB    = 0x3E,
	M_OFF_X_MSB      = 0x3F,
	M_OFF_X_LSB      = 0x40,
	M_OFF_Y_MSB      = 0x41,
	M_OFF_Y_LSB      = 0x42,
	M_OFF_Z_MSB      = 0x43,
	M_OFF_Z_LSB      = 0x44,
	MAX_X_MSB        = 0x45,
	MAX_X_LSB        = 0x46,
	MAX_Y_MSB        = 0x47,
	MAX_Y_LSB        = 0x48,
	MAX_Z_MSB        = 0x49,
	MAX_Z_LSB        = 0x4A,
	MIN_X_MSB        = 0x4B,
	MIN_X_LSB        = 0x4C,
	MIN_Y_MSB        = 0x4D,
	MIN_Y_LSB        = 0x4E,
	MIN_Z_MSB        = 0x4F,
	MIN_Z_LSB        = 0x50,
	TEMP             = 0x51,
	M_THS_CFG        = 0x52,
	M_THS_SRC        = 0x53,
	M_THS_X_MSB      = 0x54,
	M_THS_X_LSB      = 0x55,
	M_THS_Y_MSB      = 0x56,
	M_THS_Y_LSB      = 0x57,
	M_THS_Z_MSB      = 0x58,
	M_THS_Z_LSB      = 0x59,
	M_THS_COUNT      = 0x5A,
	M_CTRL_REG1      = 0x5B,
	M_CTRL_REG2      = 0x5C,
	M_CTRL_REG3      = 0x5D,
	M_INT_SRC        = 0x5E,
	A_VECM_CFG       = 0x5F,
	A_VECM_THS_MSB   = 0x60,
	A_VECM_THS_LSB   = 0x61,
	A_VECM_CNT       = 0x62,
	A_VECM_INITX_MSB = 0x63,
	A_VECM_INITX_LSB = 0x64,
	A_VECM_INITY_MSB = 0x65,
	A_VECM_INITY_LSB = 0x66,
	A_VECM_INITZ_MSB = 0x67,
	A_VECM_INITZ_LSB = 0x68,
	M_VECM_CFG       = 0x69,
	M_VECM_THS_MSB   = 0x6A,
	M_VECM_THS_LSB   = 0x6B,
	M_VECM_CNT       = 0x6C,
	M_VECM_INITX_MSB = 0x6D,
	M_VECM_INITX_LSB = 0x6E,
	M_VECM_INITY_MSB = 0x6F,
	M_VECM_INITY_LSB = 0x70,
	M_VECM_INITZ_MSB = 0x71,
	M_VECM_INITZ_LSB = 0x72,
	A_FFMT_THS_X_MSB = 0x73,
	A_FFMT_THS_X_LSB = 0x74,
	A_FFMT_THS_Y_MSB = 0x75,
	A_FFMT_THS_Y_LSB = 0x76,
	A_FFMT_THS_Z_MSB = 0x77,
	A_FFMT_THS_Z_LSB = 0x78
}


impl <I2C: hil::i2c::I2C> FXOS8700CQ <I2C> {

	pub fn new (i2c_device: &mut I2C, params: FXOS8700CQParams) -> FXOS8700CQ <I2C> {
		// return
		FXOS8700CQ {
			i2c: i2c_device,
			addr: params.addr
		}
	}

	// Returns true if 0xC7 is correctly read, false if not
	pub fn read_whoami_sync (&mut self) -> bool {

		let mut buf: [u8; 3] = [0; 3];

		self.i2c.enable();

		// Start by enabling the sensor
		buf[0] = FXOS8700CQRegisters::WHO_AM_I as u8;
		self.i2c.write_sync(self.addr, &buf[0..1]);

		self.i2c.read_sync(self.addr, &mut buf[0..1]);

		if buf[0] == 0xC7 {
			return true;
		}
		false

		// let mut sensor_voltage: i16;
		// let mut die_temp: i16;

		// // Now set the correct register pointer value so we can issue a read
		// // to the sensor voltage register
		// buf[0] = TMP006Registers::SensorVoltage as u8;
		// self.i2c.write_sync(self.addr, &buf[0..1]);

		// // Now read the sensor reading
		// self.i2c.read_sync(self.addr, &mut buf[0..2]);
		// sensor_voltage = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;

		// // Now move the register pointer to the die temp register
		// buf[0] = TMP006Registers::LocalTemperature as u8;
		// self.i2c.write_sync(self.addr, &buf[0..1]);

		// // Now read the 14bit die temp
		// self.i2c.read_sync(self.addr, &mut buf[0..2]);
		// die_temp = (((buf[0] as u16) << 8) | buf[1] as u16) as i16;
		// // Shift to the right to make it 14 bits (this should be a signed shift)
		// // The die temp is is in 1/32 degrees C.
		// die_temp = die_temp >> 2;

		// // return
		// die_temp
	}

}
