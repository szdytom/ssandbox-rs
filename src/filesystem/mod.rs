use nix::mount::{self, MsFlags};

pub trait MountNamespacedFs: std::fmt::Debug {
    fn loading(
        &self,
        _base_path: &std::path::Path,
        _workspace: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
        mount::mount::<_, _, _, str>(Some("tmpfs"), "/tmp", Some("tmpfs"), MsFlags::empty(), None)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MountProcFs;

impl MountNamespacedFs for MountProcFs {
    fn loaded(&self) -> Result<(), Box<dyn std::error::Error>> {
        mount::mount::<_, _, _, str>(Some("proc"), "/proc", Some("proc"), MsFlags::empty(), None)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MountBindFs {
    source: String,
}

impl std::convert::From<String> for MountBindFs {
    fn from(source: String) -> Self {
        Self { source: source }
    }
}

impl MountNamespacedFs for MountBindFs {
    fn loading(
        &self,
        base_path: &std::path::Path,
        _: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        mount::mount::<str, _, str, str>(
            Some(&self.source),
            base_path,
            None,
            MsFlags::MS_REC | MsFlags::MS_BIND,
            None,
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MountReadOnlyBindFs {
    source: String,
}

impl std::convert::From<String> for MountReadOnlyBindFs {
    fn from(source: String) -> Self {
        Self { source: source }
    }
}

impl MountNamespacedFs for MountReadOnlyBindFs {
    fn loading(
        &self,
        base_path: &std::path::Path,
        _: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        mount::mount::<str, _, str, str>(
            Some(&self.source),
            base_path,
            None,
            MsFlags::MS_REC | MsFlags::MS_BIND,
            None,
        )?;

        mount::mount::<str, _, str, str>(
            None,
            base_path,
            None,
            MsFlags::MS_BIND | MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY | MsFlags::MS_REC,
            None,
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MountExtraFs {
    source: Option<String>,
    inner: String,
}

impl std::convert::From<String> for MountExtraFs {
    fn from(source: String) -> Self {
        Self {
            source: Some(source),
            inner: "mnt".to_string(),
        }
    }
}

impl MountExtraFs {
    pub fn build(source: String, inner: String) -> Self {
        let inner = std::path::PathBuf::from(&inner);
        let inner = inner
            .strip_prefix("/")
            .unwrap_or(&inner)
            .to_string_lossy()
            .to_owned()
            .to_string();

        Self {
            source: Some(source),
            inner: inner,
        }
    }

    pub fn new() -> Self {
        Self {
            source: None,
            inner: "mnt".to_string(),
        }
    }
}

impl MountNamespacedFs for MountExtraFs {
    fn loading(
        &self,
        base_path: &std::path::Path,
        work_path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = match &self.source {
            Some(x) => std::path::PathBuf::from(x),
            None => {
                let path = work_path.join("extra");
                if !path.exists() {
                    std::fs::create_dir_all(&path)?;
                }
                path
            }
        };

        mount::mount::<std::path::Path, _, str, str>(
            Some(&source),
            &base_path.join(&self.inner),
            None,
            MsFlags::MS_REC | MsFlags::MS_BIND,
            None,
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct MountSizedTmpFs {
    size_limit: Option<u64>,
    target: String,
}

impl Default for MountSizedTmpFs {
    fn default() -> Self {
        Self {
            size_limit: None,
            target: "/tmp".to_string(),
        }
    }
}

impl From<u64> for MountSizedTmpFs {
    fn from(x: u64) -> Self {
        Self {
            size_limit: Some(x),
            ..Default::default()
        }
    }
}

impl From<String> for MountSizedTmpFs {
    fn from(x: String) -> Self {
        Self {
            target: x,
            ..Default::default()
        }
    }
}

impl MountNamespacedFs for MountSizedTmpFs {
    fn loaded(&self) -> Result<(), Box<dyn std::error::Error>> {
        let data = if let Some(size) = self.size_limit {
            format!("size={}", size)
        } else {
            "".to_string()
        };

        mount::mount::<_, str, _, str>(
            Some("tmpfs"),
            &self.target,
            Some("tmpfs"),
            MsFlags::empty(),
            if data.is_empty() { None } else { Some(&data) },
        )?;
        Ok(())
    }
}
