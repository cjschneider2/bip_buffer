#![feature(alloc)]
#![feature(heap_api)]
#![feature(unique)]

// XXX: For dev; Remove when ready
#![allow(dead_code)]
#![allow(unused_imports)]

// NOTE:
// As this is a generic structure and not just a buffer of bytes, we can't
// directly say that the size of the array and number of data elements are
// equal, so the terminology changes here to be based off of indexes.

extern crate core;
extern crate alloc;

use core::prelude::*;

use core::ptr;
use core::mem;

use alloc::heap;

enum Error {
    Oom,
    CapOverflow,
    ZeroSize,
}

// The main `A` window.
struct A {
    index: usize, // Starting element of the A region
    count: usize, // Elements in the a region
}

// The secondary `B` window.
struct B {
    count: usize,   // Elements in the b region
    in_use: bool, // True is the B region is used
}

struct BipBuffer<T> {
    a: A,
    b: B,
    rsvp: A, // The reserved size/index
    // Data info
    data: ptr::Unique<T>,
    size: usize,
}

/// NOTE:
/// The BipBuffer specifically disallows storing a buffer of zero sized
/// types.
impl <T> BipBuffer<T> {

    /// Creates a buffer with the capacity of `T`.
    pub fn new(cap: usize) -> Result<BipBuffer<T>, Error> {
        let ptr = unsafe { ptr::Unique::new(heap::EMPTY as *mut T) };
        Ok(
            BipBuffer {
                a: A {
                    index: 0,
                    count: 0,
                },
                b: B {
                    count: 0,
                    in_use: false,
                },
                rsvp: A {
                    index: 0,
                    count: 0,
                },
                data: ptr,
                size: mem::size_of::<T>() * cap,
            }
        )
    }

    /// Reset the buffer to it's `empty` state.
    /// NOTE: This does not free or clear the underlying buffer.
    pub fn clear(&mut self) {
        self.a.index = 0;    self.a.count = 0;
        self.b.count = 0;    self.b.in_use = false;
        self.rsvp.index = 0; self.rsvp.count = 0;
    }

    /// Returns the total size of the buffer and its contained data in bytes.
    pub fn size_of(&self) -> usize {
        mem::size_of_val(self)
    }

    /// Returns the element capacity of the buffer.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the number of elements in use by the buffer.
    pub fn used(&self) -> usize {
        (self.a.count - self.a.index) + self.b.count
    }

    /// Returns the number of elements unused by the buffer.
    pub fn unused(&self) -> usize {
        if self.b.in_use {
            self.a.index - self.b.count
        } else {
            self.size - self.a.count
        }
    }

    /// Returns a `(pointer, size)` pair to the first contiguous block of
    /// elements in the buffer and the number of elements in the block.
    pub fn get_block(&self) -> Option<(*mut T, usize)> {
        if self.a.count == 0 { return None }
        let ptr = unsafe { self.data.offset(self.a.index as isize) };
        Some( (ptr, self.a.count) )
    }

    /// Reserve the requested amount of elements for writing
    pub fn reserve(&mut self, req_cap: usize) -> Option<(*mut T, usize)> {

        if self.b.in_use { // Always allocate on `B` if it exists

            let mut free_cap = self.get_b_free_elements();
            if req_cap < free_cap { free_cap = req_cap; } // Don't over-allocate
            if free_cap == 0 { return None; } // Can't allocate

            self.rsvp.count = free_cap;
            self.rsvp.index = 0;

            let ptr = unsafe { self.data.offset(0) };

            return Some( (ptr, free_cap) );

        } else {
            // check if space after A is larger than the space before A
            // and use the larger one if it fits.
        }

        // TODO: Remove
        let ret = unsafe {
            (self.data.offset(0), 45)
        };
        Some(ret)
    }

    /// Gets the number of elements which are available after `A` for allocation
    /// This is the total buffer length minus the elements before `A` minus the
    /// elements used by `A`.
    fn get_elements_after_a(&self) -> usize {
        self.size - self.a.index - self.a.count
    }

    /// Gets the number of free elements available for `B` to allocate. This is
    /// the starting index of `A` minus the number of elements already in use
    /// by `B`. We don't have to worry about `B`'s starting index as it always
    /// begins at 0.
    fn get_b_free_elements(&self) -> usize {
        self.a.index - self.b.count
    }
}

// TODO: Impl Drop or w/e free() fn we need

#[cfg(test)]
mod tests {
    use super::BipBuffer;

    #[test]
    fn it_works() {
        BipBuffer::<u8>::new(12);
    }
}
