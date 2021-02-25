use {crate::VoidResult, caps::CapsHashSet};

#[derive(Debug, Clone)]
pub struct CapabilityPolicy {
    pub allow: CapsHashSet,
    pub deny: CapsHashSet,
}

impl CapabilityPolicy {
    pub fn new() -> Self {
        Self {
            allow: caps::read(None, caps::CapSet::Effective).unwrap_or_default(),
            deny: CapsHashSet::new(),
        }
    }

    pub fn get(&self) -> CapsHashSet {
        &self.allow - &self.deny
    }
}

impl std::default::Default for CapabilityPolicy {
    fn default() -> Self {
        use caps::Capability;

        Self {
            allow: vec![
                Capability::CAP_CHOWN,
                Capability::CAP_DAC_OVERRIDE,
                Capability::CAP_FSETID,
                Capability::CAP_FOWNER,
                Capability::CAP_MKNOD,
                Capability::CAP_NET_RAW,
                Capability::CAP_SETGID,
                Capability::CAP_SETUID,
                Capability::CAP_SETFCAP,
                Capability::CAP_SETPCAP,
                Capability::CAP_SYS_CHROOT,
                Capability::CAP_KILL,
                Capability::CAP_AUDIT_WRITE,
            ]
            .into_iter()
            .collect(),
            deny: CapsHashSet::new(),
        }
    }
}

pub trait ApplySecurityPolicy: std::fmt::Debug {
    fn apply(&self) -> VoidResult;
}

impl ApplySecurityPolicy for CapabilityPolicy {
    fn apply(&self) -> VoidResult {        
        let allowed = self.get();
        let mut ok_caps = CapsHashSet::new();
        for item in allowed.iter() {
            if caps::has_cap(None, caps::CapSet::Permitted, item.clone())? {
                ok_caps.insert(item.clone());
            }
        }
        
        caps::set(None, caps::CapSet::Inheritable, &ok_caps)?;
        caps::set(None, caps::CapSet::Effective, &ok_caps)?;
        Ok(())
    }
}
