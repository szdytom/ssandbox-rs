use {crate::VoidResult};

pub mod cap;
pub mod seccomp;

pub trait ApplySecurityPolicy: std::fmt::Debug {
    fn apply(&self) -> VoidResult;
}

pub use cap::CapabilityPolicy;
pub use seccomp::SeccompPolicy;