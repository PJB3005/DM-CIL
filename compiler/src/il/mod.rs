//! Handles the outputting and handling of CIL.

pub mod assembly;
pub mod class;
pub mod instructions;
pub mod method;
pub mod field;

pub use self::assembly::*;
pub use self::class::*;
pub use self::instructions::*;
pub use self::field::*;
pub use self::method::*;
