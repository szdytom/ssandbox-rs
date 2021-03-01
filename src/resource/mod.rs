use {
    crate::VoidResult,
    cgroups_rs::{
        cpu::CpuController, freezer::FreezerController, memory::MemController, pid::PidController,
        MaxValue,
    },
};

#[derive(Debug, Clone, Default)]
pub struct CGroupLimitPolicy {
    cpu_limit: Option<i64>,
    time_limit: Option<u64>,
    memory_limit: Option<i64>,
    fork_limit: Option<u32>,
}

impl CGroupLimitPolicy {
    pub fn set_time_limit(&mut self, value: u64) -> &mut Self {
        self.time_limit = Some(value);
        self
    }

    pub fn set_memory_limit(&mut self, value: i64) -> &mut Self {
        self.memory_limit = Some(value);
        self
    }

    pub fn set_fork_limit(&mut self, value: u32) -> &mut Self {
        self.fork_limit = Some(value);
        self
    }

    pub fn set_cpu_limit(&mut self, value: i64) -> &mut Self {
        self.cpu_limit = Some(value);
        self
    }

    pub fn clear_time_limit(&mut self) -> &mut Self {
        self.time_limit = None;
        self
    }

    pub fn clear_memory_limit(&mut self) -> &mut Self {
        self.time_limit = None;
        self
    }

    pub fn clear_fork_limit(&mut self) -> &mut Self {
        self.time_limit = None;
        self
    }

    pub fn clear_cpu_limit(&mut self) -> &mut Self {
        self.time_limit = None;
        self
    }

    pub fn apply(&self, uid: u64, pid: nix::unistd::Pid) -> VoidResult {
        let hier = cgroups_rs::hierarchies::auto();
        let cg = cgroups_rs::cgroup::Cgroup::new(hier, &format!("ssandbox.rs.container.{}", uid));
        cg.add_task(cgroups_rs::CgroupPid::from(pid.as_raw() as u64))?;

        if let Some(fork_limit) = self.fork_limit {
            let control: Option<&PidController> = cg.controller_of();
            if let Some(control) = control {
                control.set_pid_max(MaxValue::Value(fork_limit.into()))?;
            }
        }

        if let Some(cpu_limit) = self.cpu_limit {
            let control: Option<&CpuController> = cg.controller_of();
            if let Some(control) = control {
                control.set_cfs_period(50000)?;
                control.set_cfs_quota(cpu_limit * 50000 / 1000000)?;
            }
        }

        if let Some(memory_limit) = self.memory_limit {
            let control: Option<&MemController> = cg.controller_of();
            if let Some(control) = control {
                control.set_kmem_limit(memory_limit)?;
                control.set_limit(memory_limit)?;
                control.set_memswap_limit(memory_limit)?;
            }
        }

        Ok(())
    }

    pub fn freeze(&self, uid: u64) -> VoidResult {
        let hier = cgroups_rs::hierarchies::auto();
        let cg = cgroups_rs::cgroup::Cgroup::load(hier, &format!("ssandbox.rs.container.{}", uid));
        let control: Option<&FreezerController> = cg.controller_of();
        if let Some(freezer) = control {
            freezer.freeze()?;
        }
        Ok(())
    }

    pub fn thaw(&self, uid: u64) -> VoidResult {
        let hier = cgroups_rs::hierarchies::auto();
        let cg = cgroups_rs::cgroup::Cgroup::load(hier, &format!("ssandbox.rs.container.{}", uid));
        let control: Option<&FreezerController> = cg.controller_of();
        if let Some(freezer) = control {
            freezer.thaw()?;
        }
        Ok(())
    }

    pub fn delete(&self, uid: u64) -> VoidResult {
        let hier = cgroups_rs::hierarchies::auto();
        let cg = cgroups_rs::cgroup::Cgroup::load(hier, &format!("ssandbox.rs.container.{}", uid));
        cg.delete()?;
        Ok(())
    }
}
