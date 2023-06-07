pub mod contract;
mod error;
pub mod helpers;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

#[no_mangle]
extern "C" fn requires_alliance() {}
