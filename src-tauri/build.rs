use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

/// Pinned JPEXS / FFDec portable release (contains `ffdec.jar`, `lib/`, `flashlib/`, etc.).
const RELEASE_TAG: &str = "version26.0.0";
const ZIP_NAME: &str = "ffdec_26.0.0.zip";
/// SHA-256 of the release asset `ffdec_26.0.0.zip` (GitHub `digest` field).
const EXPECTED_ZIP_SHA256_HEX: &str =
    "e13509d0ed11c6d77bb1588701d803eb2c706e2ba4eaab8bc4477255cfc718f4";

fn zip_download_url() -> String {
    format!(
        "https://github.com/jindrapetrik/jpexs-decompiler/releases/download/{}/{}",
        RELEASE_TAG, ZIP_NAME
    )
}

fn ffdec_help_message(out_dir: &Path) -> String {
    format!(
        "JPEXS FFDec ({ZIP_NAME}, tag {RELEASE_TAG}) is required to build StarDelta.\n\
         Expected SHA-256: {EXPECTED_ZIP_SHA256_HEX}\n\
         Download URL: {}\n\
         \n\
         Options:\n\
         - Ensure network access and re-run `cargo build` (downloads automatically).\n\
         - Set STARDELTA_FFDEC_ZIP to a local copy of the same zip file (checksum must match).\n\
         - Extract the zip manually so this file exists: {}\n",
        zip_download_url(),
        out_dir.join("ffdec.jar").display()
    )
}

fn sha256_hex(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

fn verify_zip_digest(bytes: &[u8], out_dir: &Path) {
    let got = sha256_hex(bytes);
    if got != EXPECTED_ZIP_SHA256_HEX {
        panic!(
            "FFDec zip SHA-256 mismatch.\n\
             Expected: {EXPECTED_ZIP_SHA256_HEX}\n\
             Got:      {got}\n\
             Do not use an unverified archive. Update the pinned hash in src-tauri/build.rs when bumping the release.\n\
             {}",
            ffdec_help_message(out_dir)
        );
    }
}

/// If every non-directory entry shares a single top-level path component, return `Some("that/")` to strip.
fn shared_top_level_prefix(names: &[String]) -> Option<String> {
    let mut first: Option<&str> = None;
    for name in names {
        if name.ends_with('/') {
            continue;
        }
        let head = name.split('/').next().filter(|s| !s.is_empty())?;
        match first {
            None => first = Some(head),
            Some(f) if f == head => {}
            Some(_) => return None,
        }
    }
    let f = first?;
    let all_under = names.iter().all(|n| n == f || n.starts_with(&format!("{f}/")));
    if all_under && names.iter().any(|n| n.starts_with(&format!("{f}/"))) {
        Some(format!("{f}/"))
    } else {
        None
    }
}

fn extract_zip(bytes: &[u8], out_dir: &Path) {
    fs::create_dir_all(out_dir).unwrap_or_else(|e| {
        panic!(
            "Failed to create {}: {e}\n{}",
            out_dir.display(),
            ffdec_help_message(out_dir)
        )
    });

    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).unwrap_or_else(|e| {
        panic!(
            "Invalid FFDec zip archive: {e}\n{}",
            ffdec_help_message(out_dir)
        )
    });

    let mut names: Vec<String> = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap_or_else(|e| {
            panic!(
                "Failed to read zip entry {i}: {e}\n{}",
                ffdec_help_message(out_dir)
            )
        });
        names.push(file.name().to_string());
    }

    let strip_prefix = shared_top_level_prefix(&names);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let raw_name = file.name().to_string();
        let rel = strip_prefix
            .as_ref()
            .map_or(raw_name.as_str(), |p| raw_name.strip_prefix(p).unwrap_or(&raw_name));

        if rel.is_empty() {
            continue;
        }
        if rel.contains("..") {
            panic!(
                "FFDec zip contains an unsafe path: {rel:?}\n{}",
                ffdec_help_message(out_dir)
            );
        }

        let outpath = out_dir.join(rel);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).unwrap_or_else(|e| {
                panic!(
                    "Failed to create directory {}: {e}\n{}",
                    outpath.display(),
                    ffdec_help_message(out_dir)
                )
            });
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).unwrap_or_else(|e| {
                    panic!(
                        "Failed to create directory {}: {e}\n{}",
                        parent.display(),
                        ffdec_help_message(out_dir)
                    )
                });
            }
            let mut outfile = fs::File::create(&outpath).unwrap_or_else(|e| {
                panic!(
                    "Failed to create {}: {e}\n{}",
                    outpath.display(),
                    ffdec_help_message(out_dir)
                )
            });
            std::io::copy(&mut file, &mut outfile).unwrap_or_else(|e| {
                panic!(
                    "Failed to write {}: {e}\n{}",
                    outpath.display(),
                    ffdec_help_message(out_dir)
                )
            });
        }
    }
}

fn read_zip_bytes() -> Vec<u8> {
    if let Ok(p) = env::var("STARDELTA_FFDEC_ZIP") {
        let path = PathBuf::from(&p);
        fs::read(&path).unwrap_or_else(|e| {
            panic!(
                "STARDELTA_FFDEC_ZIP is set to {} but could not be read: {e}\n{}",
                path.display(),
                ffdec_help_message(&PathBuf::from("src-tauri/resources/jpexs"))
            )
        })
    } else {
        let url = zip_download_url();
        let mut reader = ureq::get(&url)
            .call()
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to download FFDec from {url}: {e}\n{}",
                    ffdec_help_message(&PathBuf::from("src-tauri/resources/jpexs"))
                )
            })
            .into_reader();
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).unwrap_or_else(|e| {
            panic!(
                "Failed to read FFDec download from {url}: {e}\n{}",
                ffdec_help_message(&PathBuf::from("src-tauri/resources/jpexs"))
            )
        });
        buf
    }
}

fn ensure_ffdec_resources(manifest_dir: &Path) {
    println!("cargo:rerun-if-env-changed=STARDELTA_FFDEC_ZIP");

    let out_dir = manifest_dir.join("resources/jpexs");
    let marker = out_dir.join("ffdec.jar");

    if marker.is_file() && fs::metadata(&marker).map(|m| m.len() > 0).unwrap_or(false) {
        return;
    }

    let bytes = read_zip_bytes();
    verify_zip_digest(&bytes, &out_dir);

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).unwrap_or_else(|e| {
            panic!(
                "Could not clear {} before extracting FFDec: {e}\n{}",
                out_dir.display(),
                ffdec_help_message(&out_dir)
            )
        });
    }

    extract_zip(&bytes, &out_dir);

    if !marker.is_file() {
        panic!(
            "After extracting FFDec, ffdec.jar was not found at {}.\n{}",
            marker.display(),
            ffdec_help_message(&out_dir)
        );
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    ensure_ffdec_resources(&manifest_dir);
    tauri_build::build()
}
