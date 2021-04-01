#![feature(box_syntax)]
#![feature(box_patterns)]
#![feature(never_type)]
#![feature(type_ascription)]

extern crate rand;
extern crate nix;
extern crate libscmp;
extern crate caps;
extern crate cgroups_rs;

pub mod container;
pub mod filesystem;
pub mod security;
pub mod resource;
mod idmap;

type CommonResult<T> = Result<T, Box<dyn std::error::Error>>;
type VoidResult = CommonResult<()>;
