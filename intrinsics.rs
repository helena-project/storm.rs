
extern "rust-intrinsic" {
    pub fn volatile_load<T>(src: *const T) -> T;
    pub fn volatile_store<T>(src: *mut T, value: T);
}

#[lang="sized"]
pub trait Sized {}

#[lang="panic_bounds_check"]
fn fail_bounds_check(_: &(&'static str, uint),
                         _: uint, _: uint) -> ! {
    loop {}
}

#[lang="panic"]
fn panic(_: &(&'static str, &'static str, uint)) -> ! {
    loop {}
}

#[lang="sync"]
pub trait Sync {}

