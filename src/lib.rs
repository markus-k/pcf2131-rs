#![no_std]

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
pub use rtcc::DateTimeAccess;

pub trait RegisterAccess {
    type Error;

    fn write_register(&mut self, register: u8, value: u8) -> Result<(), Self::Error>;

    /// Write multiple registers. The first value on `reg_and_values` is the start register.
    fn write_registers(&mut self, reg_and_values: &[u8]) -> Result<(), Self::Error>;

    fn read_register(&mut self, register: u8) -> Result<u8, Self::Error>;
    fn read_registers(&mut self, start: u8, buf: &mut [u8]) -> Result<(), Self::Error>;
}

pub struct I2CInterface<I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C> RegisterAccess for I2CInterface<I2C>
where
    I2C: embedded_hal::i2c::I2c,
{
    type Error = I2C::Error;

    fn write_register(&mut self, register: u8, value: u8) -> Result<(), Self::Error> {
        self.i2c.write(self.address, &[register, value])?;

        Ok(())
    }

    fn read_register(&mut self, register: u8) -> Result<u8, Self::Error> {
        let mut buf = [0u8; 1];
        self.i2c.write_read(self.address, &[register], &mut buf)?;

        Ok(buf[0])
    }

    fn write_registers(&mut self, reg_and_values: &[u8]) -> Result<(), Self::Error> {
        self.i2c.write(self.address, reg_and_values)?;

        Ok(())
    }

    fn read_registers(&mut self, start: u8, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write_read(self.address, &[start], buf)?;

        Ok(())
    }
}

pub struct Pcf2131<I> {
    interface: I,
}

impl<I> Pcf2131<I>
where
    I: RegisterAccess,
{
    pub fn new(interface: I) -> Self {
        Self { interface }
    }

    pub fn set_clockout(&mut self, freq: ClockoutFrequency) -> Result<(), I::Error> {
        let mut clkcout_ctl = self.interface.read_register(Registers::CLOCKOUT_CTL)?;
        clkcout_ctl &= !0b111;
        clkcout_ctl |= freq.to_regval();
        self.interface
            .write_register(Registers::CLOCKOUT_CTL, clkcout_ctl)?;

        Ok(())
    }

    pub fn set_stop(&mut self, stop: bool) -> Result<(), I::Error> {
        let mut control_1 = self.interface.read_register(Registers::CONTROL_1)?;
        if stop {
            control_1 |= 1 << 5;
        } else {
            control_1 &= !(1 << 5);
        }
        self.interface
            .write_register(Registers::CONTROL_1, control_1)?;
        Ok(())
    }

    pub fn clear_prescaler(&mut self) -> Result<(), I::Error> {
        self.interface.write_register(Registers::SR_RESET, 0xA4)?;

        Ok(())
    }
}

pub enum ClockoutFrequency {
    Hz32768,
    Hz16384,
    Hz8192,
    Hz4096,
    Hz2048,
    Hz1024,
    Hz1,
    HighZ,
}

impl ClockoutFrequency {
    fn to_regval(self) -> u8 {
        match self {
            ClockoutFrequency::Hz32768 => 0b000,
            ClockoutFrequency::Hz16384 => 0b001,
            ClockoutFrequency::Hz8192 => 0b010,
            ClockoutFrequency::Hz4096 => 0b011,
            ClockoutFrequency::Hz2048 => 0b100,
            ClockoutFrequency::Hz1024 => 0b101,
            ClockoutFrequency::Hz1 => 0b110,
            ClockoutFrequency::HighZ => 0b111,
        }
    }
}

impl<I> DateTimeAccess for Pcf2131<I>
where
    I: RegisterAccess,
{
    type Error = I::Error;

    fn datetime(&mut self) -> Result<chrono::prelude::NaiveDateTime, Self::Error> {
        let mut buffer = [0; 8];

        self.interface
            .read_registers(Registers::SECONDS_100TH, &mut buffer)?;

        Ok(chrono::prelude::NaiveDateTime::new(
            NaiveDate::from_ymd_opt(
                buffer[7].as_bcd() as i32 + 2000,
                buffer[6].as_bcd() as u32,
                buffer[4].as_bcd() as u32,
            )
            .unwrap(),
            NaiveTime::from_hms_milli_opt(
                buffer[3].as_bcd() as u32,
                buffer[2].as_bcd() as u32,
                (buffer[1] & 0x7F).as_bcd() as u32,
                buffer[0].as_bcd() as u32 * 10,
            )
            .unwrap(),
        ))
    }

    fn set_datetime(
        &mut self,
        datetime: &chrono::prelude::NaiveDateTime,
    ) -> Result<(), Self::Error> {
        let buffer = [
            Registers::SECONDS_100TH,
            0,
            (datetime.time().second() as u8).to_bcd(),
            (datetime.time().minute() as u8).to_bcd(),
            (datetime.time().hour() as u8).to_bcd(),
            (datetime.date().day() as u8).to_bcd(),
            // weekday?
            0,
            (datetime.date().month() as u8).to_bcd(),
            ((datetime.date().year() - 2000) as u8).to_bcd(),
        ];

        self.set_stop(true)?;
        self.clear_prescaler()?;
        self.interface.write_registers(&buffer)?;
        self.set_stop(false)?;

        todo!()
    }
}

trait AsBcd {
    /// Convert the number to Binary Coded Decimal representation
    fn to_bcd(self) -> Self;

    /// Convert the number from Binary Coded Decimal representation
    fn as_bcd(self) -> Self;
}

impl AsBcd for u8 {
    fn to_bcd(self) -> Self {
        self % 10 | (self / 10) << 4
    }

    fn as_bcd(self) -> Self {
        (self & 0xf) + (self >> 4) * 10
    }
}

struct Registers;

#[allow(dead_code)]
impl Registers {
    pub const CONTROL_1: u8 = 0x00;
    pub const CONTROL_2: u8 = 0x01;
    pub const CONTROL_3: u8 = 0x02;
    pub const CONTROL_4: u8 = 0x03;
    pub const CONTROL_5: u8 = 0x04;
    pub const SR_RESET: u8 = 0x05;
    pub const SECONDS_100TH: u8 = 0x06;
    pub const SECONDS: u8 = 0x07;
    pub const MINUTES: u8 = 0x08;
    pub const HOURS: u8 = 0x09;
    pub const DAYS: u8 = 0x0A;
    pub const WEEKDAYS: u8 = 0x0B;
    pub const MONTHS: u8 = 0x0C;
    pub const YEARS: u8 = 0x0D;

    pub const CLOCKOUT_CTL: u8 = 0x13;
    pub const AGING_OFFSET: u8 = 0x30;
}

mod tests {
    use super::*;

    #[test]
    fn test_bcd() {
        assert_eq!(9u8.to_bcd(), 0x9);
        assert_eq!(19u8.to_bcd(), 0x19);
        assert_eq!(99u8.to_bcd(), 0x99);

        assert_eq!(0x9u8.as_bcd(), 9);
        assert_eq!(0x19u8.as_bcd(), 19);
        assert_eq!(0x99u8.as_bcd(), 99);
    }
}
