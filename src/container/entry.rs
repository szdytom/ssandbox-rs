use {
    super::Config,
    crate::{CommonResult, VoidResult},
    nix::mount::{self, MsFlags},
    std::{ffi::CString, fs, sync::Arc},
};

type NeverResult = CommonResult<!>;

#[derive(Debug, Clone)]
pub struct InternalData {
    pub config: Arc<Config>,
}

fn set_hostname(hostname: &String) -> VoidResult {
    nix::unistd::sethostname(hostname)?;
    Ok(())
}

fn get_container_rootpath(base_path: &String, uid: u64) -> std::path::PathBuf {
    [base_path, &uid.to_string()].iter().collect()
}

fn mark_mount_ns_private() -> VoidResult {
    mount::mount::<str, str, str, str>(
        None,
        "/",
        None,
        MsFlags::MS_REC | MsFlags::MS_PRIVATE,
        None,
    )?;
    Ok(())
}

fn change_rootpath(root: &std::path::Path) -> VoidResult {
    nix::unistd::chdir(root)?;
    nix::unistd::chroot(".")?;
    Ok(())
}

fn create_rootdir(root: &std::path::Path) -> VoidResult {
    if root.exists() {
        fs::remove_dir_all(root)?;
    }
    fs::DirBuilder::new().recursive(true).create(root)?;
    Ok(())
}

fn mount_filesystem(config: Arc<Config>) -> VoidResult {
    let container_rootpath = get_container_rootpath(&config.working_path, config.uid);

    create_rootdir(&container_rootpath)?;
    mark_mount_ns_private()?;

    // mounts before chroot
    for x in config.fs.iter() {
        x.loading(&container_rootpath)?;
    }

    // chroot
    change_rootpath(&container_rootpath)?;

    // mounts after chroot
    for x in config.fs.iter() {
        x.loaded()?;
    }

    Ok(())
}

fn run_init(config: Arc<Config>) -> NeverResult {
    let cstyle_target = CString::new(config.target_executable.to_string()).unwrap();
    nix::unistd::execv(&cstyle_target, &[cstyle_target.as_c_str()])?;

    unreachable!()
}

fn exceptable_main(cfg: InternalData) -> NeverResult {
    let config = cfg.config;
    set_hostname(&config.hostname)?;
    mount_filesystem(config.clone())?;
    run_init(config)
}

pub fn main(cfg: InternalData) -> ! {
    match exceptable_main(cfg) {
        Err(x) => println!("{:?}\n", x),
        _ => unreachable!(),
    };
    unreachable!()
}
