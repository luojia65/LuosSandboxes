#![feature(allocator_api, alloc_layout_extra)]
#![feature(ptr_offset_from)]
#![feature(const_fn, const_let)]

use std::alloc::{Alloc, GlobalAlloc, Layout, AllocErr};    
use std::ptr::NonNull;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LuosMemory {
    buf: Vec<u8>,
}

const MAX_INDEX: u16 = u16::max_value();

impl LuosMemory {
    pub fn new() -> Self {
        Self {
            buf: vec![0u8; MAX_INDEX as usize],
        }
    }
}

#[derive(Debug, Clone)]
pub struct LuosAlloc {
    memory: LuosMemory,
    used: Vec<u16>,
}

impl LuosAlloc {
    pub fn new(memory: LuosMemory) -> Self {
        let mut used = Vec::with_capacity(MAX_INDEX as usize);
        for i in 0..MAX_INDEX {
            used.push(i);
        }
        Self { 
            memory,
            used
        }     
    }

    fn get_unused_begin(&self, size: usize) -> Option<usize> {
        if size == 0 {
            return Some(0);
        }
        if size > MAX_INDEX as usize {
            return None;
        }
        for i in 1..MAX_INDEX {
            if self.used[i as usize] == size as u16 {
                return Some(i as usize + 1 - size as usize);
            }
        }
        None
    }

    fn record_alloc_memory(&mut self, start: usize, len: usize) {
        let mut i = start + len - 1;
        let m = self.used[i];
        while self.used[i] != 0 {
            i += 1;
            if i >= MAX_INDEX as usize {
                break;
            }
            self.used[i] -= m;
        }
        for i in start..start+len {
            self.used[i] = 0;
        }
    }

    fn record_dealloc_memory(&mut self, start: usize, len: usize) {
        let last = self.used[start - 1];
        if start == 1 || (last != 0 && start != 1)  {
            for i in start..start+len {
                self.used[i] = last + 1 - start as u16 + i as u16;
            }
        }
        let following = self.used[start + len];
        if following != 0 {
            let mut i = start + len - 1;
            while self.used[i] != 0 {
                i += 1;
                if i >= MAX_INDEX as usize {
                    break;
                }
                self.used[i] = self.used[i - 1] + 1;
            }
        }
    } 
}
    
unsafe impl Alloc for LuosAlloc {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let size = layout.size();
        if let Some(start) = self.get_unused_begin(size) {
            let pos = self.memory.buf.as_mut_ptr().offset(start as isize);
            self.record_alloc_memory(start, size);
            Ok(NonNull::new(pos).unwrap())
        } else {
            Err(AllocErr)
        }
    }

    
    #[inline]
    unsafe fn alloc_zeroed(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let size = layout.size();
        if let Some(start) = self.get_unused_begin(size) {
            let pos = self.memory.buf.as_mut_ptr().offset(start as isize);
            self.record_alloc_memory(start, size);
            for i in start..start+size {
                self.memory.buf[i] = 0;
            }
            Ok(NonNull::new(pos).unwrap())
        } else {
            Err(AllocErr)
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        println!("Dealloc layout: {:?}", layout);
        let size = layout.size();
        let buf_ptr = self.memory.buf.as_mut_ptr();
        let start = ptr.as_ptr().offset_from(buf_ptr) as usize;
        self.record_dealloc_memory(start, size);
    }

    unsafe fn realloc(
        &mut self, 
        ptr: NonNull<u8>, 
        cur_layout: Layout, 
        new_size: usize
    ) -> Result<NonNull<u8>, AllocErr> {
        let buf_ptr = self.memory.buf.as_mut_ptr();
        let cur_start = ptr.as_ptr().offset_from(buf_ptr) as usize;
        let cur_size = cur_layout.size();
        if let Some(extend_start) = self.get_unused_begin(new_size - cur_size) {
            if extend_start == cur_start + cur_size {
                self.record_alloc_memory(extend_start, new_size - cur_size);
                Ok(ptr)
            } else {
                self.record_dealloc_memory(cur_start, cur_size);
                if let Some(new_start) = self.get_unused_begin(new_size) {
                    self.record_alloc_memory(new_start, new_size);
                    Ok(NonNull::new(buf_ptr.offset(new_start as isize)).unwrap())
                } else {
                    Err(AllocErr)
                }
            }
        } else {
            Err(AllocErr)
        }
    }
}

pub struct LuosGlobalAlloc {
    a: *mut LuosAlloc
}

impl LuosGlobalAlloc {
    pub fn new(memory: LuosMemory) -> Self {
        let alloc = LuosAlloc::new(memory);
        let a = Box::into_raw(Box::new(alloc));
        Self { a }
    }
}

impl Drop for LuosGlobalAlloc {
    fn drop(&mut self) {
        unsafe {
            let alloc = Box::from_raw(self.a);
            drop(alloc);
        }
    }
}

unsafe impl GlobalAlloc for LuosGlobalAlloc {

    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (*self.a).alloc(layout).unwrap().as_mut()
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        (*self.a).alloc_zeroed(layout).unwrap().as_mut()
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        (*self.a).dealloc(NonNull::new(ptr).unwrap(), layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        (*self.a).realloc(NonNull::new(ptr).unwrap(), layout, new_size).unwrap().as_mut()
    }
}

#[cfg(test)]
mod test_alloc {
    use super::{LuosAlloc, LuosMemory, MAX_INDEX};
    use std::alloc::{Alloc, Layout};

    #[test]
    fn my_alloc_alloc_record() {
        let mut a = LuosAlloc::new(LuosMemory::new());
        assert_eq!(Some(0), a.get_unused_begin(0));
        assert_eq!(Some(1), a.get_unused_begin(10));
        assert_eq!(Some(1), a.get_unused_begin(32768));
        assert_eq!(Some(1), a.get_unused_begin((MAX_INDEX - 1) as usize));
        assert_eq!(None, a.get_unused_begin(MAX_INDEX as usize));
        a.record_alloc_memory(3, 5);
        assert_eq!(Some(0), a.get_unused_begin(0));
        assert_eq!(Some(8), a.get_unused_begin(10));
        assert_eq!(Some(8), a.get_unused_begin((MAX_INDEX - 3 - 5) as usize));
        assert_eq!(None, a.get_unused_begin((MAX_INDEX - 3 - 5 + 1) as usize));
        a.record_alloc_memory(11, 2);
        assert_eq!(Some(8), a.get_unused_begin(3));
        assert_eq!(Some(13), a.get_unused_begin(4));
        let mut a = LuosAlloc::new(LuosMemory::new());
        a.record_alloc_memory(1, 65533);
        assert_eq!(Some(0), a.get_unused_begin(0));
        assert_eq!(Some(MAX_INDEX as usize - 1), a.get_unused_begin(1));
        assert_eq!(None, a.get_unused_begin(2));
    }

    #[test]
    fn my_alloc_dealloc_record() {
        let mut a = LuosAlloc::new(LuosMemory::new());
        a.record_alloc_memory(3, 5);
        assert_eq!(Some(1), a.get_unused_begin(1));
        assert_eq!(Some(8), a.get_unused_begin(3));
        assert_eq!(None, a.get_unused_begin((MAX_INDEX - 3 - 5 + 1) as usize));
        a.record_dealloc_memory(3, 5);
        assert_eq!(Some(1), a.get_unused_begin(1));
        assert_eq!(Some(1), a.get_unused_begin(3));
        assert_eq!(Some(1), a.get_unused_begin((MAX_INDEX - 1) as usize));
        a.record_alloc_memory(1, 10);
        a.record_dealloc_memory(1, 10);
        assert_eq!(Some(1), a.get_unused_begin(1));
        assert_eq!(Some(1), a.get_unused_begin(3));
    }

    #[test]
    fn my_alloc_alloc() {
        let mut a = LuosAlloc::new(LuosMemory::new());
        let l = Layout::array::<u8>(10).unwrap();
        unsafe {
            let ptr = a.alloc(l).unwrap();
            *ptr.cast::<u128>().as_mut() = 0x1f2f3f4f5f6f7f8f;
            a.dealloc(ptr, l);
            let ptr2 = a.alloc_zeroed(l).unwrap();
            assert_eq!(ptr2, ptr);
            assert_eq!(*ptr.cast::<u128>().as_ptr(), 0);
            a.dealloc(ptr2, l);
        }
        assert_eq!(Some(0), a.get_unused_begin(0));
        assert_eq!(Some(1), a.get_unused_begin(10));
        assert_eq!(Some(1), a.get_unused_begin(32768));
        assert_eq!(Some(1), a.get_unused_begin((MAX_INDEX - 1) as usize));
        assert_eq!(None, a.get_unused_begin(MAX_INDEX as usize));
    }
}