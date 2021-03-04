use {
    crate::{
        filesystem::MountNamespacedFs,
        idmap,
        resource::CGroupLimitPolicy,
        security::{self, ApplySecurityPolicy},
        VoidResult,
    },
    nix::{
        sys::signal,
        unistd::{self, Pid},
    },
    std::sync::Arc,
};

mod entry;
mod error;

#[derive(Debug)]
pub struct Config {
    pub uid: u64, // unique ID
    pub working_path: String,
    pub stack_size: usize,
    pub hostname: String,
    pub target_executable: String,
    pub fs: Vec<Box<dyn MountNamespacedFs>>,
    pub security_policies: Vec<Box<dyn ApplySecurityPolicy>>,
    pub cgroup_limits: Box<CGroupLimitPolicy>,
    pub inner_uid: u32, // uid inside container
    pub inner_gid: u32, // gid inside container
}

impl Default for Config {
    fn default() -> Self {
        Self {
            uid: rand::random(),
            working_path: "/tmp/ssandbox-rs.workspace/".to_string(),
            stack_size: 2 * 1024 * 1024, // 2048kb
            hostname: "container".to_string(),
            target_executable: "/bin/sh".into(),
            fs: Vec::new(),
            security_policies: vec![
                box (Default::default(): security::CapabilityPolicy),
                box (Default::default(): security::SeccompPolicy),
            ],
            cgroup_limits: Default::default(),
            inner_gid: 0,
            inner_uid: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Container {
    stack_memory: Vec<u8>,
    config: Arc<Config>,
    container_pid: Option<Pid>,
    already_ended: bool,
}

impl std::convert::From<Config> for Container {
    fn from(source: Config) -> Self {
        Self {
            stack_memory: Vec::new(),
            config: Arc::new(source),
            container_pid: None,
            already_ended: false,
        }
    }
}

impl Container {
    pub fn new() -> Self {
        Self {
            stack_memory: Vec::new(),
            config: Arc::new(Default::default()),
            container_pid: None,
            already_ended: false,
        }
    }

    pub fn has_started(&self) -> bool {
        self.container_pid != None
    }

    pub fn has_ened(&self) -> bool {
        self.already_ended
    }

    pub fn start(&mut self) -> VoidResult {
        if self.has_started() {
            return Err(box error::Error::AlreadyStarted);
        }

        self.stack_memory.resize(self.config.stack_size, 0);

        let (ready_pipe_read, ready_pipe_write) = nix::unistd::pipe()?;
        let (report_pipe_read, report_pipe_write) = nix::unistd::pipe()?;

        let ic = entry::InternalData {
            config: self.config.clone(),
            ready_pipe_set: (ready_pipe_read, ready_pipe_write),
            report_pipe_set: (report_pipe_read, report_pipe_write),
        };

        use nix::sched::CloneFlags;
        let pid = match nix::sched::clone(
            box || entry::main(ic.clone()),
            self.stack_memory.as_mut(),
            CloneFlags::CLONE_NEWUTS
                | CloneFlags::CLONE_NEWIPC
                | CloneFlags::CLONE_NEWPID
                | CloneFlags::CLONE_NEWNS
                | CloneFlags::CLONE_NEWUSER,
            Some(signal::SIGCHLD as i32),
        ) {
            Ok(x) => x,
            Err(e) => return Err(box error::Error::ForkFailed(e)),
        };
        self.container_pid = Some(pid);

        unistd::close(ready_pipe_read)?;
        unistd::close(report_pipe_write)?;

        match (|| -> VoidResult {
            idmap::map_to_root(pid)?;
            self.config.cgroup_limits.apply(self.config.uid, pid)?;
            Ok(())
        })() {
            Err(x) => {
                signal::kill(pid, signal::SIGKILL)?;
                return Err(x);
            }
            _ => {}
        }

        // ready, let's tell child:
        // The closing of ready_pipe ends the block of read() at childs entry.
        // So that the real command can be executed via execvp().
        unistd::close(ready_pipe_write)?;

        // our child maybe now complaining about errors
        let mut child_status_buf: [u8; 1] = [255];
        unistd::read(report_pipe_read, &mut child_status_buf)?;
        if child_status_buf[0] != 0 {
            let code = child_status_buf[0];
            let mut addtional_info_buf = Vec::new();
            unistd::read(ready_pipe_read, &mut addtional_info_buf)?;

            let wrapped_error: error::Error =
                error::EntryError::new(code, &addtional_info_buf).into();
            return Err(box wrapped_error);
        }

        Ok(())
    }

    pub fn wait(&mut self) -> VoidResult {
        if !self.has_ened() {
            if let Some(pid) = self.container_pid {
                nix::sys::wait::waitpid(pid, None)?;
            }
            self.already_ended = true;
        }
        Ok(())
    }

    pub fn terminate(&mut self) -> VoidResult {
        if !self.has_ened() {
            if let Some(pid) = self.container_pid {
                signal::kill(pid, signal::SIGKILL)?;
            }
            self.wait()?;
        }
        Ok(())
    }

    pub fn delete(&mut self) -> VoidResult {
        let uid = self.config.uid;
        self.terminate()?;
        self.config.cgroup_limits.delete(uid)?;
        std::fs::remove_dir_all(
            std::path::PathBuf::from(&self.config.working_path).join(format!("{}", uid)),
        )?;
        Ok(())
    }

    pub fn freeze(&self) -> VoidResult {
        self.config.cgroup_limits.freeze(self.config.uid)
    }

    pub fn thaw(&self) -> VoidResult {
        self.config.cgroup_limits.thaw(self.config.uid)
    }
}

impl Drop for Container {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        if self.has_started() {
            self.delete();
        }
    }
}
