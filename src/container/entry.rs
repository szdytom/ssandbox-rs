use {
    super::Config,
    crate::{CommonResult, VoidResult},
    nix::{
        mount::{self, MsFlags},
        unistd,
    },
    std::{ffi::CString, fs, os::unix::io::RawFd, sync::Arc},
};

type NeverResult = CommonResult<!>;

#[derive(Debug, Clone)]
pub struct InternalData {
    pub config: Arc<Config>,
    pub ready_pipe_set: (RawFd, RawFd),
    pub report_pipe_set: (RawFd, RawFd),
}

fn set_hostname(hostname: &String) -> VoidResult {
    unistd::sethostname(hostname)?;
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
    unistd::chdir(root)?;
    unistd::chroot(".")?;
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
    unistd::execve::<_, CString>(&cstyle_target, &[cstyle_target.as_c_str()], &[])?;

    unreachable!()
}

fn block_until_ready(p: RawFd) -> VoidResult {
    unistd::read(p, &mut Vec::new())?;
    Ok(())
}

fn exceptable_main(config: Arc<Config>, ready_pipe: RawFd, report_pipe: RawFd) -> NeverResult {
    set_hostname(&config.hostname)?;
    mount_filesystem(config.clone())?;
    block_until_ready(ready_pipe)?;
    unistd::write(report_pipe, &[0])?;
    run_init(config)
}

fn extract_pipes(rd_set: (RawFd, RawFd), rp_set: (RawFd, RawFd)) -> CommonResult<(RawFd, RawFd)> {
    let (rd_read, rd_write) = rd_set;
    let (rp_read, rp_write) = rp_set;
    unistd::close(rd_write)?;
    unistd::close(rp_read)?;
    Ok((rd_read, rp_write))
}

#[allow(unused_must_use)]
pub fn main(cfg: InternalData) -> isize {
    let (ready_pipe, report_pipe) = extract_pipes(cfg.ready_pipe_set, cfg.report_pipe_set).unwrap();
    match exceptable_main(cfg.config, ready_pipe, report_pipe) {
        Err(x) => {
            println!("Entry Error:\n{}\nEnd.\n", x);
            unistd::write(report_pipe, &[1]);
            unistd::write(report_pipe, format!("{}", x).as_bytes());
            return -1;
        }
        _ => unreachable!(),
    };
}
