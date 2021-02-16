#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(never_type)]

extern crate rand;
extern crate nix;

pub mod container;
pub mod filesystem;

type CommonResult<T> = Result<T, Box<dyn std::error::Error>>;
type VoidResult = CommonResult<()>;