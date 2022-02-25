//! Build script that bundles Riposte into a tarball.
//!
//! Responsibilities:
//! * Locate the client and server executables
//! * Locate all dynamic libraries
//! * Locate all assets
//! and bundle everything into a tarball.

use serde::Deserialize;
use std::{
    fs::{self, File},
    io::Cursor,
    path::Path,
};
use tar::Header;

const BUILD_DIR: &str = "target/release";

const UI_DIR: &str = "crates/client/ui";
const STYLE_PATH: &str = "crates/client/style.yml";

#[derive(Debug)]
struct Asset {
    path: String,
    contents: Vec<u8>,
}

#[derive(Debug)]
struct Executable {
    name: String,
    contents: Vec<u8>,
}

#[derive(Debug, Default)]
struct Bundle {
    assets: Vec<Asset>,
    executables: Vec<Executable>,
}

impl Bundle {
    pub fn byte_size(&self) -> usize {
        self.executables
            .iter()
            .map(|e| e.contents.len())
            .chain(self.assets.iter().map(|m| m.contents.len()))
            .sum::<usize>()
    }
}

fn main() -> anyhow::Result<()> {
    let bundle = find_bundle()?;

    println!(
        "Bundle contains {} assets and {} executables.",
        bundle.assets.len(),
        bundle.executables.len(),
    );
    println!(
        "Total uncompressed size: {} MiB",
        bundle.byte_size() / 1024 / 1024
    );

    println!("Bundling into a tarball...");

    let tarball_data = build_tarball(&bundle)?;
    fs::write("target/release/riposte.tar.zst", &tarball_data)?;
    println!(
        "Success. Final bundle size: {} MiB",
        tarball_data.len() / 1024 / 1024
    );

    Ok(())
}

fn find_bundle() -> anyhow::Result<Bundle> {
    let mut bundle = Bundle::default();
    find_assets(&mut bundle)?;
    find_executables(&mut bundle)?;
    Ok(bundle)
}

#[derive(Debug, Deserialize)]
struct AssetEntry {
    #[allow(unused)]
    id: String,
    path: String,
}

fn find_assets(bundle: &mut Bundle) -> anyhow::Result<()> {
    let index: Vec<AssetEntry> = serde_json::from_reader(File::open("assets/index.json")?)?;

    bundle.assets.push(Asset {
        path: "index.json".to_owned(),
        contents: fs::read("assets/index.json")?,
    });

    for entry in index {
        let asset = Asset {
            path: entry.path.clone(),
            contents: fs::read(Path::new("assets").join(&entry.path))?,
        };
        bundle.assets.push(asset);

        println!("Found asset '{}'", entry.path);
    }

    Ok(())
}

fn find_executables(bundle: &mut Bundle) -> anyhow::Result<()> {
    for (file, name) in [("riposte", "Riposte")] {
        bundle.executables.push(Executable {
            name: name.to_owned(),
            contents: fs::read(Path::new(BUILD_DIR).join(file))?,
        });
    }
    Ok(())
}

const COMPRESSION_LEVEL: i32 = 14;

fn build_tarball(bundle: &Bundle) -> anyhow::Result<Vec<u8>> {
    let mut encoder = zstd::Encoder::new(Vec::<u8>::new(), COMPRESSION_LEVEL)?;
    encoder.multithread(4)?;
    let mut archive = tar::Builder::new(encoder);

    for asset in &bundle.assets {
        let mut header = Header::new_gnu();
        header.set_path(format!("assets/{}", asset.path))?;
        header.set_size(asset.contents.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append(&header, Cursor::new(&asset.contents))?;
    }

    // UI YAML specs and stylesheets
    archive.append_dir_all("ui", UI_DIR)?;
    archive.append_file("style.yml", &mut File::open(STYLE_PATH)?)?;

    // Executables
    for executable in &bundle.executables {
        let mut header = Header::new_gnu();
        header.set_path(&executable.name)?;
        header.set_size(executable.contents.len() as u64);
        header.set_mode(0o744);
        header.set_cksum();
        archive.append(&header, Cursor::new(&executable.contents))?;
    }

    let inner = archive.into_inner()?;
    Ok(inner.finish()?)
}
