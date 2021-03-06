use std::thread;
use std::time::Duration;

use rppal::gpio::{Gpio, InputPin, Level, Level::*, OutputPin};
use std::error::Error;

pub struct Fanshim {
    clk: OutputPin,
    dat: OutputPin,
    btn: InputPin,
    fan: OutputPin,
}

impl Fanshim {
    pub fn default_config() -> Result<Fanshim, Box<dyn Error>> {
        const CLK: u8 = 14;
        const DAT: u8 = 15;
        const BTN: u8 = 17;
        const FAN: u8 = 18;
        Ok(Fanshim {
            clk: Gpio::new()?.get(CLK)?.into_output(),
            dat: Gpio::new()?.get(DAT)?.into_output(),
            btn: Gpio::new()?.get(BTN)?.into_input_pullup(),
            fan: Gpio::new()?.get(FAN)?.into_output(),
        })
    }

    pub fn fan_on(&mut self) {
        self.fan.set_high();
    }

    pub fn fan_off(&mut self) {
        self.fan.set_low();
    }

    fn write_byte(&mut self, byte: u8) {
        let seq: Vec<Level> = (0..8u8)
            .map(move |bit| if ((byte << bit) & 128) == 128 { High } else { Low })
            .collect();
        for level in seq {
            self.dat.write(level);
            self.clk.set_high();
            thread::sleep(Duration::from_nanos(500));
            self.clk.set_low();
            thread::sleep(Duration::from_nanos(500));
        }
    }

    pub fn color(&mut self, br: u8, r: u8, g: u8, b: u8) {
        let brightness = if br > 31 { 1 } else { br };

        self.sof();
        self.write_byte(224 + brightness);
        self.write_byte(b);
        self.write_byte(g);
        self.write_byte(r);
        self.eof();
    }

    fn sof(&mut self) {
        for _ in 0..4 {
            self.write_byte(000);
        }
    }

    fn eof(&mut self) {
        for _ in 0..4 {
            self.write_byte(255);
        }
    }

    pub fn led_off(&mut self) {
        self.color(0, 0, 0, 0);
    }

    pub fn btn_state(&self) -> Level {
        self.btn.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blink_led_rgb() -> Result<(), Box<dyn Error>> {
        let mut fs: Fanshim = Fanshim::default_config()?;

        fs.led_off();
        thread::sleep(Duration::from_millis(1000));
        fs.color(16, 255, 0, 0);
        thread::sleep(Duration::from_millis(1000));
        fs.led_off();
        thread::sleep(Duration::from_millis(1000));
        fs.color(16, 0, 255, 0);
        thread::sleep(Duration::from_millis(1000));
        fs.led_off();
        thread::sleep(Duration::from_millis(1000));
        fs.color(16, 0, 0, 255);
        thread::sleep(Duration::from_millis(1000));
        fs.led_off();

        Ok(())
    }

    #[test]
    fn blink_led_brightness() -> Result<(), Box<dyn Error>> {
        let mut fs: Fanshim = Fanshim::default_config()?;

        fs.led_off();
        thread::sleep(Duration::from_millis(1000));
        for i in 0..32 {
            fs.color(i, 255, 255, 255);
            thread::sleep(Duration::from_millis(1000));
        }
        fs.led_off();

        Ok(())
    }

    #[test]
    fn blink_fan() -> Result<(), Box<dyn Error>> {
        let mut fs: Fanshim = Fanshim::default_config()?;

        fs.fan_off();
        fs.led_off();
        thread::sleep(Duration::from_millis(5000));
        fs.fan_on();
        fs.color(16, 0, 255, 0);
        thread::sleep(Duration::from_millis(5000));
        fs.fan_off();
        fs.color(16, 255, 0, 0);
        thread::sleep(Duration::from_millis(5000));
        fs.fan_on();
        fs.color(16, 0, 255, 0);
        thread::sleep(Duration::from_millis(5000));
        fs.fan_off();
        fs.color(16, 255, 0, 0);
        thread::sleep(Duration::from_millis(5000));
        fs.fan_on();
        fs.color(16, 0, 255, 0);
        thread::sleep(Duration::from_millis(5000));
        fs.fan_off();
        fs.led_off();

        Ok(())
    }

    #[test]
    fn react_to_btn_10s() -> Result<(), Box<dyn Error>> {
        use std::time::Instant;

        let mut fs: Fanshim = Fanshim::default_config()?;

        let start = Instant::now();
        let stop = Duration::from_secs(10);

        fs.led_off();
        while (Instant::now()).duration_since(start) < stop {
            if fs.btn_state() == High {
                fs.color(16, 255, 255, 255);
            } else {
                fs.led_off();
            }
        }
        fs.led_off();

        Ok(())
    }
}
