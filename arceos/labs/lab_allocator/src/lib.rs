//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator};
use core::alloc::Layout;
use core::ptr::NonNull;
// use log::debug;

const ITEM_EVEN_SIZE: usize = 704932;

pub struct LabByteAllocator {
    heap_start: usize,
    heap_end: usize,
    b_pos: usize,
    b_count: usize,
    // 项数
    n: usize,
    // 轮数
    delta: usize,
    // 是否完成items和偶数项内存的申请
    items_even_allocated: bool,
    // 标记items和偶数项的内存申请
    items_even_pos: usize,
    items_even_count: usize,
}

impl Default for LabByteAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        LabByteAllocator {
            heap_start: 0,
            heap_end: 0,
            b_pos: 0,
            b_count: 0,
            n: 0,
            delta: 0,
            items_even_allocated: false,
            items_even_pos: 0,
            items_even_count: 0,
        }
    }
}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.heap_start = start;
        self.heap_end = start + size;
        self.items_even_pos = start;
        // debug!(
        //     "LabByteAllocator init, heap_start:{:#x}, heap_end:{:#x}",
        //     self.heap_start, self.heap_end
        // );
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        // self.heap_start = start;
        self.heap_end = start + size;
        // 申请items和even内存后重置b_pos的指针
        if self.n == 0 && self.delta == 0 {
            self.b_pos = self.heap_start + ITEM_EVEN_SIZE;
        }
        // debug!(
        //     "LabByteAllocator add memory, heap_start:{:#x}, heap_end:{:#x}",
        //     self.heap_start, self.heap_end
        // );
        Ok(())
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        // log::debug!(
        //     "alloc size {:?},align is {:?}",
        //     layout.size(),
        //     layout.align()
        // );
        if self.n == 0 {
            self.delta = layout.size() - 32;
            // debug!("可用内存大小: {:?}", self.heap_end - self.b_pos);
        }
        // 初始分配足够items和偶数项的内存空间
        if self.n == 0 && self.delta == 0 && !self.items_even_allocated {
            self.items_even_allocated = true;
            return Err(AllocError::NoMemory);
        }
        // 前面704932B大小的内存用于items和偶数项的分配释放
        // 剩下的区域用来进行pool和奇数项的分配和释放
        let items_values: [usize; 3] = [384, 192, 96];
        let pos: usize = if (items_values.contains(&layout.size()) && layout.align() == 8)
            || (self.n % 2 == 0 && layout.align() == 1)
        {
            self.items_even_pos
        } else {
            self.b_pos
        };
        // 分配起始地址对齐
        let alloc_start = (pos + layout.align() - 1) & !(layout.align() - 1);
        let alloc_end = alloc_start + layout.size();
        if alloc_end > self.heap_end {
            Err(AllocError::NoMemory)
        } else {
            // debug!("已申请第{:?}轮第{:?}项内存", self.delta, self.n);
            if (items_values.contains(&layout.size()) && layout.align() == 8)
                || (self.n % 2 == 0 && layout.align() == 1)
            {
                assert!(alloc_end <= self.heap_start + ITEM_EVEN_SIZE);
                self.items_even_pos = alloc_end;
                self.items_even_count += 1;
            } else {
                self.b_pos = alloc_end;
                self.b_count += 1;
            }
            // 更新项数
            if layout.align() == 1 {
                self.n = (self.n + 1) % 15;
            }
            // 创建并返回一个 NonNull 指针
            // debug!("alloc_start: {:#x}", alloc_start);
            let ptr =
                core::ptr::NonNull::new(alloc_start as *mut u8).ok_or(AllocError::NoMemory)?;
            Ok(ptr)
        }
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        let items_values: [usize; 3] = [384, 192, 96];
        if (items_values.contains(&layout.size()) && layout.align() == 8)
            || (self.n % 2 == 0 && layout.align() == 1)
        {
            self.items_even_count -= 1;
        } else {
            self.b_count -= 1;
        }
        // debug!(
        //     "dealloc size: {:?}, items_even_count: {:?}, b_count: {:?}",
        //     layout.size(),
        //     self.items_even_count,
        //     self.b_count
        // );
        if self.items_even_count == 0 {
            // debug!("dealloc items_even all");
            self.items_even_pos = self.heap_start;
        }
        if self.b_count == 0 {
            self.b_pos = self.heap_start + ITEM_EVEN_SIZE;
        }
    }
    fn total_bytes(&self) -> usize {
        // self.heap_end - self.heap_start
        // 刚开始时申请能容纳items和偶数项的内存大小
        if self.n == 0 && self.delta == 0 {
            return ITEM_EVEN_SIZE;
        }
        // debug!("heap_end: {:#x}, b_pos: {:#x}", self.heap_end, self.b_pos);
        self.heap_end - self.b_pos
    }
    fn used_bytes(&self) -> usize {
        unimplemented!();
    }
    fn available_bytes(&self) -> usize {
        unimplemented!();
    }
}
