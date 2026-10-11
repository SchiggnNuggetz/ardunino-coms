
use avr_device::interrupt;
use crate::CONSOLE;


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



pub struct Data256 {
    pub data: [u8; 256],
    pub length: u8,
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
    pub fn encrypt_to_ascii(self) -> Self {
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
        send_value.data[send_value.length as usize] = 0x03;
        send_value.length += 1;

        send_value
    }

    #[must_use]
    pub fn decrypt_from_ascii(self) -> Self {
        let mut recieved_value =  Self {
            data: [0; 256],
            length: 0,
        };
        let mut counter: u16 = 0;
        let mut counter_0: u16  = 0;

        let mut length: u8 = 0;
        while self.data[length as usize] != 0x03 {
            length+=1;
        }


        for i in 1..u16::from(length) * 8 {
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

        match u8::try_from(counter / 8) {
            Ok(len) => recieved_value.length = len,
            Err(_e) => recieved_value.length = 0,
        }

        recieved_value
    }

    pub fn print(&self) {
        for i in 0..self.length {
            print!("{:#x} ", self.data[i as usize]);
        }
        println!("");
    }
}