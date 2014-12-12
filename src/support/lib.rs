// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Support functions currently required by the linker for bare-metal targets.

#![crate_name = "support"]
#![crate_type = "rlib"]
#![feature(asm, lang_items)]
#![no_std]

extern crate core;

use core::fmt::Arguments;
use core::intrinsics::set_memory;

#[doc(hidden)]
#[cfg(test)]
#[no_stack_check]
#[no_mangle]
pub extern fn breakpoint() { unimplemented!() }

/// Call the debugger.
#[cfg(not(test))]
#[no_stack_check]
#[no_mangle]
pub extern fn breakpoint() {
  unsafe { asm!("bkpt") }
}

/// Call the debugger and halts execution.
#[no_stack_check]
#[no_mangle]
pub extern fn abort() -> ! {
  breakpoint();
  loop {}
}

#[doc(hidden)]
#[no_stack_check]
#[no_mangle]
pub extern fn __aeabi_unwind_cpp_pr0() {
  abort();
}

#[doc(hidden)]
#[no_stack_check]
#[no_mangle]
pub extern fn __aeabi_unwind_cpp_pr1() {
  abort();
}

// TODO(bgamari): This is only necessary for exception handling and
// can be removed when we have this issue resolved.
#[doc(hidden)]
#[no_stack_check]
#[no_mangle]
pub extern fn __aeabi_memset(dest: *mut u8, size: uint, value: u32) {
  unsafe {
    set_memory(dest, value as u8, size);
  }
}

#[doc(hidden)]
#[no_stack_check]
#[no_mangle]
pub extern fn get_eit_entry() {
  abort();
}

#[cfg(not(test))]
#[inline(always)]
/// NOP instruction
pub fn nop() {
  unsafe { asm!("nop" :::: "volatile"); }
}

#[cfg(test)]
/// NOP instruction (mock)
pub fn nop() {
}

#[cfg(not(test))]
#[inline(always)]
/// WFI instruction
pub fn wfi() {
    unsafe { asm!("wfi" :::: "volatile"); }
}

#[cfg(test)]
/// WFI instruction (mock)
pub fn wfi() {
}

#[cfg(not(test))]
#[lang="stack_exhausted"]
extern fn stack_exhausted() {}

#[cfg(not(test))]
#[lang="eh_personality"]
extern fn eh_personality() {}

#[cfg(not(test))]
#[lang="begin_unwind"]
extern fn begin_unwind() {}

#[cfg(not(test))]
#[lang="panic_fmt"]
#[no_mangle]
extern fn rust_begin_unwind(_fmt: &Arguments,
    _file_line: &(&'static str, uint)) -> ! {
  loop { }
}

