#![feature(box_syntax)]

use ssandbox::{container::{Config, Container}, filesystem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config: Config = Default::default();
    config.fs.push(box filesystem::MountTmpFs);
    config.fs.push(box filesystem::MountProcFs);
    config.fs.push(box filesystem::MountReadOnlyBindFs::from("/root/sandbox/image".to_string()));
    config.fs.push(box filesystem::MountExtraFs::new());
    config.cgroup_limits.set_fork_limit(3);
    let mut c = Container::from(config);
    c.start()?;
    c.wait()?;
    println!("Finished!");
    Ok(())
}
