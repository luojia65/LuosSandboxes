//! Useful sandboxes for testing memory, network(todo) and file system(todo),
//! which is useful in developing data structures. 
//! 
//! # Memory Sandboxing
//! 
//! The memory sandboxing part is designed for debugging data structures who
//! manages memory ownerships. With this purpose it provides various customizable 
//! kinds of allocators. There are 4 structs in this part. 
//! 
//! `LuosAlloc` is a general allocator which provides function `inner` to see 
//! through what had been allocated, and by using `new_filled_with` on creation
//! we can easily distinguish bytes that is normal or being leaked. It adapts for
//! the Rust allocation API by implementing the `Alloc` trait.
//! 
//! `LuosMustReplaceAlloc` is like `LuosAlloc`, also fitted in the Rust allocation
//! API, but every time it `realloc`'s it ensures that new memory section is not 
//! extended from the old one, instead it always trys to allocate for new memory
//! sections. It's useful to detect logical bugs when writing data strutures.
//! 
//! `LuosGlobalAlloc` is also able to allocate, but it implements `GlobalAlloc`
//! trait instead. By using it as a global allocator we can debug our programs 
//! with the full Rust minimum-runtime allocated into it. 
//! 
//! `LuosMemory` is a 64-KiB linear buffer which is used by all allocators provided
//! in this crate. When constructing allocators from this crate, we must create a 
//! `LuosMemory` buffer using `LuosMemory::new()`, and we may create allocators 
//! using `let mut a = LuosAlloc::new(LuosMemory::new())`.
//! 

#![feature(allocator_api, alloc_layout_extra)]
#![feature(ptr_offset_from)]
#![feature(const_fn, const_let)]
#![feature(int_to_from_bytes)]

#![warn(missing_docs)]

mod mem_sandbox;
mod net_sandbox;

pub use crate::mem_sandbox::{
    LuosAlloc, 
    LuosMustReplaceAlloc, 
    LuosGlobalAlloc,
    LuosMemory
};
