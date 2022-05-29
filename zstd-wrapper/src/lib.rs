#![feature(core_intrinsics)]
#![feature(negative_impls)]

pub mod dctx;
pub mod cctx;

pub mod params;

pub mod prelude;

#[derive(Clone, Copy)]
struct UnsafeCell<T> {
    ptr: *mut T
}

unsafe impl<T> Send for UnsafeCell<T> {}
