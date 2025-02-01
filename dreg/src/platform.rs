//! Platform



pub mod native;
pub mod web;



pub use native::*;
pub use web::*;


pub trait Platform {}
