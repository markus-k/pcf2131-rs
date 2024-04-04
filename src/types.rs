/// CLKOUT frequency selection
#[derive(Debug, Clone, Copy)]
pub enum ClockoutFrequency {
    /// 32768 Hz
    Hz32768,
    /// 16384 Hz
    Hz16384,
    /// 8192 Hz
    Hz8192,
    /// 4096 Hz
    Hz4096,
    /// 2048 Hz
    Hz2048,
    /// 1024 Hz
    Hz1024,
    /// 1 Hz
    Hz1,
    /// CLKOUT is kept in High Z state
    HighZ,
}

impl ClockoutFrequency {
    pub(crate) fn to_regval(self) -> u8 {
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

/// Power management options for selecting between Vdd and Vbat
#[derive(Debug, Clone, Copy)]
pub enum PowerManagement {
    /// battery switch-over function is enabled in standard mode;
    /// battery low detection function is enabled
    StandardModeBatteryLowEnabled,

    /// battery switch-over function is enabled in standard mode;
    /// battery low detection function is disabled
    StandardModeBatteryLowDisabled,

    /// attery switch-over function is enabled in direct switching mode;
    /// battery low detection function is enabled
    DirectModeBatteryLowEnabled,

    /// battery switch-over function is enabled in direct switching mode;
    /// battery low detection function is disabled
    DirectModeBatteryLowDisabled,

    /// battery switch-over function is disabled, only one power supply (VDD);
    /// battery low detection function is disabled
    ///
    /// When the battery switch-over function is disabled, the device works
    /// only with the power supply VDD. VBAT must be put to ground and the
    /// battery low detection function is disabled
    Disabled,
}

impl PowerManagement {
    pub(crate) fn to_regval(self) -> u8 {
        match self {
            PowerManagement::StandardModeBatteryLowEnabled => 0b000,
            PowerManagement::StandardModeBatteryLowDisabled => 0b001, // and 0b010
            PowerManagement::DirectModeBatteryLowEnabled => 0b011,
            PowerManagement::DirectModeBatteryLowDisabled => 0b100, // and 0b101
            PowerManagement::Disabled => 0b110,                     // and 0b111
        }
    }
}
