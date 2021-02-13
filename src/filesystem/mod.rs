use nix::mount::{self, MsFlags};

pub trait MountNamespacedFs: std::fmt::Debug {
    fn loading(&self, _: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    fn loaded(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct MountTmpFs;

impl MountNamespacedFs for MountTmpFs {
    fn loaded(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Mounting Temp FS");
        mount::mount::<str, str, str, str>(
            Some("tmpfs"),
            "/tmp",
            Some("tmpfs"),
            MsFlags::empty(),
            None,
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MountProcFs;

impl MountNamespacedFs for MountProcFs {
    fn loaded(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Mounting Proc FS (proc)");
        mount::mount::<str, str, str, str>(
            Some("proc"),
            "/proc",
            Some("proc"),
            MsFlags::empty(),
            None,
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MountBindFs {
    source: String,
}

impl std::convert::From<String> for MountBindFs {
    fn from(source: String) -> Self {
        Self {
            source: source,
        }
    }
}

impl MountNamespacedFs for MountBindFs {
    fn loading(&self, base_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("Mounting Main FS from {} to {:?}", self.source, base_path);
        mount::mount::<str, std::path::Path, str, str>(
            Some(&self.source),
            base_path,
            None,
            MsFlags::MS_REC | MsFlags::MS_BIND,
            None,
        )?;
        Ok(())
    }
}
