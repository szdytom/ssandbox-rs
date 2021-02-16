#![feature(box_syntax)]

use ssandbox::{container::{Config, Container}, filesystem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config: Config = Default::default();
    config.fs.push(box filesystem::MountTmpFs);
    config.fs.push(box filesystem::MountProcFs);
    config.fs.push(box filesystem::MountBindFs::from("/root/sandbox/image".to_string()));
    let mut c = Container::from(config);
    c.start()?;
    c.wait()?;
    Ok(())
}
