//! Core types for the simulation engine.

pub mod config;
pub mod result;
pub mod trade_info;
pub mod wad;

pub use config::{PoolConfigV2, SimulationConfig, SimulationConfigV2};
pub use result::{
    BatchSimulationResult, BatchSimulationResultV2, LightweightSimResult, LightweightSimResultV2,
    LightweightStepResult, PoolStateV2,
};
pub use trade_info::{TradeInfo, TradeInfoV2};
pub use wad::Wad;
