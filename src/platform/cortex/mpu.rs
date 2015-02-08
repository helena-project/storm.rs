use core::intrinsics::{volatile_load, volatile_store};

pub const BASE_ADDRESS : usize = 0xE000ED90;

/// Memory Protection Unit
#[allow(dead_code,missing_copy_implementations)]
pub struct MPU {
    mpu_type: usize,
    control: usize,
    region_number: usize,
    region_base_addr: usize,
    region_att_size: usize,
}

impl MPU {
    /// Enables the MPU
    ///
    /// # Arguments
    ///
    /// * `hf_enable` - Enable the MPU during hard fault.
    ///
    /// * `default_map` - Enable the default memory map. When
    /// enabled, any access by privileged software that does not address an
    /// enabled memory region behaves as defined by the default memory map (as
    /// described in[^1]).
    ///
    /// [^1]: Cortex-M4 Device Generic User Guide, Page 2-12
    pub fn enable(&mut self, hf_enable: bool, default_map: bool) {
        let val = 1 | ((hf_enable as usize) << 1)
                    | ((default_map as usize) << 2);
        unsafe {
            volatile_store(&mut self.control, val);
        }
    }

    /// Disables the MPU
    pub fn disable(&mut self) {
        unsafe {
            volatile_store(&mut self.control, 0);
        }
    }
}

/// Regions 0-7 of the MPU
#[derive(Copy)]
pub enum RegionNum {
    R0, R1, R2, R3, R4, R5, R6, R7
}

/// A single MPU region
#[allow(missing_copy_implementations)]
pub struct Region {
    mpu: &'static mut MPU,
    region_num: RegionNum
}

impl Region {
    /// Enables the Region
    ///
    /// _Non re-entrant_
    pub fn enable(&mut self) {
        unsafe {
            // set active region
            let num = volatile_load(&self.region_num) as usize;
            volatile_store(&mut self.mpu.region_number, num);

            // set enable bit
            let val = volatile_load(&self.mpu.region_att_size) | 1;
            volatile_store(&mut self.mpu.region_att_size, val);
        }
    }

    /// Disables the Region
    ///
    /// _Non re-entrant_
    pub fn disable(&mut self) {
        unsafe {
            // set active region
            let num = volatile_load(&self.region_num) as usize;
            volatile_store(&mut self.mpu.region_number, num);

            // clear enable bit
            let val = volatile_load(&self.mpu.region_att_size) & !1;
            volatile_store(&mut self.mpu.region_att_size, val);
        }
    }

    /// Sets the base address of the Region.
    ///
    /// # Arguments
    ///
    /// * `addr` - The base address. The bottom `N` bits of `addr` are ignored
    /// for `N = Log2(Region size in bytes)`. As a result, the base addressed
    /// is always aligned to the size of the region.
    pub fn set_address(&mut self, addr: usize) {
        let region_valid = (self.region_num as usize) & 0b111 | 0b1000;
        let val = (addr & !(0b1111)) | region_valid;
        unsafe { volatile_store(&mut self.mpu.region_base_addr, val) };
    }

    /// Set the attribute-and-size register in one write.
    ///
    /// | Bits    | Function                        | Name   |
    /// |:--------|:--------------------------------|-------:|
    /// | [28]    | Disable instruction fetches     | XN     |
    /// | [26:24] | Access Permission Field         | AP     |
    /// | [21:19] |                                 | TEX    |
    /// | [18]    | Sharable bit                    | S      |
    /// | [17]    |                                 | C      |
    /// | [16]    |                                 | B      |
    /// | [15:8]  | Subregion disable bits          | SRD    |
    /// | [5:1]   | Exponent for size of the region | SIZE   |
    /// | [0]     | Region enable bit               | ENABLE |
    ///
    /// # Arguments
    ///
    /// * `att_size` - The attribute and size register has seven fields:
    ///
    /// * `enable` - Enable or disable this region.
    pub fn set_att_size(&mut self, att_size: usize, enable: bool) {
        let val = (att_size & !1) | enable as usize;
        unsafe {
            // set active region
            let num = volatile_load(&self.region_num) as usize;
            volatile_store(&mut self.mpu.region_number, num);

            // set attribute and size register
            volatile_store(&mut self.mpu.region_att_size, val)
        };
    }
}

