use std::{
    fs::{create_dir_all, write},
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};

use uuid::Uuid;

pub enum AssetKind {
    Avatar,
    Background,
}

pub fn save_persona_asset(
    assets_dir: &Path,
    persona_id: Uuid,
    asset_type: AssetKind,
    mime_type: &str,
    raw_bytes: &[u8],
) -> Result<PathBuf> {
    let file_extension = match mime_type {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        _ => return Err(Error::new(ErrorKind::InvalidInput, "unsupported mime type")),
    };

    let dir = assets_dir.join(persona_id.to_string());

    create_dir_all(&dir)?;

    let filename_stem = match asset_type {
        AssetKind::Avatar => "avatar",
        AssetKind::Background => "background",
    };

    let filename = format!("{filename_stem}.{file_extension}");
    let full_path = dir.join(filename);
    write(&full_path, raw_bytes)?;
    Ok(full_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    #[test]
    fn save_asset_writes_file_to_disk() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let persona_id = Uuid::now_v7();

        let saved_path = save_persona_asset(
            temp_dir.path(),
            persona_id,
            AssetKind::Avatar,
            "image/png",
            b"image bytes",
        )
        .expect("save asset");

        assert!(saved_path.exists());
    }
}
