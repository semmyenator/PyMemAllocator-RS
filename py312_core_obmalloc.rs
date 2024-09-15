// Import necessary standard libraries
use std::os::raw::{c_int, c_uint, c_void}; // For C language integer types
use std::ptr; // For pointer operations
use std::sync::Arc; // For atomic reference counting

// Memory pool header structure, containing all information needed to manage the memory pool
#[repr(C)]
pub struct PoolHeader {
    pub ref_: PoolHeaderRef, // Reference count or padding for managing memory pool references
    pub freeblock: *mut u8, // Pointer to free memory block
    pub nextpool: *mut PoolHeader, // Pointer to the next memory pool
    pub prevpool: *mut PoolHeader, // Pointer to the previous memory pool
    pub arenaindex: c_uint, // Index of the memory pool in the Arena
    pub szidx: c_uint, // Corresponding memory size index
    pub nextoffset: c_uint, // Next available offset
    pub maxnextoffset: c_uint, // Maximum available offset
}

// Reference counting union, providing counting functionality
#[repr(C)]
pub union PoolHeaderRef {
    pub _padding: *mut u8, // Padding pointer, placeholder when unused
    pub count: c_uint, // Count, indicates the number of references to this memory pool
}

// Arena object structure, managing a group of memory pools
#[repr(C)]
pub struct ArenaObject {
    pub address: usize, // Address of the Arena
    pub pool_address: *mut u8, // Pointer to the pools in this Arena
    pub nfreepools: c_uint, // Number of free pools
    pub ntotalpools: c_uint, // Total number of pools
    pub freepools: *mut PoolHeader, // Pointer to free pools
    pub nextarena: *mut ArenaObject, // Pointer to the next Arena
    pub prevarena: *mut ArenaObject, // Pointer to the previous Arena
}

// External C functions declaration for memory management
extern "C" {
    pub fn _PyObject_VirtualAlloc(size: usize) -> *mut c_void; // Virtual memory allocation
    pub fn _PyObject_VirtualFree(ptr: *mut c_void, size: usize); // Virtual memory free
    pub fn _Py_GetGlobalAllocatedBlocks() -> isize; // Get global allocated blocks
    pub fn _PyInterpreterState_GetAllocatedBlocks(interp: *mut c_void) -> isize; // Get allocated blocks for the interpreter
    pub fn _PyInterpreterState_FinalizeAllocatedBlocks(interp: *mut c_void); // Finalize allocated blocks
    pub fn _PyMem_init_obmalloc(interp: *mut c_void) -> c_int; // Initialize obmalloc
    pub fn _PyMem_obmalloc_state_on_heap(interp: *mut c_void) -> bool; // Check obmalloc state
}

// Memory allocator structure, interacting with the Python interpreter
pub struct PyMemAllocator {
    interp: *mut c_void, // Pointer to the Python interpreter
}

// Implementing methods for memory allocator
impl PyMemAllocator {
    // Constructor to initialize memory allocator
    pub fn new(interp: *mut c_void) -> Self {
        PyMemAllocator { interp }
    }

    // Initialize obmalloc memory allocation system
    pub fn init_obmalloc(&mut self) {
        unsafe {
            _PyMem_init_obmalloc(self.interp); // Call C function to initialize
        }
    }

    // Check if obmalloc state is on heap
    pub fn is_obmalloc_state_on_heap(&self) -> bool {
        unsafe {
            _PyMem_obmalloc_state_on_heap(self.interp) // Call C function to check state
        }
    }

    // Get global allocated blocks
    pub fn get_allocated_blocks(&self) -> isize {
        unsafe {
            _Py_GetGlobalAllocatedBlocks() // Call C function to get global allocated blocks
        }
    }

    // Get interpreter allocated blocks
    pub fn get_interpreter_allocated_blocks(&self) -> isize {
        unsafe {
            _PyInterpreterState_GetAllocatedBlocks(self.interp) // Call C function to get allocated blocks for the interpreter
        }
    }

    // Finalize allocated blocks, releasing resources
    pub fn finalize_allocated_blocks(&mut self) {
        unsafe {
            _PyInterpreterState_FinalizeAllocatedBlocks(self.interp); // Call C function to release resources
        }
    }

    // Virtual memory allocation, allocating memory based on size
    pub fn virtual_alloc(&self, size: usize) -> *mut c_void {
        unsafe { _PyObject_VirtualAlloc(size) } // Call C function to allocate memory
    }

    // Virtual memory free, releasing specified memory
    pub fn virtual_free(&self, ptr: *mut c_void, size: usize) {
        unsafe { _PyObject_VirtualFree(ptr, size) } // Call C function to free memory
    }
}

// Implementing methods for Arena object
impl ArenaObject {
    // Constructor to initialize Arena object
    pub fn new() -> Self {
        ArenaObject {
            address: 0, // Initialize address to 0
            pool_address: ptr::null_mut(), // Initialize to null pointer
            nfreepools: 0, // Initialize free pool count to 0
            ntotalpools: 0, // Initialize total pool count to 0
            freepools: ptr::null_mut(), // Initialize free pool pointer to null
            nextarena: ptr::null_mut(), // Initialize to null pointer
            prevarena: ptr::null_mut(), // Initialize to null pointer
        }
    }

    // Allocate a new pool, returning the pool pointer
    pub fn allocate_pool(&mut self) -> *mut PoolHeader {
        let pool = Box::into_raw(Box::new(PoolHeader {
            ref_: PoolHeaderRef { _padding: ptr::null_mut() }, // Initialize reference
            freeblock: ptr::null_mut(), // Initialize free block pointer to null
            nextpool: ptr::null_mut(), // Initialize to null pointer
            prevpool: ptr::null_mut(), // Initialize to null pointer
            arenaindex: 0, // Initialize Arena index to 0
            szidx: 0, // Initialize size index to 0
            nextoffset: 0, // Initialize next offset to 0
            maxnextoffset: 0, // Initialize maximum offset to 0
        }));
        self.freepools = pool; // Update free pools pointer
        pool // Return newly allocated pool pointer
    }

    // Free a pool, releasing memory
    pub fn free_pool(&mut self, pool: *mut PoolHeader) {
        unsafe {
            Box::from_raw(pool); // Release the pool back to memory
        }
        self.nfreepools += 1; // Update the count of free pools
    }
}
