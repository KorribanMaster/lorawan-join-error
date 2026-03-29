//! Common types and enums used across Victron devices

/// Device operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OperationMode {
    Off = 0,
    LowPower = 1,
    Fault = 2,
    Bulk = 3,
    Absorption = 4,
    Float = 5,
    Storage = 6,
    Equalize = 7,
    Inverting = 9,
    PowerSupply = 11,
    StartingUp = 245,
    RepeatedAbsorption = 246,
    AutoEqualize = 247,
    BatterySafe = 248,
    ExternalControl = 252,
}

impl OperationMode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            1 => Some(Self::LowPower),
            2 => Some(Self::Fault),
            3 => Some(Self::Bulk),
            4 => Some(Self::Absorption),
            5 => Some(Self::Float),
            6 => Some(Self::Storage),
            7 => Some(Self::Equalize),
            9 => Some(Self::Inverting),
            11 => Some(Self::PowerSupply),
            245 => Some(Self::StartingUp),
            246 => Some(Self::RepeatedAbsorption),
            247 => Some(Self::AutoEqualize),
            248 => Some(Self::BatterySafe),
            252 => Some(Self::ExternalControl),
            _ => None,
        }
    }
}

/// Charger error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ChargerError {
    NoError = 0,
    BatteryTemperatureTooHigh = 1,
    BatteryVoltageTooHigh = 2,
    BatteryTemperatureSensorMiswired = 3,
    RemoteTemperatureSensorFailure = 4,
    RemoteTemperatureSensorMiswired = 5,
    RemoteVoltageSenseMiswired = 6,
    RemoteVoltageWireLost = 7,
    ChargerTemperatureTooHigh = 17,
    ChargerOverCurrent = 18,
    ChargerCurrentReversed = 19,
    BulkTimeLimitExceeded = 20,
    CurrentSensorIssue = 21,
    InternalTemperatureSensorFailure = 26,
    FanFailure = 27,
    InternalDCVoltageError = 28,
    InternalSupplyVoltageError = 29,
    InternalBatteryVoltageSensorError = 33,
    InternalDCVoltageSensorError = 34,
    PVInputShutdownExcessiveCurrent = 35,
    PVInputShutdownOverVoltage = 36,
    PVInputShutdown = 38,
    PVInputFailedToShutdown = 39,
    InverterShutdownPanelVoltage = 65,
    InverterShutdownVoltageRange = 66,
    InverterShutdownWiring = 67,
    InverterShutdownConverterIssue = 68,
    InverterShutdownOverCurrent = 114,
    InverterShutdownBatteryVoltage = 116,
    InverterShutdownHighBatteryVoltage = 117,
    InverterOverload = 119,
    CPUTemperatureTooHigh = 121,
    CommunicationLost = 200,
    SynchronizationCalibration = 201,
    BmsTempTransmitError = 202,
    BMSConnectionLost = 203,
}

impl ChargerError {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::NoError),
            1 => Some(Self::BatteryTemperatureTooHigh),
            2 => Some(Self::BatteryVoltageTooHigh),
            3 => Some(Self::BatteryTemperatureSensorMiswired),
            4 => Some(Self::RemoteTemperatureSensorFailure),
            5 => Some(Self::RemoteTemperatureSensorMiswired),
            6 => Some(Self::RemoteVoltageSenseMiswired),
            7 => Some(Self::RemoteVoltageWireLost),
            17 => Some(Self::ChargerTemperatureTooHigh),
            18 => Some(Self::ChargerOverCurrent),
            19 => Some(Self::ChargerCurrentReversed),
            20 => Some(Self::BulkTimeLimitExceeded),
            21 => Some(Self::CurrentSensorIssue),
            26 => Some(Self::InternalTemperatureSensorFailure),
            27 => Some(Self::FanFailure),
            28 => Some(Self::InternalDCVoltageError),
            29 => Some(Self::InternalSupplyVoltageError),
            33 => Some(Self::InternalBatteryVoltageSensorError),
            34 => Some(Self::InternalDCVoltageSensorError),
            35 => Some(Self::PVInputShutdownExcessiveCurrent),
            36 => Some(Self::PVInputShutdownOverVoltage),
            38 => Some(Self::PVInputShutdown),
            39 => Some(Self::PVInputFailedToShutdown),
            65 => Some(Self::InverterShutdownPanelVoltage),
            66 => Some(Self::InverterShutdownVoltageRange),
            67 => Some(Self::InverterShutdownWiring),
            68 => Some(Self::InverterShutdownConverterIssue),
            114 => Some(Self::InverterShutdownOverCurrent),
            116 => Some(Self::InverterShutdownBatteryVoltage),
            117 => Some(Self::InverterShutdownHighBatteryVoltage),
            119 => Some(Self::InverterOverload),
            121 => Some(Self::CPUTemperatureTooHigh),
            200 => Some(Self::CommunicationLost),
            201 => Some(Self::SynchronizationCalibration),
            202 => Some(Self::BmsTempTransmitError),
            203 => Some(Self::BMSConnectionLost),
            _ => None,
        }
    }
}

/// AC input state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ACInState {
    NotConnected = 0,
    Connected = 1,
    Unknown = 2,
}

impl ACInState {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::NotConnected),
            1 => Some(Self::Connected),
            2 => Some(Self::Unknown),
            _ => None,
        }
    }
}

/// Alarm notification level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum AlarmNotification {
    Off = 0,
    Alarm = 1,
    Warning = 2,
}

impl AlarmNotification {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            1 => Some(Self::Alarm),
            2 => Some(Self::Warning),
            _ => None,
        }
    }
}

/// Alarm reason flags (bitfield)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AlarmReason(pub u32);

impl AlarmReason {
    pub const NONE: u32 = 0;
    pub const LOW_VOLTAGE: u32 = 1 << 0;
    pub const HIGH_VOLTAGE: u32 = 1 << 1;
    pub const LOW_SOC: u32 = 1 << 2;
    pub const LOW_STARTER_VOLTAGE: u32 = 1 << 3;
    pub const HIGH_STARTER_VOLTAGE: u32 = 1 << 4;
    pub const LOW_TEMPERATURE: u32 = 1 << 5;
    pub const HIGH_TEMPERATURE: u32 = 1 << 6;
    pub const MID_VOLTAGE: u32 = 1 << 7;
    pub const OVERLOAD: u32 = 1 << 8;
    pub const DC_RIPPLE: u32 = 1 << 9;
    pub const LOW_AC_OUT_VOLTAGE: u32 = 1 << 10;
    pub const HIGH_AC_OUT_VOLTAGE: u32 = 1 << 11;

    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn has_flag(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }
}

/// Off reason flags (bitfield)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OffReason(pub u32);

impl OffReason {
    pub const NONE: u32 = 0;
    pub const NO_INPUT_POWER: u32 = 1 << 0;
    pub const SWITCHED_OFF_SWITCH: u32 = 1 << 1;
    pub const SWITCHED_OFF_REGISTER: u32 = 1 << 2;
    pub const REMOTE_INPUT: u32 = 1 << 3;
    pub const PROTECTION_ACTIVE: u32 = 1 << 4;
    pub const PAYGO: u32 = 1 << 5;
    pub const BMS: u32 = 1 << 6;
    pub const ENGINE_SHUTDOWN: u32 = 1 << 7;
    pub const ANALYSING_INPUT_VOLTAGE: u32 = 1 << 8;

    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn has_flag(&self, flag: u32) -> bool {
        (self.0 & flag) != 0
    }
}

/// Output state for Smart Battery Protect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OutputState {
    On = 1,
    Off = 4,
}

impl OutputState {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::On),
            4 => Some(Self::Off),
            _ => None,
        }
    }
}

/// Balancer status for Smart Lithium
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum BalancerStatus {
    Unknown = 0,
    Balanced = 1,
    Balancing = 2,
    Imbalance = 3,
}

impl BalancerStatus {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Unknown),
            1 => Some(Self::Balanced),
            2 => Some(Self::Balancing),
            3 => Some(Self::Imbalance),
            _ => None,
        }
    }
}

/// Meter type for DC Energy Meter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(i8)]
pub enum MeterType {
    SolarCharger = -9,
    WindCharger = -8,
    ShaftGenerator = -7,
    Alternator = -6,
    FuelCell = -5,
    WaterGenerator = -4,
    DcDcCharger = -3,
    AcCharger = -2,
    GenericSource = -1,
    GenericLoad = 1,
    ElectricDrive = 2,
    Fridge = 3,
    WaterPump = 4,
    BilgePump = 5,
    DcSystem = 6,
    Inverter = 7,
    WaterHeater = 8,
}

impl MeterType {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            -9 => Some(Self::SolarCharger),
            -8 => Some(Self::WindCharger),
            -7 => Some(Self::ShaftGenerator),
            -6 => Some(Self::Alternator),
            -5 => Some(Self::FuelCell),
            -4 => Some(Self::WaterGenerator),
            -3 => Some(Self::DcDcCharger),
            -2 => Some(Self::AcCharger),
            -1 => Some(Self::GenericSource),
            1 => Some(Self::GenericLoad),
            2 => Some(Self::ElectricDrive),
            3 => Some(Self::Fridge),
            4 => Some(Self::WaterPump),
            5 => Some(Self::BilgePump),
            6 => Some(Self::DcSystem),
            7 => Some(Self::Inverter),
            8 => Some(Self::WaterHeater),
            _ => None,
        }
    }
}

/// Device type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum DeviceType {
    SolarCharger = 0x01,
    BatteryMonitor = 0x02,
    Inverter = 0x03,
    DcDcConverter = 0x04,
    SmartLithium = 0x05,
    AcCharger = 0x08,
    SmartBatteryProtect = 0x09,
    LynxSmartBMS = 0x0A,
    VEBus = 0x0C,
    DcEnergyMeter = 0x0D,
    OrionXS = 0x0F,
}

impl DeviceType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::SolarCharger),
            0x02 => Some(Self::BatteryMonitor),
            0x03 => Some(Self::Inverter),
            0x04 => Some(Self::DcDcConverter),
            0x05 => Some(Self::SmartLithium),
            0x08 => Some(Self::AcCharger),
            0x09 => Some(Self::SmartBatteryProtect),
            0x0A => Some(Self::LynxSmartBMS),
            0x0C => Some(Self::VEBus),
            0x0D => Some(Self::DcEnergyMeter),
            0x0F => Some(Self::OrionXS),
            _ => None,
        }
    }
}
