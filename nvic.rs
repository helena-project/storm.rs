use intrinsics;

#[repr(C, packed)]
struct Nvic {
    iser : [u32, ..28]
}

pub fn enable(int : uint) {
    let nvic_addr : u32 = 0xe000e100;
    let nvic = unsafe { &mut *(nvic_addr as *mut Nvic)};

    unsafe {
        intrinsics::volatile_store(&mut nvic.iser[int / 32], 1 << (int & 31));
    }
}

