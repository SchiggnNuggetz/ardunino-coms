#![no_std]
#![no_main]

mod ascii_encryption;
use ascii_encryption::Data256;

// use arduino_hal::{hal::usart::UsartWriter, pac::ac::acsr::W};
use panic_halt as _;

use avr_device::interrupt;
use core::cell::RefCell;


type Console = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;
static CONSOLE: interrupt::Mutex<RefCell<Option<Console>>> =
    interrupt::Mutex::new(RefCell::new(None));


// macro for print
macro_rules! print {
    ($($t:tt)*) => {
        interrupt::free(
            |cs| {
                if let Some(console) = CONSOLE.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwrite!(console, $($t)*);
                }
            }
        )
    };
}

// macro for println
macro_rules! println {
    ($($t:tt)*) => {
        interrupt::free(
            |cs| {
                if let Some(console) = CONSOLE.borrow(cs).borrow_mut().as_mut() {
                    let _ = ufmt::uwriteln!(console, $($t)*);
                }
            }
        )
    };
}

// function to put consol
fn put_console(console: Console) {
    interrupt::free(|cs| {
        *CONSOLE.borrow(cs).borrow_mut() = Some(console);
    });
}




#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    put_console(serial);

    let sample_data: [u8; 10] = [0xFF, 0xAF, 0x00, 0x23, 0x66, 0x50, 0x00, 0x00, 0x00, 0x23];
    let mut data = Data256::new(10);

    data.data[..10].copy_from_slice(&sample_data);
    data.print();
    let sending_data = data.encrypt_to_ascii();
    sending_data.print();
    let sending_data = sending_data.decrypt_from_ascii();
    sending_data.print();
    // print!("{}", sending_data[])
    println!("{}", 0x3F as char);

    let mut led = pins.d13.into_output();

    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
