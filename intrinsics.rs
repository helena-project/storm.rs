
extern "rust-intrinsic" {
    pub fn volatile_load<T>(src: *const T) -> T;
    pub fn volatile_store<T>(src: *mut T, value: T);
}

