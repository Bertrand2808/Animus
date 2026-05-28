use std::{
    fs::{create_dir_all, write},
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose, Engine as _};
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

pub fn parse_data_uri(uri: &str) -> Option<(String, Vec<u8>)> {
    if !uri.starts_with("data:") {
        return None;
    }

    let (header, data) = uri.split_once(";base64,")?;

    let mime_type = header.strip_prefix("data:")?;

    let bytes: Vec<u8> = general_purpose::STANDARD.decode(data).ok()?;
    Some((mime_type.to_string(), bytes))
}

pub fn delete_persona_assets(assets_dir: &Path, persona_id: Uuid) -> Result<()> {
    let dir = assets_dir.join(persona_id.to_string());
    match std::fs::remove_dir_all(&dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
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

    #[test]
    fn parse_data_uri_decodes_base64_bytes() {
        let (mime_type, bytes) =
            parse_data_uri("data:image/png;base64,aGVsbG8gd29ybGQ=").expect("parse data URI");

        assert_eq!(mime_type, "image/png");
        assert_eq!(bytes, b"hello world");
    }

    #[test]
    fn delete_persona_assets_removes_directory() {
        let temp_dir = TempDir::new().expect("create temp dir");
        let persona_id = Uuid::now_v7();

        save_persona_asset(temp_dir.path(), persona_id, AssetKind::Avatar, "image/png", b"bytes")
            .unwrap();
        assert!(temp_dir.path().join(persona_id.to_string()).exists());

        delete_persona_assets(temp_dir.path(), persona_id).unwrap();
        assert!(!temp_dir.path().join(persona_id.to_string()).exists());
    }

    #[test]
    fn delete_persona_assets_ok_when_not_found() {
        let temp_dir = TempDir::new().expect("create temp dir");
        delete_persona_assets(temp_dir.path(), Uuid::now_v7()).unwrap();
    }
}
