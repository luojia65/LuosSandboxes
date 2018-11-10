#![feature(allocator_api, alloc_layout_extra)]
#![feature(ptr_offset_from)]

use luos_memory_sandbox::{LuosMemory, LuosAlloc};

use std::alloc::{Alloc, Layout, Global};
use std::mem;
use std::ptr::{self, NonNull};

pub struct RawVec<T, A: Alloc = Global> {
    ptr: *mut T,
    cap: usize,
    a: A,
}

const DEFAULT_CAPACITY: usize = 4;

impl<T, A: Alloc> RawVec<T, A> {

    pub fn cap(&self) -> usize {
        self.cap
    }

    pub fn new_in(a: A) -> Self {
        Self {
            ptr: ptr::null_mut(),
            cap: 0,
            a
        }
    }

    pub fn with_capacity_in(cap: usize, a: A) -> Self {
        Self::allocate_in(cap, a)
    }

    fn allocate_in(cap: usize, mut a: A) -> Self {
        unsafe {
            let elem_size = mem::size_of::<T>();
            let alloc_size = cap.checked_mul(elem_size)
                .expect("Capacity overflow!");
            let ptr = if alloc_size == 0 {
                ptr::null_mut()
            } else {
                let align = mem::align_of::<T>();
                let layout = Layout::from_size_align(alloc_size, align).unwrap();
                let ptr = a.alloc(layout).unwrap();
                ptr.cast().as_ptr()
            };
            Self {
                ptr,
                cap,
                a
            }
        }
    }

    pub fn double(&mut self) {
        unsafe {
            let elem_size = mem::size_of::<T>();
            let (new_cap, new_ptr) = match self.current_layout() {
                Some(cur_layout) => {
                    let new_cap = 2 * self.cap;
                    let new_size = new_cap * elem_size;
                    let new_ptr = self.a.realloc(NonNull::new(self.ptr).unwrap().cast(), cur_layout, new_size)
                        .expect("Realloc error!");
                    (new_cap, new_ptr)
                },
                None => {
                    let new_cap = DEFAULT_CAPACITY;
                    let new_ptr = self.a.alloc_array(new_cap)
                        .expect("Alloc error!");
                    (new_cap, new_ptr)
                }
            };
            self.ptr = new_ptr.cast().as_ptr();
            self.cap = new_cap;
        }
    } 

    fn current_layout(&self) -> Option<Layout> {
        if self.cap == 0 {
            return None;
        }
        unsafe {
            let align = mem::align_of::<T>();
            let size = mem::size_of::<T>() * self.cap;
            Some(Layout::from_size_align_unchecked(size, align))
        }
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn alloc(&self) -> &A {
        &self.a
    }

    pub fn alloc_mut(&mut self) -> &mut A {
        &mut self.a
    }
}

impl<T> RawVec<T, Global> {

    pub fn new() -> Self {
        Self::new_in(Global)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_in(cap, Global)
    }
}

#[test]
fn raw_vec_create() {
    let a = LuosAlloc::new(LuosMemory::new());
    let mut vec: RawVec<u128, LuosAlloc> = RawVec::new_in(a);
    vec.double();
    vec.double();
    drop(vec);
}
