use std::cell::Cell;

pub struct ControllerState {
    button_state: Cell<u8>,
    /* bits
    0 - a
    1 - n
    2 - select
    3 - start
    4 - up
    5 - down
    6 - left
    7 - right
     */
    shift_reg: Cell<u8>,
    strobe: Cell<bool>,
}

impl ControllerState {
    pub fn new() -> Self {
        Self { button_state: Cell::new(0), shift_reg: Cell::new(0), strobe: Cell::new(false) }
    }

    pub fn set_buttons(&self, buttons: u8) {
        self.button_state.set(buttons);
    }

    pub fn write_strobe(&self, data: u8) {
        let strobe_on = data & 1 != 0;
        if strobe_on {
            self.strobe.set(true);
            self.shift_reg.set(self.button_state.get());
        } else {
            if self.strobe.get() {
                self.shift_reg.set(self.button_state.get());
            }
            self.strobe.set(false);
        }
    }

    pub fn read(&self) -> u8 {
        if self.strobe.get() {
            self.button_state.get() & 1
        } else {
            let bit = self.shift_reg.get() & 1;
            self.shift_reg.set((self.shift_reg.get() >> 1) | 0x80);
            bit
        }
    }
}