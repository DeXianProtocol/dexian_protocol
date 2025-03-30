use scrypto::prelude::*;

// Include the generated constants
include!(concat!(env!("OUT_DIR"), "/env_constants.rs"));

#[cfg(feature = "mainnet")]
pub const BABYLON_START_EPOCH: u64 = 32718; // //mainnet: 32718, stokenet: 0
#[cfg(feature = "stokenet")]
pub const BABYLON_START_EPOCH: u64 = 0;
#[cfg(not(any(feature = "mainnet", feature = "stokenet")))]
pub const BABYLON_START_EPOCH: u64 = 0; // Default value if no feature is enabled


pub const EPOCH_OF_YEAR: u64 = 105120; // 5*24*7*52
pub const A_WEEK_EPOCHS: u64 = 2016; //60/5*24*7;
pub const RESERVE_WEEKS: usize = 4;
pub const TO_INFINITY: WithdrawStrategy = WithdrawStrategy::Rounded(RoundingMode::ToPositiveInfinity);
pub const TO_ZERO: WithdrawStrategy = WithdrawStrategy::Rounded(RoundingMode::ToZero);
