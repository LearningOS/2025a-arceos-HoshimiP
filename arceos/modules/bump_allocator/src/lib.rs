#![no_std]

use allocator::{BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = start + size;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        if size == 0 {
            return Err(allocator::AllocError::InvalidParam);
        }
        let end = start.checked_add(size).unwrap();
        if start >= self.start && end <= self.end {
            return Err(allocator::AllocError::MemoryOverlap);
        }
        if start == self.end {
            self.end = end;
            self.p_pos = self.end;
            return Ok(());
        }
        if end == self.start {
            self.start = start;
            if self.b_pos < self.start {
                self.b_pos = self.start;
            }
            return Ok(());
        }
        Err(allocator::AllocError::InvalidParam)
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        use allocator::AllocError;

        let size = layout.size();
        let align = layout.align();

        let align_mask = align - 1;
        let aligned = (self.b_pos + align_mask) & !align_mask;

        let new_b_pos = aligned.checked_add(size).unwrap();
        if new_b_pos > self.p_pos {
            return Err(AllocError::NoMemory);
        }

        let ptr = core::ptr::NonNull::new(aligned as *mut u8)
            .ok_or(AllocError::InvalidParam)?;
        self.b_pos = new_b_pos;
        
        Ok(ptr)
    }

    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        let mut size = layout.size();
        if size == 0 {
            size = 1;
        }
        let addr = pos.as_ptr() as usize;
        if let Some(end_addr) = addr.checked_add(size) {
            if end_addr == self.b_pos {
                if addr >= self.start && end_addr <= self.end {
                    self.b_pos = addr;
                }
            }
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        todo!()
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        todo!()
    }

    fn total_pages(&self) -> usize {
        todo!()
    }

    fn used_pages(&self) -> usize {
        todo!()
    }

    fn available_pages(&self) -> usize {
        todo!()
    }
}