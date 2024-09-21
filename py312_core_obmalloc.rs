use std::os::raw::{c_uint}; // Keep only used c_uint
use std::sync::{Arc, Mutex}; // For atomic reference counting and mutexes
use std::alloc::{GlobalAlloc, Layout, System}; // For custom memory allocation
use std::boxed::Box; // For memory safety with Box
use std::mem::ManuallyDrop; // For wrapping non-Copy types in a union

// Memory pool header structure
#[repr(C)]
pub struct PoolHeader {
    pub ref_: PoolHeaderRef,
    pub freeblock: *mut u8,
    pub nextpool: Option<Box<PoolHeader>>,
    pub prevpool: Option<Box<PoolHeader>>,
    pub arenaindex: c_uint,
    pub szidx: c_uint,
    pub nextoffset: c_uint,
    pub maxnextoffset: c_uint,
}

// Reference counting union
#[repr(C)]
pub union PoolHeaderRef {
    pub ref_count: ManuallyDrop<Arc<Mutex<u32>>>,
    pub _padding: *mut u8,
}

// Custom memory allocator
struct CustomAllocator;

unsafe impl GlobalAlloc for CustomAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
}

// Memory allocation function
pub fn allocate_pool(size: usize) -> Result<*mut u8, &'static str> {
    let layout = Layout::from_size_align(size, 8).map_err(|_| "Invalid memory layout")?;
    unsafe {
        let ptr = System.alloc(layout);
        if ptr.is_null() {
            Err("Memory allocation failed")
        } else {
            Ok(ptr)
        }
    }
}

// Memory management function
pub fn manage_memory_pools() -> Result<(), &'static str> {
    let pool_header = PoolHeader {
        ref_: PoolHeaderRef {
            ref_count: ManuallyDrop::new(Arc::new(Mutex::new(1))),
        },
        freeblock: allocate_pool(1024)?,
        nextpool: None,
        prevpool: None,
        arenaindex: 0,
        szidx: 0,
        nextoffset: 0,
        maxnextoffset: 1024,
    };

    unsafe {
        let mut ref_lock = pool_header.ref_.ref_count.lock().map_err(|_| "Mutex lock failed")?;
        *ref_lock += 1;
        println!("Memory pool successfully managed with reference count: {}", *ref_lock);
    }

    Ok(())
}

// Custom memory allocator
#[global_allocator]
static A: CustomAllocator = CustomAllocator;

// Main function
#[cfg(test)]
fn main() {
    let _ = std::panic::catch_unwind(|| {
        manage_memory_pools().unwrap();
        println!("Program completed successfully.");
    });
}

// Test module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pools() {
        let result = manage_memory_pools();
        assert!(result.is_ok());
    }
}
