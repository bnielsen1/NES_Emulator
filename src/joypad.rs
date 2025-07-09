use bitflags::bitflags;

bitflags! {
    // https://wiki.nesdev.com/w/index.php/Controller_reading_code
    #[derive(Copy, Clone)]
    pub struct JoypadButton: u8 {
        const RIGHT             = 0b10000000;
        const LEFT              = 0b01000000;
        const DOWN              = 0b00100000;
        const UP                = 0b00010000;
        const START             = 0b00001000;
        const SELECT            = 0b00000100;
        const BUTTON_B          = 0b00000010;
        const BUTTON_A          = 0b00000001;
    }
}

pub struct Joypad {
    strobe_status: bool,
    button_index: u8,
    button_status: JoypadButton,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe_status: false,
            button_index: 0,
            button_status: JoypadButton::from_bits_truncate(0b0000_0000)
        }
    }

    pub fn write(&mut self, data: u8) {
        if (data & 1) == 1 { // Writing strobe status on resets button index
            self.strobe_status = true;
            self.button_index = 0;
        } else { // Writing strobe status off starts the cycle of inputs
            self.strobe_status = false;
        }
    }

    pub fn read(&mut self) -> u8 {
        if self.button_index > 7 { // if we
            return 1;
        }

        // response gets a 1 or 0 depending on if the button at button_index is pressed or not
        let response = (self.button_status.bits() & (1 << self.button_index)) >> self.button_index;
        
        // response not included in if statement to force a controller button A read
        // every read if the strobe_status == true

        if !self.strobe_status && self.button_index <= 7 {
            self.button_index += 1;
        }
        response
    }

    pub fn set_button_pressed_status(&mut self, button: JoypadButton, status: bool) {
        if status {
            self.button_status.insert(button);
        } else {
            self.button_status.remove(button);
        }
    }
}