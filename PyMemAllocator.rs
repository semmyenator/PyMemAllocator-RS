use std::os::raw::{c_int, c_uint, c_void};

// The PoolHeader struct represents a single memory pool in the obmalloc allocator.
// It contains information about the free blocks, linked list of pools, and metadata.
#[repr(C)]
pub struct PoolHeader {
    // The ref_ field is a union that can be used to access either the _padding pointer
    // or the count of free blocks in this pool.
    pub ref_: PoolHeaderRef,
    // The freeblock pointer points to the next available free block in this pool.
    pub freeblock: *mut u8,
    // The nextpool and prevpool pointers form a doubly-linked list of pools.
    pub nextpool: *mut PoolHeader,
    pub prevpool: *mut PoolHeader,
    // The arenaindex and szidx fields store metadata about the pool's location and size class.
    pub arenaindex: c_uint,
    pub szidx: c_uint,
    pub nextoffset: c_uint,
    pub maxnextoffset: c_uint,
}

// The PoolHeaderRef union is used to access the _padding pointer or the count field in the PoolHeader.
#[repr(C)]
pub union PoolHeaderRef {
    pub _padding: *mut u8,
    pub count: c_uint,
}

// The ArenaObject struct represents a larger memory arena that contains multiple pools.
// It stores information about the arena's address, the pools it contains, and linked list pointers.
#[repr(C)]
pub struct ArenaObject {
    pub address: usize,
    pub pool_address: *mut u8,
    pub nfreepools: c_uint,
    pub ntotalpools: c_uint,
    pub freepools: *mut PoolHeader,
    pub nextarena: *mut ArenaObject,
    pub prevarena: *mut ArenaObject,
}

// The following extern "C" block declares the C functions that the Rust code will call.
// These functions are part of the Python obmalloc allocator implementation.
extern "C" {
    // _PyObject_VirtualAlloc allocates a block of memory from the operating system.
    pub fn _PyObject_VirtualAlloc(size: usize) -> *mut c_void;
    // _PyObject_VirtualFree releases a block of memory back to the operating system.
    pub fn _PyObject_VirtualFree(ptr: *mut c_void, size: usize);
    // _Py_GetGlobalAllocatedBlocks returns the total number of memory blocks allocated globally.
    pub fn _Py_GetGlobalAllocatedBlocks() -> isize;
    // _PyInterpreterState_GetAllocatedBlocks returns the number of memory blocks allocated for a specific interpreter.
    pub fn _PyInterpreterState_GetAllocatedBlocks(interp: *mut c_void) -> isize;
    // _PyInterpreterState_FinalizeAllocatedBlocks performs cleanup for the memory blocks allocated for a specific interpreter.
    pub fn _PyInterpreterState_FinalizeAllocatedBlocks(interp: *mut c_void);
    // _PyMem_init_obmalloc initializes the obmalloc memory allocator for a specific interpreter.
    pub fn _PyMem_init_obmalloc(interp: *mut c_void) -> c_int;
    // _PyMem_obmalloc_state_on_heap indicates whether the obmalloc state is stored on the heap for a specific interpreter.
    pub fn _PyMem_obmalloc_state_on_heap(interp: *mut c_void) -> bool;
}

// The PyMemAllocator struct is the main interface for interacting with the obmalloc allocator.
// It encapsulates a pointer to the interpreter state, which is used when calling the C functions.
pub struct PyMemAllocator {
    interp: *mut c_void,
}

impl PyMemAllocator {
    // The new function creates a new PyMemAllocator instance, taking a pointer to the interpreter state.
    pub fn new(interp: *mut c_void) -> Self {
        PyMemAllocator { interp }
    }

    // The init_obmalloc function initializes the obmalloc allocator for the associated interpreter.
    pub fn init_obmalloc(&mut self) {
        unsafe {
            _PyMem_init_obmalloc(self.interp);
        }
    }

    // The is_obmalloc_state_on_heap function checks whether the obmalloc state is stored on the heap for the associated interpreter.
    pub fn is_obmalloc_state_on_heap(&self) -> bool {
        unsafe {
            _PyMem_obmalloc_state_on_heap(self.interp)
        }
    }

    // The get_allocated_blocks function returns the total number of memory blocks allocated globally.
    pub fn get_allocated_blocks(&self) -> isize {
        unsafe {
            _Py_GetGlobalAllocatedBlocks()
        }
    }

    // The get_interpreter_allocated_blocks function returns the number of memory blocks allocated for the associated interpreter.
    pub fn get_interpreter_allocated_blocks(&self) -> isize {
        unsafe {
            _PyInterpreterState_GetAllocatedBlocks(self.interp)
        }
    }

    // The finalize_allocated_blocks function performs cleanup for the memory blocks allocated for the associated interpreter.
    pub fn finalize_allocated_blocks(&mut self) {
        unsafe {
            _PyInterpreterState_FinalizeAllocatedBlocks(self.interp);
        }
    }

    // The virtual_alloc function allocates a block of memory from the operating system.
    pub fn virtual_alloc(&self, size: usize) -> *mut c_void {
        unsafe { _PyObject_VirtualAlloc(size) }
    }

    // The virtual_free function releases a block of memory back to the operating system.
    pub fn virtual_free(&self, ptr: *mut c_void, size: usize) {
        unsafe { _PyObject_VirtualFree(ptr, size) }
    }
}