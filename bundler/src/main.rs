//! Build script that bundles Riposte into a tarball.
//!
//! Responsibilities:
//! * Locate all used Lua modules (including C shared libraries)
//! * Locate all dynamic libraries
//! * Locate all assets
//! and bundle everything into a tarball.

use std::{
    collections::HashSet,
    fs::{self, File},
    io::Cursor,
    path::Path,
};

use anyhow::{bail, Context};
use regex::Regex;
use serde::Deserialize;
use tar::Header;

const BUILD_DIR: &str = "cmake-build-release";

#[derive(Debug)]
struct LuaModule {
    package_path: String,
    contents: Vec<u8>,
    is_shared_library: bool,
}

#[derive(Debug)]
struct LuaPath {
    path: String,
    cpath: String,
}

impl LuaPath {
    pub fn load_package(&self, path: &str) -> anyhow::Result<LuaModule> {
        for (possible_path, is_shared_library) in self
            .path
            .split(';')
            .map(|p| (p, false))
            .chain(self.cpath.split(';').map(|p| (p, true)))
        {
            let possible_path = possible_path.replace('?', path);

            if Path::new(&possible_path).exists() {
                return Ok(LuaModule {
                    package_path: path.to_owned(),
                    contents: fs::read(&possible_path)?,
                    is_shared_library,
                });
            }
        }

        bail!("could not locate Lua package '{}'", path);
    }
}

impl LuaModule {
    /// Attempts to locate the package.path and package.cpath
    /// inside this module.
    pub fn locate_lua_path(&mut self) -> anyhow::Result<LuaPath> {
        let path_regex = Regex::new("package\\.path = \"(.+)\"").unwrap();
        let cpath_regex = Regex::new("package\\.cpath = \"(.+)\"").unwrap();

        let contents = std::str::from_utf8(&self.contents)?.to_owned();
        let path = path_regex
            .captures(&contents)
            .context("missing package.path statement")?
            .get(1)
            .unwrap()
            .as_str()
            .to_owned();
        let cpath = cpath_regex
            .captures(&contents)
            .context("missing package.cpath statement")?
            .get(1)
            .unwrap()
            .as_str()
            .to_owned();

        // Replace the path and cpath with new values for the bundle.
        self.contents = path_regex
            .replace(&contents, "package.path = \"client/?.lua\"")
            .to_string()
            .into_bytes();
        self.contents = cpath_regex
            .replace(
                std::str::from_utf8(&self.contents)?,
                "package.cpath = \"client/?.so\"",
            )
            .to_string()
            .into_bytes();

        Ok(LuaPath { path, cpath })
    }
}

#[derive(Debug)]
struct Asset {
    path: String,
    contents: Vec<u8>,
}

#[derive(Debug)]
struct DynLib {
    name: String,
    contents: Vec<u8>,
}

#[derive(Debug, Default)]
struct Bundle {
    lua_modules: Vec<LuaModule>,
    assets: Vec<Asset>,
    dynlibs: Vec<DynLib>,
    executable: Vec<u8>,
}

impl Bundle {
    pub fn byte_size(&self) -> usize {
        self.lua_modules
            .iter()
            .map(|m| m.contents.len())
            .chain(self.assets.iter().map(|m| m.contents.len()))
            .chain(self.dynlibs.iter().map(|d| d.contents.len()))
            .sum::<usize>()
            + self.executable.len()
    }
}

fn main() -> anyhow::Result<()> {
    let bundle = find_bundle()?;

    println!(
        "Bundle contains {} assets, {} Lua modules, and {} dynamic libraries.",
        bundle.assets.len(),
        bundle.lua_modules.len(),
        bundle.dynlibs.len()
    );
    println!(
        "Total uncompressed size: {} MiB",
        bundle.byte_size() / 1024 / 1024
    );

    println!("Bundling into a tarball...");

    let tarball_data = build_tarball(&bundle)?;
    fs::write("cmake-build-release/riposte.tar.zst", &tarball_data)?;
    println!(
        "Success. Final bundle size: {} MiB",
        tarball_data.len() / 1024 / 1024
    );

    Ok(())
}

fn find_bundle() -> anyhow::Result<Bundle> {
    let mut bundle = Bundle::default();
    find_lua_modules(&mut bundle)?;
    find_assets(&mut bundle)?;
    find_dynlibs(&mut bundle)?;
    find_executable(&mut bundle)?;
    Ok(bundle)
}

fn find_lua_modules(bundle: &mut Bundle) -> anyhow::Result<()> {
    let mut main_module = LuaModule {
        package_path: "main".to_owned(),
        contents: fs::read("client/main.lua")?,
        is_shared_library: false,
    };
    let lua_path = main_module.locate_lua_path()?;

    // Recursively search each Lua module for require() statements.
    search_lua_module(&main_module, bundle, &lua_path, &mut HashSet::new())?;
    bundle.lua_modules.push(main_module);

    Ok(())
}

// This function recurses.
fn search_lua_module(
    module: &LuaModule,
    bundle: &mut Bundle,
    lua_path: &LuaPath,
    visited: &mut HashSet<String>,
) -> anyhow::Result<()> {
    if module.is_shared_library {
        return Ok(());
    }
    let require_regex = Regex::new("require\\(\"(.+)\"\\)").unwrap();
    let require_regex2 = Regex::new("require '(.+)'").unwrap();

    let contents = std::str::from_utf8(&module.contents)?;
    for capture in require_regex
        .captures_iter(contents)
        .chain(require_regex2.captures_iter(contents))
    {
        let package_path = capture.get(1).unwrap();
        let package_path = package_path.as_str().replace('.', "/");

        if visited.insert(package_path.clone()) {
            println!("Found Lua module '{}'", package_path);
            let package_module = lua_path.load_package(&package_path)?;
            search_lua_module(&package_module, bundle, lua_path, visited)?;
            bundle.lua_modules.push(package_module);
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct AssetEntry {
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

fn find_dynlibs(bundle: &mut Bundle) -> anyhow::Result<()> {
    let libs_dir = Path::new(BUILD_DIR);
    for entry in fs::read_dir(&libs_dir)? {
        let entry = entry?;
        let path = entry.path();

        let extension = path.extension().map(|s| s.to_str().unwrap());
        if !matches!(extension, Some("so" | "dylib" | "dll")) {
            continue;
        }

        bundle.dynlibs.push(DynLib {
            name: entry.file_name().to_string_lossy().to_string(),
            contents: fs::read(&path)?,
        });

        println!("Found dynamic library '{}'", path.display());
    }

    // Special case for Protocol Buffers file
    bundle.dynlibs.push(DynLib {
        name: "proto/riposte.proto".to_owned(),
        contents: fs::read("proto/riposte.proto")?,
    });

    Ok(())
}

fn find_executable(bundle: &mut Bundle) -> anyhow::Result<()> {
    bundle.executable = fs::read(Path::new(BUILD_DIR).join("bin/riposte"))?;
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

    for lua_module in &bundle.lua_modules {
        let mut header = Header::new_gnu();
        header.set_path(format!(
            "client/{}.{}",
            lua_module.package_path,
            if lua_module.is_shared_library {
                "so"
            } else {
                "lua"
            }
        ))?;
        header.set_size(lua_module.contents.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append(&header, Cursor::new(&lua_module.contents))?;
    }

    for dynlib in &bundle.dynlibs {
        let mut header = Header::new_gnu();
        header.set_path(&dynlib.name)?;
        header.set_size(dynlib.contents.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        archive.append(&header, Cursor::new(&dynlib.contents))?;
    }

    // Executable
    let mut eheader = Header::new_gnu();
    eheader.set_path("Riposte")?;
    eheader.set_size(bundle.executable.len() as u64);
    eheader.set_mode(0o744);
    eheader.set_cksum();
    archive.append(&eheader, Cursor::new(&bundle.executable))?;

    let inner = archive.into_inner()?;
    Ok(inner.finish()?)
}
