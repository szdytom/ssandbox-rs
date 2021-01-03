use {nix::unistd::Pid, std::ffi::CString};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    ForkFailed(nix::Error),
    AlreadyStarted,
}
type SResult<T> = Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    stack_size: usize,
    hostname: String,
    target_process: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stack_size: 16 * 1024, // 16kb
            hostname: "container".into(),
            target_process: "/bin/sh".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Container {
    is_started: bool,
    stack_memory: Vec<u8>,
    config: Config,
    container_pid: Pid,
}

#[derive(Debug, Clone)]
struct InternalData {
    config: Box<Config>,
}

#[allow(unused_must_use)]
fn container_entry_handle(cfg: InternalData) -> isize {
    let config = &cfg.config;
    nix::unistd::sethostname(config.hostname.clone());

    let cstyle_target = CString::new(config.target_process.to_string()).unwrap();
    nix::unistd::execv(&cstyle_target, &[cstyle_target.as_c_str()]);

    0
}

impl Container {
    pub fn new() -> Self {
        Self {
            is_started: false,
            stack_memory: Vec::new(),
            config: Default::default(),
            container_pid: Pid::from_raw(-1),
        }
    }

    pub fn start(&mut self) -> SResult<()> {
        if self.is_started {
            return Err(Error::AlreadyStarted);
        }

        self.is_started = true;
        self.stack_memory.resize(self.config.stack_size, 0);

        let ic = InternalData {
            config: box self.config.clone(),
        };

        use nix::sched::CloneFlags;
        self.container_pid = match nix::sched::clone(
            box || container_entry_handle(ic.clone()),
            self.stack_memory.as_mut(),
            CloneFlags::CLONE_NEWUTS | CloneFlags::CLONE_NEWIPC,
            Some(nix::sys::signal::SIGCHLD.into()),
        ) {
            Ok(x) => x,
            Err(e) => return Err(Error::ForkFailed(e)),
        };

        Ok(())
    }

    #[allow(unused_must_use)]
    pub fn wait(&self) {
        if self.is_started {
            nix::sys::wait::waitpid(self.container_pid, None);
        }
    }
}
