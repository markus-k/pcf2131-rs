pub struct Registers;

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
