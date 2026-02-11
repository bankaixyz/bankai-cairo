use crate::app::now_ms;
use std::path::{Path, PathBuf};

pub(crate) async fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    tokio::fs::create_dir_all(parent).await?;

    let tmp = tmp_path(parent, path.file_name().unwrap_or_default().to_string_lossy());
    tokio::fs::write(&tmp, bytes).await?;
    tokio::fs::rename(&tmp, path).await?;
    Ok(())
}

pub(crate) async fn copy_atomic(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    let parent = dst.parent().unwrap_or_else(|| Path::new("."));
    tokio::fs::create_dir_all(parent).await?;

    let tmp = tmp_path(parent, dst.file_name().unwrap_or_default().to_string_lossy());
    tokio::fs::copy(src, &tmp).await?;
    tokio::fs::rename(&tmp, dst).await?;
    Ok(())
}

fn tmp_path(parent: &Path, base: impl AsRef<str>) -> PathBuf {
    parent.join(format!(".{}.tmp.{}", base.as_ref(), now_ms()))
}
