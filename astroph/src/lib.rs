#![allow(dead_code)]

#[cfg(test)]
mod tests;

pub mod galaxy;

mod prelude {
    pub use crate::galaxy::{Galaxy, Direction};
}