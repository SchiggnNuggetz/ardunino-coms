#![no_std]
#![no_main]

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

fn put_console(console: Console) {
    interrupt::free(|cs| {
        *CONSOLE.borrow(cs).borrow_mut() = Some(console);
    });
}

pub struct Data256 {
    data: [u8; 256],
    length: u8,
}

impl Data256 {
    #[must_use]
    pub const fn new(length: u8) -> Self {
        Self {
            data: [0; 256],
            length,
        }
    }
    #[must_use]
    pub fn decrypt(self) -> Self {
        let mut send_value = Self {
            data: [0; 256],
            length: 0,
        };
        let mut counter: u16 = 1;
        let mut counter_0: u16 = 0;

        for i in 0..u16::from(self.length) * 8 {
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

    #[must_use]
    pub fn encrypt(self) -> Self {
        let mut recieved_value =  Self {
            data: [0; 256],
            length: 0,
        };
        let mut counter: u16 = 0;
        let mut counter_0: u16  = 0;


        for i in 1..u16::from(self.length) * 8 {
            let bit: u8 = self.data[(i / 8) as usize] >> (7 - (i % 8)) & 0x01;
            if i % 8 == 0 {
                recieved_value.length+=1;
                continue;
            }

            if counter_0 >= 4 {
                counter_0 = 0;
                continue;
            }
            if bit == 0 {
                counter_0 += 1;
            } else {
                counter_0 = 0;
            }
            recieved_value.data[(counter / 8) as usize] |= bit << (7 - (counter % 8));
            counter+=1;
        }
        counter -= 1;

        match u8::try_from(counter.div_ceil(8)) {
            Ok(len) => recieved_value.length = len,
            Err(_e) => println!("Error chasting u16 to u8")
        }

        recieved_value
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let serial = arduino_hal::default_serial!(dp, pins, 57600);

    put_console(serial);

    let sample_data: [u8; 10] = [0xFF, 0xAF, 0x00, 0x23, 0x66, 0x10, 0x00, 0x00, 0x99, 0xFF];
    let mut data = Data256::new(10);

    data.data[..10].copy_from_slice(&sample_data);

    let sending_data = data.decrypt();
    let sending_data = sending_data.encrypt();

    for i in 0..sending_data.length {
        print!("{:#x} ", sending_data.data[i as usize]);
    }
    // print!("{}", sending_data[])

    println!("{}", 0x3F as char);

    let mut led = pins.d13.into_output();

    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
