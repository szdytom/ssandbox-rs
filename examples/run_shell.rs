// You need to have a alpine image @ /root/sandbox/image to run this example
// You will get a shell inside the container.

#![feature(box_syntax)]
#![feature(type_ascription)]

use ssandbox::{container::{Config, Container}, filesystem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config: Config = Default::default();
    config.fs.push(box filesystem::MountSizedTmpFs::from(2 * 1024 * 1024 * 1024));
    config.fs.push(box filesystem::MountProcFs);
    config.fs.push(box filesystem::MountReadOnlyBindFs::from("/root/sandbox/image".to_string()));
    config.fs.push(box filesystem::MountExtraFs::new());
    config.cgroup_limits.set_fork_limit(10);
    config.cgroup_limits.set_memory_limit(512 * 1024 * 1024); // 512Mb
    let mut c = Container::from(config);
    c.start()?;
    c.wait()?;
    println!("Finished!");
    Ok(())
}
