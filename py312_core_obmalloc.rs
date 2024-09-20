use std::os::raw::{c_uint}; // Keep only used c_uint
use std::sync::{Arc, Mutex}; // For atomic reference counting and mutexes
use std::alloc::{GlobalAlloc, Layout, System}; // For custom memory allocation
use std::boxed::Box; // For memory safety with Box
use std::mem::ManuallyDrop; // For wrapping non-Copy types in a union

// Memory pool header structure, containing all information needed to manage the memory pool
#[repr(C)]
pub struct PoolHeader {
    pub ref_: PoolHeaderRef, // Use smart pointers for safe memory management
    pub freeblock: *mut u8,  // Pointer to free memory block
    pub nextpool: Option<Box<PoolHeader>>, // Use Box to handle memory automatically
    pub prevpool: Option<Box<PoolHeader>>, // Use Box for previous pool reference
    pub arenaindex: c_uint, // Index of the memory pool in the Arena
    pub szidx: c_uint,      // Corresponding memory size index
    pub nextoffset: c_uint, // Next available offset
    pub maxnextoffset: c_uint, // Maximum available offset
}

// Reference counting union, providing counting functionality
#[repr(C)]
pub union PoolHeaderRef {
    pub ref_count: ManuallyDrop<Arc<Mutex<u32>>>, // Atomic reference counting with Mutex for thread safety
    pub _padding: *mut u8, // Padding if needed
}

// Implement custom memory allocation using Rust's std::alloc module
struct CustomAllocator;

unsafe impl GlobalAlloc for CustomAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout) // Use system allocator by default
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout) // Deallocate memory using system allocator
    }
}

// Use Result and Option types for error handling
#[allow(dead_code)]
fn allocate_pool(size: usize) -> Result<*mut u8, &'static str> {
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

// Example function demonstrating ownership, thread safety, and safe memory management
#[allow(dead_code)]
fn manage_memory_pools() -> Result<(), &'static str> {
    let pool_header = PoolHeader {
        ref_: PoolHeaderRef {
            ref_count: ManuallyDrop::new(Arc::new(Mutex::new(1))), // Initialize reference count with Arc + Mutex
        },
        freeblock: allocate_pool(1024)?, // Allocate memory safely using custom allocator
        nextpool: None, // Initialize with no next pool
        prevpool: None, // Initialize with no previous pool
        arenaindex: 0,  // Set arena index
        szidx: 0,       // Set memory size index
        nextoffset: 0,  // Start with no offset
        maxnextoffset: 1024, // Maximum offset set to memory pool size
    };

    // Accessing union field safely using an unsafe block
    unsafe {
        let mut ref_lock = pool_header.ref_.ref_count.lock().map_err(|_| "Mutex lock failed")?;
        *ref_lock += 1; // Increment reference count in a thread-safe manner
        println!("Memory pool successfully managed with reference count: {}", *ref_lock);
    }

    Ok(())
}

// Demonstrating the custom memory allocator
#[global_allocator]
static A: CustomAllocator = CustomAllocator;

#[allow(dead_code)]
fn main() -> Result<(), &'static str> {
    // Call memory pool management function in the main program
    manage_memory_pools()?;
    println!("Program completed successfully.");
    Ok(())
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
