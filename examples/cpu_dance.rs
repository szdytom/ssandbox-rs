// You need to have a alpine image @ /root/sandbox/image to run this example
// You will get a shell inside the container.
// You will also need a command named "loop" @ /bin/loop inside the root image.
// The source of loop can be found @ https://gist.githubusercontent.com/szdytom/ec2d99a41477787a8c85f6ec73af4ca2/raw/54fb722cb3ed60ef902573ee2ce503d8760ec723/loop.c

#![feature(box_syntax)]

use ssandbox::{container::{Config, Container}, filesystem};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config: Config = Default::default();
    config.fs.push(box filesystem::MountTmpFs);
    config.fs.push(box filesystem::MountProcFs);
    config.fs.push(box filesystem::MountReadOnlyBindFs::from("/root/sandbox/image".to_string()));
    config.fs.push(box filesystem::MountExtraFs::new());
    config.cgroup_limits.set_fork_limit(3);
    config.target_executable = "/bin/loop".to_string();
    let mut c = Container::from(config);
    c.start()?;
    for _i in 0..15 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        c.freeze()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        c.thaw()?;
    }
    c.terminate()?;
    println!("Finished!");
    Ok(())
}
