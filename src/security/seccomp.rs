use {super::ApplySecurityPolicy, crate::VoidResult};

#[derive(Debug, Clone)]
pub struct SeccompPolicy {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

fn common_apply(
    target: &Vec<String>,
    default_action: libscmp::Action,
    matched_action: libscmp::Action,
) -> VoidResult {
    if target.len() == 0 {
        return Ok(());
    }

    let mut filter = libscmp::Filter::new(default_action)?;
    for call_name in target.iter() {
        let call_id = libscmp::resolve_syscall_name(call_name);
        match call_id {
            Some(call_id) if call_id >= 0 => {
                filter.add_rule_exact(matched_action, call_id, &[])?;
            }
            _ => {}
        };
    }

    filter.load()?;
    Ok(())
}

impl SeccompPolicy {
    pub fn new() -> Self {
        Self {
            allow: Vec::new(),
            deny: Vec::new(),
        }
    }

    fn apply_as_whitelist(&self) -> VoidResult {
        use libscmp::Action;
        common_apply(
            &self.allow,
            Action::Errno(nix::errno::Errno::EACCES as i32),
            Action::Allow,
        )
    }

    fn apply_as_blacklist(&self) -> VoidResult {
        use libscmp::Action;
        common_apply(
            &self.deny,
            Action::Allow,
            Action::Errno(nix::errno::Errno::EACCES as i32),
        )
    }
}

impl Default for SeccompPolicy {
    fn default() -> Self {
        Self {
            deny: vec![
                "add_key".to_string(),
                "bpf".to_string(),
                "get_kernel_syms".to_string(),
                "keyctl".to_string(),
                "lookup_dcookie".to_string(),
                "mount".to_string(),
                "move_pages".to_string(),
                "nfsservctl".to_string(),
                "open_by_handle_at".to_string(),
                "perf_event_open".to_string(),
                "personality".to_string(),
                "pivot_root".to_string(),
                "swapon".to_string(),
                "swapoff".to_string(),
                "query_module".to_string(),
                "request_key".to_string(),
                "sysfs".to_string(),
                "unshare".to_string(),
                "umount".to_string(),
                "umount2".to_string(),
                "_sysctl".to_string(),
                "uselib".to_string(),
                "userfaultfd".to_string(),
                "vm86".to_string(),
                "vm86old".to_string(),
            ],
            allow: Vec::new(),
        }
    }
}

impl ApplySecurityPolicy for SeccompPolicy {
    fn apply(&self) -> VoidResult {
        self.apply_as_whitelist()?;
        self.apply_as_blacklist()
    }
}
