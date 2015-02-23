pub mod irq {
    pub mod syscall;
}

pub mod lib {
    pub mod array_list;
    pub mod ring_buffer;
    pub mod bitvector;
}

pub mod process;

mod mem {
    mod alloc;
}
