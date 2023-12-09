pub mod bump;
pub mod fixed_size;
pub mod linked_list;

use spin::Mutex;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

#[cfg(bump_allocator)]
use bump::BumpAllocator;

use crate::memory::{BootInfoFrameAllocator, FRAME_ALLOCATOR};

use fixed_size::FixedSizeBlockAllocator;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    mut frame_allocator: BootInfoFrameAllocator,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, &mut frame_allocator)?
                .flush()
        }
    }

    FRAME_ALLOCATOR.init_once(|| Mutex::new(frame_allocator));

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub struct Locked<T>(Mutex<T>);

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked(Mutex::new(inner))
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.0.lock()
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    let rem = addr % align;

    if rem == 0 {
        return addr;
    }
    addr - rem + align
}

#[cfg(bump_allocator)]
#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());
