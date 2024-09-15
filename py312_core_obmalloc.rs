use std::os::raw::{c_int, c_uint, c_void};
use std::ptr;
use std::sync::Arc;

#[repr(C)]
pub struct PoolHeader {
    pub ref_: PoolHeaderRef,
    pub freeblock: *mut u8,
    pub nextpool: *mut PoolHeader,
    pub prevpool: *mut PoolHeader,
    pub arenaindex: c_uint,
    pub szidx: c_uint,
    pub nextoffset: c_uint,
    pub maxnextoffset: c_uint,
}

#[repr(C)]
pub union PoolHeaderRef {
    pub _padding: *mut u8,
    pub count: c_uint,
}

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

extern "C" {
    pub fn _PyObject_VirtualAlloc(size: usize) -> *mut c_void;
    pub fn _PyObject_VirtualFree(ptr: *mut c_void, size: usize);
    pub fn _Py_GetGlobalAllocatedBlocks() -> isize;
    pub fn _PyInterpreterState_GetAllocatedBlocks(interp: *mut c_void) -> isize;
    pub fn _PyInterpreterState_FinalizeAllocatedBlocks(interp: *mut c_void);
    pub fn _PyMem_init_obmalloc(interp: *mut c_void) -> c_int;
    pub fn _PyMem_obmalloc_state_on_heap(interp: *mut c_void) -> bool;
}

pub struct PyMemAllocator {
    interp: *mut c_void,
}

impl PyMemAllocator {
    pub fn new(interp: *mut c_void) -> Self {
        PyMemAllocator { interp }
    }

    pub fn init_obmalloc(&mut self) {
        unsafe {
            _PyMem_init_obmalloc(self.interp);
        }
    }

    pub fn is_obmalloc_state_on_heap(&self) -> bool {
        unsafe {
            _PyMem_obmalloc_state_on_heap(self.interp)
        }
    }

    pub fn get_allocated_blocks(&self) -> isize {
        unsafe {
            _Py_GetGlobalAllocatedBlocks()
        }
    }

    pub fn get_interpreter_allocated_blocks(&self) -> isize {
        unsafe {
            _PyInterpreterState_GetAllocatedBlocks(self.interp)
        }
    }

    pub fn finalize_allocated_blocks(&mut self) {
        unsafe {
            _PyInterpreterState_FinalizeAllocatedBlocks(self.interp);
        }
    }

    pub fn virtual_alloc(&self, size: usize) -> *mut c_void {
        unsafe { _PyObject_VirtualAlloc(size) }
    }

    pub fn virtual_free(&self, ptr: *mut c_void, size: usize) {
        unsafe { _PyObject_VirtualFree(ptr, size) }
    }
}

impl ArenaObject {
    pub fn new() -> Self {
        ArenaObject {
            address: 0,
            pool_address: ptr::null_mut(),
            nfreepools: 0,
            ntotalpools: 0,
            freepools: ptr::null_mut(),
            nextarena: ptr::null_mut(),
            prevarena: ptr::null_mut(),
        }
    }

    pub fn allocate_pool(&mut self) -> *mut PoolHeader {
        let pool = Box::into_raw(Box::new(PoolHeader {
            ref_: PoolHeaderRef { _padding: ptr::null_mut() },
            freeblock: ptr::null_mut(),
            nextpool: ptr::null_mut(),
            prevpool: ptr::null_mut(),
            arenaindex: 0,
            szidx: 0,
            nextoffset: 0,
            maxnextoffset: 0,
        }));
        self.freepools = pool;
        pool
    }

    pub fn free_pool(&mut self, pool: *mut PoolHeader) {
        unsafe {
            Box::from_raw(pool);
        }
        self.nfreepools += 1;
    }
}
