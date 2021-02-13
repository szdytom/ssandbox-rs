use {
    crate::filesystem::MountNamespacedFs,
    nix::{mount::MsFlags, unistd::Pid},
    std::{ffi::CString, sync::Arc},
};

mod error;

#[derive(Debug)]
pub struct Config {
    pub uid: u64,
    pub working_path: String,
    pub stack_size: usize,
    pub hostname: String,
    pub target_executable: String,
    pub fs: Vec<Box<dyn MountNamespacedFs>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            uid: rand::random(),
            working_path: "/root/sandbox/work".to_string(),
            stack_size: 16 * 1024, // 16kb
            hostname: "container".to_string(),
            target_executable: "/bin/sh".into(),
            fs: Vec::new(),
        }
    }
}


#[derive(Debug, Clone)]
struct InternalData {
    config: Arc<Config>,
}

#[allow(unused_must_use)]
fn container_entry_handle(cfg: InternalData) -> ! {
    let config = cfg.config;
    nix::unistd::sethostname(config.hostname.clone());
    
    use std::path::Path;
    let container_rootpath: &String = &config.working_path;
    let container_rootpath = Path::new(container_rootpath);
    let container_rootpath = container_rootpath.join(format!("{}/", config.uid));
    let container_rootpath = Path::new(&container_rootpath);

    if !container_rootpath.exists() {
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(container_rootpath).unwrap();
    }

    nix::mount::mount::<str, str, str, str>(
        None,
        "/",
        None,
        MsFlags::MS_REC | MsFlags::MS_PRIVATE,
        None,
    );
    for x in config.fs.iter() {
        x.loading(container_rootpath);
    }
    nix::unistd::chroot(container_rootpath);
    for x in config.fs.iter() {
        x.loaded();
    }
    
    let cstyle_target = CString::new(config.target_executable.to_string()).unwrap();
    nix::unistd::execv(&cstyle_target, &[cstyle_target.as_c_str()]);

    unreachable!()
}

#[derive(Debug, Clone)]
pub struct Container {
    stack_memory: Vec<u8>,
    config: Arc<Config>,
    container_pid: Option<Pid>,
}

impl std::convert::From<Config> for Container {
    fn from(source: Config) -> Self {
        Self {
            stack_memory: Vec::new(),
            config: Arc::new(source),
            container_pid: None,
        }
    }
}

impl Container {
    pub fn new() -> Self {
        Self {
            stack_memory: Vec::new(),
            config: Arc::new(Default::default()),
            container_pid: None,
        }
    }

    pub fn has_started(&self) -> bool {
        self.container_pid != None
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.has_started() {
            return Err(box error::Error::AlreadyStarted);
        }

        self.stack_memory.resize(self.config.stack_size, 0);

        let ic = InternalData {
            config: self.config.clone(),
        };

        use nix::sched::CloneFlags;
        self.container_pid = Some(match nix::sched::clone(
            box || container_entry_handle(ic.clone()),
            self.stack_memory.as_mut(),
            CloneFlags::CLONE_NEWUTS
                | CloneFlags::CLONE_NEWIPC
                | CloneFlags::CLONE_NEWPID
                | CloneFlags::CLONE_NEWNS,
            Some(nix::sys::signal::SIGCHLD.into()),
        ) {
            Ok(x) => x,
            Err(e) => return Err(box error::Error::ForkFailed(e)),
        });

        Ok(())
    }

    pub fn wait(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.has_started() {
            nix::sys::wait::waitpid(self.container_pid, None)?;
        }
        Ok(())
    }
}
