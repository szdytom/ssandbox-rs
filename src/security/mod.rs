use {crate::VoidResult};

pub mod cap;

pub trait ApplySecurityPolicy: std::fmt::Debug {
    fn apply(&self) -> VoidResult;
}

pub use cap::CapabilityPolicy;