use crate::alp_binding::*;
use std::error::Error;
use strum_macros::FromRepr;
use std::fmt::{self, Formatter, Display};



pub type AlpResult<T> = Result<T, AlpError>;

#[repr(i64)]
#[derive(Copy, Clone, Debug, PartialEq, FromRepr)]
pub enum AlpError {
    NotOnline = ALP_NOT_ONLINE as i64,
    NotIdle = ALP_NOT_IDLE as i64,
    NotAvailable = ALP_NOT_AVAILABLE as i64,
    NotReady = ALP_NOT_READY as i64,
    ParameterInvalid = ALP_PARM_INVALID as i64,
    AddressInvalid = ALP_ADDR_INVALID as i64,
    MemoryFull = ALP_MEMORY_FULL as i64,
    SequenceInUse = ALP_SEQ_IN_USE as i64,
    Halted = ALP_HALTED as i64,
    InitFail = ALP_ERROR_INIT as i64,
    CommunicationFail = ALP_ERROR_COMM as i64,
    DeviceRemoved = ALP_DEVICE_REMOVED as i64,
    NotConfigured = ALP_NOT_CONFIGURED as i64,
    LoaderVersion = ALP_LOADER_VERSION as i64,
    PoweredDown = ALP_ERROR_POWER_DOWN as i64,
    DriverVersion = ALP_DRIVER_VERSION as i64,
    SdramInitFail = ALP_SDRAM_INIT as i64,
    ConfigMismatch = ALP_CONFIG_MISMATCH as i64,
    Unknown = ALP_ERROR_UNKNOWN as i64
}

impl Display for AlpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{self:?}")
    }
}

impl Error for AlpError {}
