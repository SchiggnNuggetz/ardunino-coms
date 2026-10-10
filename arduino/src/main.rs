#![no_std]
#![no_main]

use arduino_hal::{hal::usart::UsartWriter, pac::ac::acsr::W};
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

fn put_console(console: Console) {
    interrupt::free(|cs| {
        *CONSOLE.borrow(cs).borrow_mut() = Some(console);
    })
}

pub struct Data256 {
    data: [u8; 256],
    length: u8,
}

impl Data256 {
    pub fn new(length: u8) -> Data256 {
        Data256 {
            data: [0; 256],
            length,
        }
    }
    pub fn prepare_for_sending(self) -> Data256 {
        let mut send_value = Data256 {
            data: [0; 256],
            length: 0,
        };
        let mut counter: u16 = 1;
        let mut counter_0: u16 = 0;

        for i in 0..(self.length as u16) * 8 {
            let bit: u8 = self.data[(i / 8) as usize] >> (7 - (i % 8)) & 0x01;
            if bit == 0 {
                counter_0 += 1;
            } else {
                counter_0 = 0;
            }
            send_value.data[(counter / 8) as usize] |= bit << (7 - (counter % 8));
            counter += 1;

            if counter % 8 == 0 {
                counter += 1;
                send_value.length += 1;
            }
            if counter_0 >= 4 {
                // there are too many 0s (4)
                send_value.data[(counter / 8) as usize] |= 1 << (7 - (counter % 8));
                counter += 1;

                if counter % 8 == 0 {
                    counter += 1;
                    send_value.length += 1;
                }
                counter_0 = 0;
            }
        }
        if counter % 8 != 1 {
            send_value.length += 1;
        }

        send_value
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 115200);

    put_console(serial);

    let sample_data: [u8; 10] = [0xFF, 0xAF, 0x00, 0x23, 0x66, 0x10, 0x00, 0x00, 0x99, 0xFF];
    let mut data = Data256::new(10);
    for i in 0..10 {
        data.data[i] = sample_data[i];
    }

    let sending_data = data.prepare_for_sending();


    for i in 0..sending_data.length {
        print!("{} ", sending_data.data[i as usize] as char);
    }
    // print!("{}", sending_data[])

    println!("{}", 0x3F as char);

    let mut led = pins.d13.into_output();

    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
