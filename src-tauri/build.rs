use std::env;
use std::fs;
use std::path::Path;

// Embed Windows manifest to resolve STATUS_ENTRYPOINT_NOT_FOUND
#[cfg(windows)]
use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    // Embed Windows manifest to resolve STATUS_ENTRYPOINT_NOT_FOUND
    #[cfg(windows)]
    {
        let manifest = new_manifest("re-strike-vta");
        embed_manifest(manifest).expect("Failed to embed manifest");
    }

    println!("cargo:rerun-if-changed=../ui/public/scoreboard-overlay.html");
    println!("cargo:rerun-if-changed=../ui/public/player-introduction-overlay.html");
    println!("cargo:rerun-if-changed=../ui/public/test-scoreboard-fixes.html");
    println!("cargo:rerun-if-changed=../ui/public/assets/");

    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // Create assets directory in the output directory
    let assets_dir = Path::new(&out_dir).join("assets");
    fs::create_dir_all(&assets_dir).unwrap();
    
    // Copy HTML overlay files
    let html_files = [
        "../ui/public/scoreboard-overlay.html",
        "../ui/public/player-introduction-overlay.html", 
        "../ui/public/test-scoreboard-fixes.html"
    ];
    
    for html_file in &html_files {
        let src_path = Path::new(&manifest_dir).join(html_file);
        let dst_path = assets_dir.join(Path::new(html_file).file_name().unwrap());
        
        if src_path.exists() {
            fs::copy(&src_path, &dst_path).unwrap();
            println!("cargo:warning=Copied {} to {}", src_path.display(), dst_path.display());
        } else {
            println!("cargo:warning=Source file not found: {}", src_path.display());
        }
    }
    
    // Copy assets directory recursively
    let src_assets = Path::new(&manifest_dir).join("../ui/public/assets");
    let dst_assets = assets_dir.join("assets");
    
    if src_assets.exists() {
        copy_dir_all(&src_assets, &dst_assets).unwrap();
        println!("cargo:warning=Copied assets directory to {}", dst_assets.display());
    } else {
        println!("cargo:warning=Source assets directory not found: {}", src_assets.display());
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        fs::create_dir(dst)?;
    }
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
} 