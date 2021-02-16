use {
    crate::container::Config,
    nix::mount::{self, MsFlags},
    std::{ffi::CString, sync::Arc},
};

#[derive(Debug, Clone)]
pub struct InternalData {
    pub config: Arc<Config>,
}

fn exceptable_main(cfg: InternalData) -> Result<!, Box<dyn std::error::Error>> {
    let config = cfg.config;

    nix::unistd::sethostname(config.hostname.clone())?;
    
    use std::path::Path;
    let container_rootpath: &String = &config.working_path;
    let container_rootpath = Path::new(container_rootpath);
    let container_rootpath = container_rootpath.join(format!("{}/", config.uid));
    let container_rootpath = Path::new(&container_rootpath);

    if !container_rootpath.exists() {
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(container_rootpath)?;
    }

    mount::mount::<str, str, str, str>(
        None,
        "/",
        None,
        MsFlags::MS_REC | MsFlags::MS_PRIVATE,
        None,
    )?;
    for x in config.fs.iter() {
        x.loading(container_rootpath)?;
    }
    nix::unistd::chdir(container_rootpath)?;
    nix::unistd::chroot("./")?;
    for x in config.fs.iter() {
        x.loaded()?;
    }
    let cstyle_target = CString::new(config.target_executable.to_string()).unwrap();
    nix::unistd::execv(&cstyle_target, &[cstyle_target.as_c_str()])?;

    unreachable!()
}

pub fn main(cfg: InternalData) -> ! {
    match exceptable_main(cfg) {
        Err(x) => println!("{:?}\n", x),
        _ => unreachable!(),
    };
    
    unreachable!()
}
