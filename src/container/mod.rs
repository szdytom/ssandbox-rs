use {crate::filesystem::MountNamespacedFs, nix::unistd::Pid, std::sync::Arc};

mod entry;
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
            stack_size: 256 * 1024, // 256kb
            hostname: "container".to_string(),
            target_executable: "/bin/sh".into(),
            fs: Vec::new(),
        }
    }
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

        let ic = entry::InternalData {
            config: self.config.clone(),
        };

        use nix::sched::CloneFlags;
        self.container_pid = Some(
            match nix::sched::clone(
                box || entry::main(ic.clone()),
                self.stack_memory.as_mut(),
                CloneFlags::CLONE_NEWUTS
                    | CloneFlags::CLONE_NEWIPC
                    | CloneFlags::CLONE_NEWPID
                    | CloneFlags::CLONE_NEWNS,
                Some(nix::sys::signal::SIGCHLD as i32),
            ) {
                Ok(x) => x,
                Err(e) => return Err(box error::Error::ForkFailed(e)),
            },
        );

        Ok(())
    }

    pub fn wait(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.has_started() {
            nix::sys::wait::waitpid(self.container_pid, None)?;
        }
        Ok(())
    }
}
