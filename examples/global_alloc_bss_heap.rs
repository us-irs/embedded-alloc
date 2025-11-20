#![no_std]
#![no_main]

extern crate alloc;

use cortex_m as _;
use cortex_m_rt::entry;
use defmt::Debug2Format;
use defmt_semihosting as _;

use core::panic::PanicInfo;
use embedded_alloc::TlsfHeap as Heap;

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[entry]
fn main() -> ! {
    const HEAP_SIZE: usize = 16 * 1024;
    static HEAP_MEM: static_cell::ConstStaticCell<[u8; HEAP_SIZE]> =
        static_cell::ConstStaticCell::new([0; HEAP_SIZE]);
    unsafe {
        HEAP.init(HEAP_MEM.take().as_mut_ptr() as usize, HEAP_SIZE);
    }

    let vec = alloc::vec![1];

    defmt::info!("Allocated vector: {:?}", Debug2Format(&vec));

    let string = alloc::string::String::from("Hello, world!");

    defmt::info!("Allocated string: {:?}", Debug2Format(&string));

    semihosting::process::exit(0);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    defmt::error!("{}", info);
    semihosting::process::exit(0);
}
