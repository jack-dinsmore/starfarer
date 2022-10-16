#![allow(dead_code)]

#[cfg(test)]
mod tests;

pub mod galaxy;
pub mod sky;

mod prelude {
    pub use crate::galaxy::{Galaxy, Direction};
    pub use crate::sky::Sky;
}