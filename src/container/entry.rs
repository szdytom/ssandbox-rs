use {
    super::Config,
    crate::{security::ApplySecurityPolicy, CommonResult, VoidResult},
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

fn get_container_workpath(base_path: &String, uid: u64) -> std::path::PathBuf {
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
    fs::create_dir_all(root)?;
    Ok(())
}

fn mount_filesystem(config: Arc<Config>) -> VoidResult {
    let container_workpath = get_container_workpath(&config.working_path, config.uid);
    let container_rootpath = container_workpath.join("target");

    create_rootdir(&container_rootpath)?;
    mark_mount_ns_private()?;

    // mounts before chroot
    for x in config.fs.iter() {
        x.loading(&container_rootpath, &container_workpath)?;
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
    let cstyle_target = CString::new(config.target_executable.to_string())?;
    unistd::execve::<_, CString>(&cstyle_target, &[cstyle_target.as_c_str()], &[])?;

    unreachable!()
}

fn block_until_ready(p: RawFd) -> VoidResult {
    unistd::read(p, &mut Vec::new())?;
    Ok(())
}

fn apply_security_policy(policies: &Vec<Box<dyn ApplySecurityPolicy>>) -> VoidResult {
    for policy in policies.iter() {
        policy.apply()?;
    }
    Ok(())
}

fn redirect_standard_io(config: Arc<Config>) -> VoidResult {
    const STDIN_FN: RawFd = 0;
    const STDOUT_FN: RawFd = 1;
    const STDERR_FN: RawFd = 2;

    use nix::{fcntl::OFlag, sys::stat::Mode};
    fn open_input(path: std::path::PathBuf) -> CommonResult<RawFd> {
        Ok(nix::fcntl::open(
            &path,
            OFlag::O_RDONLY | OFlag::O_CLOEXEC,
            Mode::empty(),
        )?)
    }

    fn open_output(path: std::path::PathBuf) -> CommonResult<RawFd> {
        Ok(nix::fcntl::open(
            &path,
            OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_TRUNC | OFlag::O_CLOEXEC,
            Mode::from_bits_truncate(0o644),
        )?)
    }

    fn do_redirect(source: RawFd, target: RawFd) -> VoidResult {
        unistd::dup2(source, target)?;
        unistd::close(source)?;
        Ok(())
    }

    if let Some(p) = &config.stdin {
        let fd = open_input(std::path::PathBuf::from(p))?;
        do_redirect(fd, STDIN_FN)?;
    }

    if let Some(p) = &config.stdout {
        let fd = open_output(std::path::PathBuf::from(p))?;
        do_redirect(fd, STDOUT_FN)?;
    }

    if let Some(p) = &config.stderr {
        let fd = open_output(std::path::PathBuf::from(p))?;
        do_redirect(fd, STDERR_FN)?;
    }

    Ok(())
}

fn check_init(config: Arc<Config>) -> VoidResult {
    unistd::access::<str>(&config.target_executable, unistd::AccessFlags::X_OK)?;
    Ok(())
}

fn exceptable_main(config: Arc<Config>, ready_pipe: RawFd, report_pipe: RawFd) -> NeverResult {
    set_hostname(&config.hostname)?;
    redirect_standard_io(config.clone())?;
    mount_filesystem(config.clone())?;
    apply_security_policy(&config.security_policies)?;
    check_init(config.clone())?;

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
        Err(err) => {
            println!("Entry Error:\n{}\nEnd.\n", err);
            unistd::write(report_pipe, &[1]);
            let report_msg = format!("{}", err).to_string();
            let report_msg_buf = report_msg.as_bytes();
            unistd::write(report_pipe, &report_msg_buf.len().to_ne_bytes());
            unistd::write(report_pipe, report_msg_buf);
            -1
        }
        _ => unreachable!(),
    }
}
