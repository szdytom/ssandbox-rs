use {crate::VoidResult, std::fs, nix::unistd};

pub fn set_one_map(file: &String, inside_id: u32, outside_id: u32) -> VoidResult {
    fs::write(file, format!("{} {} 1\n", inside_id, outside_id))?;
    Ok(())
}

pub fn map_uid_to_root(pid: unistd::Pid) -> VoidResult {
    set_one_map(&format!("/proc/{}/uid_map", pid), 0, unistd::geteuid().as_raw())?;
    Ok(())
}

pub fn map_gid_to_root(pid: unistd::Pid) -> VoidResult {
    set_one_map(&format!("/proc/{}/gid_map", pid), 0, unistd::getegid().as_raw())?;
    Ok(())
}

pub fn map_to_root(pid: unistd::Pid) -> VoidResult {
    map_uid_to_root(pid)?;
    map_gid_to_root(pid)?;
    Ok(())
}