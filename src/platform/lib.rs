#![crate_name = "platform"]
#![crate_type = "rlib"]
#![no_std]
#![feature(asm,plugin,core,concat_idents,no_std)]
#![plugin(plugins)]

extern crate core;
extern crate hil;

macro_rules! volatile {
    ($item:expr) => ({
        use core::intrinsics::volatile_load;
        unsafe { volatile_load(&$item) }
    });

    ($item:ident = $value:expr) => ({
        use core::intrinsics::volatile_store;
        unsafe { volatile_store(&mut $item, $value); }
    });
}

macro_rules! volatile_bitset {
    ($item:expr, $value:expr) => ({
        use core::intrinsics::volatile_load;
        use core::intrinsics::volatile_store;
        unsafe { volatile_store(&mut $item, volatile_load(&$item) | $value); }
    });
}

macro_rules! volatile_bitclear {
    ($item:expr, $value:expr) => ({
        // Bitwise negation is Rust is ! instead of ~, because they just had
        // to be different. [fwiw, I agree with the underlying premise that
        // with Rust's type system there's no reason for a logical not operator
        // to exist---comparisons to bool should cast to bool explicitly to be
        // unambiguous---however, they should have used ~ as the operator to
        // preserve consistency with existing languages]
        use core::intrinsics::volatile_load;
        use core::intrinsics::volatile_store;
        unsafe { volatile_store(&mut $item, volatile_load(&$item) & !$value); }
    });
}

mod std {
    pub use core::*;
}

pub mod sam4l;
pub mod cortex;
