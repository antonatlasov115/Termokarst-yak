//! Физические расчеты для термокарстовых процессов

pub mod frozen_ground;
pub mod heat_transfer;
pub mod heat_transfer_2d;
pub mod nfactor;
pub mod phase_transition;
pub mod richards;
pub mod snow;
pub mod subsidence;
pub mod surface_energy;
pub mod thaw;
pub mod thawing_index;
pub mod thermal_conductivity;

pub use frozen_ground::*;
pub use heat_transfer::*;
pub use heat_transfer_2d::*;
pub use nfactor::*;
pub use phase_transition::*;
pub use richards::*;
pub use snow::*;
pub use subsidence::*;
pub use thaw::*;
pub use thawing_index::*;
pub use thermal_conductivity::*;

// Re-export SurfaceType from surface_energy with alias to avoid conflict
pub use surface_energy::SurfaceType as SurfaceEnergyType;
pub use surface_energy::{
    DailyEnergyBalance, FullDailyBalance, FullEnergyBalance, SurfaceEnergyBalance,
};
