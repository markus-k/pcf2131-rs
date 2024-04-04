#![no_std]

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};
use embedded_hal::i2c::I2c;

// re-export `DateTImeAccess` so crates don't have to depend on `rtcc`
pub use rtcc::DateTimeAccess;

mod registers;
mod types;

use crate::registers::Registers;

pub use types::{ClockoutFrequency, PowerManagement};

/// Default I2C address of the PCF2131
pub const DEFAULT_I2C_ADDRESS: u8 = 0x53;

/// Trait that provides access to register values of the RTC
pub trait RegisterAccess {
    type Error;

    fn write_register(&mut self, register: u8, value: u8) -> Result<(), Self::Error> {
        self.write_registers(&[register, value])
    }

    /// Write multiple registers. The first value on `reg_and_values` is the start register.
    fn write_registers(&mut self, reg_and_values: &[u8]) -> Result<(), Self::Error>;

    fn read_register(&mut self, register: u8) -> Result<u8, Self::Error> {
        let mut buf = [0u8];
        self.read_registers(register, &mut buf)?;

        Ok(buf[0])
    }

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

    fn write_registers(&mut self, reg_and_values: &[u8]) -> Result<(), Self::Error> {
        self.i2c.write(self.address, reg_and_values)?;

        Ok(())
    }

    fn read_registers(&mut self, start: u8, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c.write_read(self.address, &[start], buf)?;

        Ok(())
    }
}

/// PCF2131 driver
pub struct Pcf2131<I> {
    interface: I,
}

impl<I2C> Pcf2131<I2CInterface<I2C>>
where
    I2C: I2c,
{
    pub fn new_i2c(i2c: I2C) -> Self {
        Self::new_i2c_addr(i2c, DEFAULT_I2C_ADDRESS)
    }

    pub fn new_i2c_addr(mut i2c: I2C, address: u8) -> Self {
        // do a dummy write to the address and ignore the result.
        // This is done since the i2c interface of the pcf2131 may
        // be in some weird state if Vdd was lost during a transaction
        // but the device was still backed by a battery (see 7.16.3)
        i2c.write(address, &[]).ok();
        i2c.write(address, &[]).ok();

        Self {
            interface: I2CInterface { i2c, address },
        }
    }
}

impl<I> Pcf2131<I>
where
    I: RegisterAccess,
{
    pub fn new(interface: I) -> Self {
        Self { interface }
    }

    /// Set frequency of the CLKOUT pin
    pub fn set_clockout(&mut self, freq: ClockoutFrequency) -> Result<(), I::Error> {
        let mut clkcout_ctl = self.interface.read_register(Registers::CLOCKOUT_CTL)?;
        clkcout_ctl &= !0b111;
        clkcout_ctl |= freq.to_regval();
        self.interface
            .write_register(Registers::CLOCKOUT_CTL, clkcout_ctl)?;

        Ok(())
    }

    /// Stop the clock
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

    /// Clear the clock prescaler
    pub fn clear_prescaler(&mut self) -> Result<(), I::Error> {
        self.interface.write_register(Registers::SR_RESET, 0xA4)?;

        Ok(())
    }

    /// Set power management options for backup battery
    pub fn set_powermanagement(&mut self, mode: PowerManagement) -> Result<(), I::Error> {
        let mut control3 = self.interface.read_register(Registers::CONTROL_3)?;
        control3 &= !0b1110_0000;
        control3 |= mode.to_regval() << 5;
        self.interface
            .write_register(Registers::CONTROL_3, control3)?;

        Ok(())
    }

    /// Disable the POR override
    pub fn disable_por_override(&mut self) -> Result<(), I::Error> {
        let mut control1 = self.interface.read_register(Registers::CONTROL_1)?;
        control1 &= !(1 << 3);
        self.interface
            .write_register(Registers::CONTROL_1, control1)?;

        Ok(())
    }

    /// Perform a OTP refresh
    pub fn perform_otp_refresh(&mut self) -> Result<(), I::Error> {
        let mut clockout = self.interface.read_register(Registers::CLOCKOUT_CTL)?;
        clockout &= !(1 << 5);
        self.interface
            .write_register(Registers::CLOCKOUT_CTL, clockout)?;
        clockout |= 1 << 5;
        self.interface
            .write_register(Registers::CLOCKOUT_CTL, clockout)?;

        clockout = self.interface.read_register(Registers::CLOCKOUT_CTL)?;
        while (clockout & (1 << 5)) == 0 {
            clockout = self.interface.read_register(Registers::CLOCKOUT_CTL)?;
        }

        Ok(())
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

        Ok(())
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

#[cfg(test)]
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
