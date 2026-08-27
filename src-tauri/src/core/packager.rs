use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use chrono::Local;
use serde_json::json;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::core::constants::get_pack_formats;

pub struct PackWriter {
    pub rp_zip_path: PathBuf,
    pub dp_zip_path: PathBuf,
    rp_writer: Option<ZipWriter<File>>,
    dp_writer: Option<ZipWriter<File>>,
    written: HashSet<String>,
}

impl PackWriter {
    pub fn new(
        mc_dir: &Path,
        pack_base_name: &str,
        mc_version: &str,
        lang_name: &str,
    ) -> Result<Self, String> {
        let pack_formats = get_pack_formats();
        let default_fmt = pack_formats.get("1.21.1").unwrap();
        let fmt = pack_formats.get(mc_version).unwrap_or(default_fmt);

        let rp_dir = mc_dir.join("resourcepacks");
        let dp_dir = mc_dir.join("config").join("openloader").join("data");

        let _ = fs::create_dir_all(&rp_dir);
        let _ = fs::create_dir_all(&dp_dir);

        let mut safe_name = pack_base_name
            .trim()
            .replace(['\\', '/', '*', '?', ':', '"', '<', '>', '|'], "");
        if safe_name.is_empty() {
            safe_name = "ModLang_Pack".to_string();
        }
        if !safe_name.to_lowercase().ends_with(".zip") {
            safe_name.push_str(".zip");
        }

        let rp_zip_path = rp_dir.join(&safe_name);
        let dp_name = safe_name.replace(".zip", "_Datapack.zip");
        let dp_zip_path = dp_dir.join(&dp_name);

        // Initialize RP Zip
        let rp_file = File::create(&rp_zip_path).map_err(|e| e.to_string())?;
        let mut rp_writer = ZipWriter::new(rp_file);
        let rp_meta = json!({
            "pack": {
                "pack_format": fmt.rp,
                "description": format!("ModLang Localization [{}]", lang_name)
            }
        });
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        rp_writer.start_file("pack.mcmeta", options).map_err(|e| e.to_string())?;
        rp_writer.write_all(serde_json::to_string_pretty(&rp_meta).unwrap().as_bytes())
            .map_err(|e| e.to_string())?;

        // Initialize DP Zip
        let dp_file = File::create(&dp_zip_path).map_err(|e| e.to_string())?;
        let mut dp_writer = ZipWriter::new(dp_file);
        let dp_meta = json!({
            "pack": {
                "pack_format": fmt.dp,
                "description": format!("ModLang Datapack [{}]", lang_name)
            }
        });
        dp_writer.start_file("pack.mcmeta", options).map_err(|e| e.to_string())?;
        dp_writer.write_all(serde_json::to_string_pretty(&dp_meta).unwrap().as_bytes())
            .map_err(|e| e.to_string())?;

        Ok(Self {
            rp_zip_path,
            dp_zip_path,
            rp_writer: Some(rp_writer),
            dp_writer: Some(dp_writer),
            written: HashSet::new(),
        })
    }

    fn unique_path(dir: &Path, filename: &str) -> PathBuf {
        let candidate = dir.join(filename);
        if !candidate.exists() {
            return candidate;
        }
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("pack");
        let ext = Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("zip");
        let mut counter = 1;
        loop {
            let next = dir.join(format!("{}_{}.{}", stem, counter, ext));
            if !next.exists() {
                return next;
            }
            counter += 1;
        }
    }

    pub fn write_entry(&mut self, internal_path: &str, data: &[u8]) -> Result<(), String> {
        let normalized = internal_path.replace('\\', "/").trim_start_matches('/').to_string();
        if self.written.contains(&normalized) {
            return Ok(());
        }

        let is_data = normalized.to_lowercase().starts_with("data/");
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        if is_data {
            if let Some(dp) = self.dp_writer.as_mut() {
                dp.start_file(&normalized, options).map_err(|e| e.to_string())?;
                dp.write_all(data).map_err(|e| e.to_string())?;
                self.written.insert(normalized);
            }
        } else if let Some(rp) = self.rp_writer.as_mut() {
            rp.start_file(&normalized, options).map_err(|e| e.to_string())?;
            rp.write_all(data).map_err(|e| e.to_string())?;
            self.written.insert(normalized);
        }

        Ok(())
    }

    pub fn close(&mut self) -> Result<(), String> {
        if let Some(mut rp) = self.rp_writer.take() {
            rp.finish().map_err(|e| e.to_string())?;
        }
        if let Some(mut dp) = self.dp_writer.take() {
            dp.finish().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

impl Drop for PackWriter {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

pub fn create_backup(mc_dir: &Path) -> Result<PathBuf, String> {
    let backup_dir = mc_dir.join("backups_localization");
    let _ = fs::create_dir_all(&backup_dir);

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let backup_zip_path = backup_dir.join(format!("backup_{}.zip", timestamp));

    let file = File::create(&backup_zip_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let folders_to_backup = ["config", "kubejs", "defaultconfigs"];
    for folder in &folders_to_backup {
        let source_path = mc_dir.join(folder);
        if !source_path.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&source_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Ok(rel_path) = path.strip_prefix(mc_dir) {
                    let internal_name = rel_path.to_string_lossy().replace('\\', "/");
                    if zip.start_file(&internal_name, options).is_ok() {
                        if let Ok(mut f) = File::open(path) {
                            let mut buffer = Vec::new();
                            if f.read_to_end(&mut buffer).is_ok() {
                                let _ = zip.write_all(&buffer);
                            }
                        }
                    }
                }
            }
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(backup_zip_path)
}
