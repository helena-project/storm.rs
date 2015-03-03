/*
 * AESA (Advanced Encryption Standard [AES-128]) Support for Atmel SAM4L
 *
 * Section 18 of datasheet
 */

use core::prelude::SliceExt;
use core::intrinsics;

use hil;
use sam4l;

// Table 18-2
#[repr(C, packed)]
#[allow(dead_code)]
struct AESARegisters {
    control:                usize,
    mode:                   usize,
    data_buffer_pointer:    usize,
    status:                 usize,  // RO
    interrupt_enable:       usize,  // WO
    interrupt_disable:      usize,  // WO
    interrupt_mask:         usize,  // RO
    // Access to keyN, ivN registers _must_ use full-word ops (e.g. no strh)
    key0:                   u32,  // WO
    key1:                   u32,  // WO
    key2:                   u32,  // WO
    key3:                   u32,  // WO
    // n.b. key4-7 aren't usable b/c this core only has an AES-128 engine
    key4:                   u32,  // WO
    key5:                   u32,  // WO
    key6:                   u32,  // WO
    key7:                   u32,  // WO
    iv0:                    u32,  // WO
    iv1:                    u32,  // WO
    iv2:                    u32,  // WO
    iv3:                    u32,  // WO
    input_data:            [u32; 4],  // WO
    output_data:           [u32; 4],  // RO
    drng_seed:              u32,  // WO
    _reserved0:            [usize; 33], // 3+4*7+2
    parameter:              usize,  // RO
    version:                usize   // RO
}

// The addresses in memory (7.1 of datasheet) of AESA peripheral
const AESA_BASE_ADDR: usize = 0x400B0000;

// Only one AESA
pub enum AESALocation {
    AESA
}

// Parameters from the platform device tree
pub struct AESAParams {
    pub location: AESALocation
}

pub struct AESADevice {
    registers: &'static mut AESARegisters,  // Pointer to AESA reg's in memory
    clock: sam4l::pm::Clock,
}

// Need to implement the `new` function on the TRNG device as a constructor.
// This gets called from the device tree.
impl AESADevice {
    pub fn new (params: AESAParams) -> AESADevice {
        AESADevice {
            registers: unsafe { intrinsics::transmute(AESA_BASE_ADDR) },
            clock: sam4l::pm::Clock::HSB(sam4l::pm::HSBClock::AESA)
        }
    }

    fn enable (&mut self) {
        sam4l::pm::enable_clock(self.clock);
        volatile!(self.registers.control = 0x1);

        // Require MMIO mode, at least for now, as there's no DMA infrastructure
        self.set_mode_MMIO();
    }

    fn disable (&mut self) {
        volatile!(self.registers.control = 0x0);
        sam4l::pm::disable_clock(self.clock);
    }

    fn reset (&mut self) {
        volatile_bitset!(self.registers.control, 0x1 << 8);
    }

    /// If the cipher mode needs to behave differently for a new message
    /// (most need to go back to the IV), this indicates message boundary
    fn new_message (&mut self) {
        // Must |= to preserve enable bit
        volatile_bitset!(self.registers.control, 0x1 << 2);
    }

    /// By default, the crypto engine lazily generates round keys.
    /// On the encryption path, it can keep up with demand and this causes no
    /// latency. Decryption on a fresh key needs the last round key in the first
    /// round of decryption, which adds extra latency to the first decryption
    /// call. If you need deterministic latency, you can force generation of the
    /// round keys in advance with this method.
    fn generate_expanded_key (&mut self) {
        // Must |= to preserve enable bit
        volatile_bitset!(self.registers.control, 0x1 << 1);
    }


    fn set_mode_encrypt (&mut self) {
        volatile_bitset!(self.registers.mode, 0x1);
    }

    fn set_mode_decrypt (&mut self) {
        volatile_bitclear!(self.registers.mode, 0x1);
    }

    fn set_mode_DMA (&mut self) {
        volatile_bitset!(self.registers.mode, 0x8);
    }

    fn set_mode_MMIO (&mut self) {
        volatile_bitclear!(self.registers.mode, 0x8);
    }


    /* The excess of functions below feels a bit much, but at the same time
     * it's hard to understand what's happening when you just return a
     * (usize, usize) tuple [I wish I could name return params..]. At the
     * moment I feel a bit like a Java developer (and not in a good way) with
     * the number of very specific micro-functions I'm writing, so I'm hoping
     * a better style will emerge with time */

    /// Returns (input_index, output_index)
    fn get_data_buffer_indicies (&mut self) -> (usize, usize) {
        let reg = volatile!(self.registers.data_buffer_pointer);
        ((reg >> 4) & 0x3, reg & 0x3)
    }

    fn get_input_data_buffer_index (&mut self) -> usize {
        self.get_data_buffer_indicies().0
    }

    fn get_output_data_buffer_index (&mut self) -> usize {
        self.get_data_buffer_indicies().1
    }

    fn set_data_buffer_indicies (&mut self, input_index: usize, output_index: usize) {
        volatile!(self.registers.data_buffer_pointer =
                  ((input_index & 0x3) << 4) |
                  ((output_index & 0x3) << 0)
                  );
    }

    fn set_input_data_buffer_index (&mut self, input_index: usize) {
        // Rust thinks there's a *self sharing issue w/out the `temp`
        let temp = self.get_output_data_buffer_index();
        self.set_data_buffer_indicies(input_index, temp);
    }

    fn set_output_data_buffer_index (&mut self, output_index: usize) {
        let temp = self.get_input_data_buffer_index();
        self.set_data_buffer_indicies(temp, output_index);
    }

    /// Returns: (input_ready, output_ready)
    ///  * `input_ready` - Set if module can accept input
    ///  * `output_ready` - Set when ready, cleared by HW when all is read
    fn get_status (&mut self) -> (bool, bool) {
        let reg = volatile!(self.registers.status);
        ((reg >> 16) == 0x1, reg == 0x1)
    }

    fn enable_interrupts (&mut self,
                          input_buffer_ready: bool,
                          output_data_ready: bool) {
        volatile!(self.registers.interrupt_enable =
                  (output_data_ready as usize) |
                  ((input_buffer_ready as usize) << 16)
                  );
    }

    fn disable_interrupts (&mut self,
                          input_buffer_ready: bool,
                          output_data_ready: bool) {
        volatile!(self.registers.interrupt_disable =
                  (output_data_ready as usize) |
                  ((input_buffer_ready as usize) << 16)
                  );
    }

    /// Returns: (input_buffer_ready, output_data_ready)
    /// Booleans indicating whether interrupts for these events are enabled
    fn query_interrupts (&mut self) -> (bool, bool) {
        let reg = volatile!(self.registers.interrupt_mask);
        ((reg >> 16) == 0x1, reg == 0x1)
    }

    fn _set_key (&mut self, key: &[u32; 4]) {
        // These registers must be written using 32-bit operations. If Rust
        // exposed llvm.memset.p0i32 or similar we could use that instead

        /* So, rust's asm! macro doesn't actually work either. We'll just
         * write these as simple operations and hope the compiler doesn't
         * do anything silly
        unsafe {
            asm!("\
str %0, [%1], #4\n\t\
str %2, [%1], #4\n\t\
str %3, [%1], #4\n\t\
str %4, [%1], #4\n\t"
            : /* no outputs */
            : "r"(key[0]),
              "r"(self.registers.key0),
              "r"(key[1])
              "r"(key[2])
              "r"(key[3])
            : "%1" /* clobbers addr */
            : "volatile"
            );
        }
        */

        volatile!(self.registers.key0 = key[0]);
        volatile!(self.registers.key1 = key[1]);
        volatile!(self.registers.key2 = key[2]);
        volatile!(self.registers.key3 = key[3]);
    }

    fn _set_iv (&mut self, iv: &[u32; 4]) {
        // Like set_key, must use 32-bit ops
        volatile!(self.registers.iv0 = iv[0]);
        volatile!(self.registers.iv1 = iv[1]);
        volatile!(self.registers.iv2 = iv[2]);
        volatile!(self.registers.iv3 = iv[3]);
    }

    fn set_drng_seed (&mut self, seed: u32) {
        // Like set_key, must use 32-bit ops
        volatile!(self.registers.drng_seed = seed);
    }
}

impl hil::crypto::Symmetric128 for AESADevice {
    fn set_mode (&mut self, mode: hil::crypto::SymmetricMode) {
        self.enable();

        let mask = match mode {
            hil::crypto::SymmetricMode::ElectronicCodeBook => 0,
            hil::crypto::SymmetricMode::CipherBlockChaining => 1,
            hil::crypto::SymmetricMode::CipherFeedback => 2,
            hil::crypto::SymmetricMode::OutputFeedback => 3,
            hil::crypto::SymmetricMode::Counter => 4
        };
        let mut mode = volatile!(self.registers.mode);
        mode &= !0x770; // Clear CFBS, OPMODE
        mode |= mask << 4;
        volatile!(self.registers.mode = mode);
    }

    fn set_key (&mut self, key: &[u32; 4]) {
        self.enable();
        self._set_key(key);
    }

    fn set_iv (&mut self, iv: &[u32; 4]) {
        self.enable();
        self._set_iv(iv);
    }

    fn encrypt_sync(&mut self, plaintext: &[u32], ciphertext: &mut[u32]) {
        self.enable();

        self.set_mode_encrypt();
        self.set_data_buffer_indicies(0, 0);
        self.new_message();

        let mut idx = 3;
        while idx < plaintext.len() && idx < ciphertext.len() {
            while !self.get_status().0 {
                // busy-wait
                ;
            }

            self.registers.input_data[0] = plaintext[idx-3];
            self.registers.input_data[1] = plaintext[idx-2];
            self.registers.input_data[2] = plaintext[idx-1];
            self.registers.input_data[3] = plaintext[idx-0];

            while !self.get_status().1 {
                // busy-wait
                ;
            }

            ciphertext[idx-3] = self.registers.output_data[0];
            ciphertext[idx-2] = self.registers.output_data[1];
            ciphertext[idx-1] = self.registers.output_data[2];
            ciphertext[idx-0] = self.registers.output_data[3];

            idx += 4;
        }

        self.disable();
    }

    fn decrypt_sync(&mut self, ciphertext: &[u32], plaintext: &mut[u32]) {
        self.enable();

        self.set_mode_decrypt();
        self.set_data_buffer_indicies(0, 0);
        self.new_message();

        let mut idx = 3;
        while idx < plaintext.len() && idx < ciphertext.len() {
            while !self.get_status().0 {
                // busy-wait
                ;
            }

            self.registers.input_data[0] = ciphertext[idx-3];
            self.registers.input_data[1] = ciphertext[idx-2];
            self.registers.input_data[2] = ciphertext[idx-1];
            self.registers.input_data[3] = ciphertext[idx-0];

            while !self.get_status().1 {
                // busy-wait
                ;
            }

            plaintext[idx-3] = self.registers.output_data[0];
            plaintext[idx-2] = self.registers.output_data[1];
            plaintext[idx-1] = self.registers.output_data[2];
            plaintext[idx-0] = self.registers.output_data[3];

            idx += 4;
        }

        self.disable();
    }
}
