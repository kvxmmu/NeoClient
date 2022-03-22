#![feature(core_intrinsics)]
#![feature(thread_local)]

// Protocol
pub mod frame;
pub mod reader;

pub mod ipc;
pub mod net;

pub mod client;
pub mod session;
pub mod client_impl;

pub mod channels;
pub mod proxy_impl;

// Helpers
pub mod compressor;

// Prelude
pub mod prelude;
