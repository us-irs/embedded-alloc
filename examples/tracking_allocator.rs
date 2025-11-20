//! Shows simple tracking allocator which tracks the number of allocated bytes.
#![no_std]
#![no_main]

extern crate alloc;

use cortex_m as _;
use cortex_m_rt::entry;
use defmt::Debug2Format;
use defmt_semihosting as _;

use core::{alloc::GlobalAlloc, mem::MaybeUninit, panic::PanicInfo};
use embedded_alloc::TlsfHeap as Heap;

pub struct TrackingHeap(Heap);

impl TrackingHeap {
    /// # Safety
    ///
    /// See safety note of [Heap::init].
    pub unsafe fn init(&self, start_addr: usize, size: usize) {
        unsafe { self.0.init(start_addr, size) }
    }

    pub const fn empty() -> Self {
        TrackingHeap(Heap::empty())
    }
}

static ALLOC_BYTES: core::sync::atomic::AtomicUsize = core::sync::atomic::AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingHeap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        ALLOC_BYTES.fetch_add(
            layout.pad_to_align().size(),
            core::sync::atomic::Ordering::Relaxed,
        );
        self.0.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        ALLOC_BYTES.fetch_sub(
            layout.pad_to_align().size(),
            core::sync::atomic::Ordering::Relaxed,
        );
        self.0.dealloc(ptr, layout)
    }
}

#[global_allocator]
static HEAP: TrackingHeap = TrackingHeap::empty();

#[entry]
fn main() -> ! {
    // Initialize the allocator BEFORE you use it
    const HEAP_SIZE: usize = 4096;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }

    let mut long_vec = alloc::vec::Vec::new();
    while ALLOC_BYTES.load(core::sync::atomic::Ordering::Relaxed)
        < (0.75 * HEAP_SIZE as f32) as usize
    {
        defmt::info!(
            "{} of {} heap memory allocated so far...",
            ALLOC_BYTES.load(core::sync::atomic::Ordering::Relaxed),
            HEAP_SIZE
        );
        long_vec.push([1; 16].to_vec());
    }

    defmt::warn!(
        "{} of {} heap memory are allocated!",
        ALLOC_BYTES.load(core::sync::atomic::Ordering::Relaxed),
        HEAP_SIZE
    );

    drop(long_vec);

    defmt::warn!(
        "{} of {} heap memory are allocated after drop",
        ALLOC_BYTES.load(core::sync::atomic::Ordering::Relaxed),
        HEAP_SIZE
    );

    // Panic is expected here.
    semihosting::process::exit(-1);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    defmt::error!("{}: {}", info, Debug2Format(&info.message()));
    semihosting::process::exit(0);
}
