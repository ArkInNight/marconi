use std::path::{Path, PathBuf};

pub fn exe_dir() -> anyhow::Result<PathBuf> {
    let exe = std::env::current_exe()?;
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("cannot resolve executable directory"))?;
    Ok(dir.to_path_buf())
}

pub fn project_runtime_dir(name: &str) -> anyhow::Result<PathBuf> {
    let dir = exe_dir()?.join(name);
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn ensure_project_runtime_dirs() -> anyhow::Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for name in ["cache", "temp", "logs"] {
        dirs.push(project_runtime_dir(name)?);
    }
    Ok(dirs)
}

pub fn remove_dir_if_exists(path: &Path) {
    if let Err(err) = std::fs::remove_dir_all(path) {
        if err.kind() != std::io::ErrorKind::NotFound {
            eprintln!("warning: failed to remove {}", path.display());
        }
    }
}
