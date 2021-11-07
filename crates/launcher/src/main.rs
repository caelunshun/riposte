use std::{
    convert::TryInto,
    fs,
    io::Cursor,
    path::PathBuf,
    process::{self, exit},
};

use anyhow::anyhow;
use directories_next::ProjectDirs;
use duit::{Rect, Spec, Ui, Vec2, WindowPositioner};
use generated::RiposteLauncher;
use octocrab::{models::repos::Asset, Octocrab};
use reqwest::header;
use serde::{Deserialize, Serialize};
use tar::Archive;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

mod generated;

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        const OS_SPECIFIER: &str = "macOS";
        const LD_ENV: &str = "DYLD_LIBRARY_PATH";
    } else if #[cfg(unix)] {
        const OS_SPECIFIER: &str = "linux";
        const LD_ENV: &str = "LD_LIBRARY_PATH";
    } else if #[cfg(windows)] {
        const OS_SPECIFIER: &str = "windows";
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct DownloadedVersion {
    id: u64,
    created_at: String,
}

impl DownloadedVersion {
    pub fn from_asset(asset: &Asset) -> Self {
        Self {
            id: asset.id.into_inner(),
            created_at: asset.created_at.to_rfc2822(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let github = octocrab::OctocrabBuilder::new().build()?;

    let asset = get_latest_tarball_info(&github).await?;

    let current_version = get_downloaded_version()?;
    let latest_version = DownloadedVersion::from_asset(&asset);

    if current_version.as_ref() != Some(&latest_version) {
        println!("New version found; downloading it");

        let (progress_updates_tx, progress_updates_rx) = flume::bounded(16);

        let download_size = asset.size as u64;

        tokio::task::spawn(async move {
            download_new_version(&asset, progress_updates_tx)
                .await
                .expect("failed to download new version");
            set_downloaded_version(latest_version).expect("failed to set downloaded version");
            launch_game().expect("failed to launch game");
            exit(0);
        });

        show_launcher_window(download_size, progress_updates_rx);
    } else {
        println!("Up to date");
        launch_game().expect("failed to launch game");
        exit(0);
    }

    Ok(())
}

async fn get_latest_tarball_info(github: &Octocrab) -> anyhow::Result<Asset> {
    let release = github
        .repos("caelunshun", "riposte")
        .releases()
        .get_latest()
        .await?;

    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name.contains(OS_SPECIFIER))
        .ok_or_else(|| {
            anyhow!(
                "could not find a release asset that corresponds to this OS ({})",
                OS_SPECIFIER
            )
        })?;

    Ok(asset)
}

fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("me.caelunshun", "", "Riposte").expect("failed to get home directory")
}

fn riposte_unpack_dir() -> PathBuf {
    project_dirs().data_dir().join("game")
}

fn version_file_path() -> PathBuf {
    project_dirs().config_dir().join("downloaded_version.json")
}

fn get_downloaded_version() -> anyhow::Result<Option<DownloadedVersion>> {
    let path = version_file_path();
    if !path.exists() {
        return Ok(None);
    }

    let version = serde_json::from_slice(&fs::read(&path)?)?;
    Ok(Some(version))
}

fn set_downloaded_version(version: DownloadedVersion) -> anyhow::Result<()> {
    fs::create_dir_all(project_dirs().config_dir()).ok();
    fs::write(version_file_path(), serde_json::to_vec_pretty(&version)?)?;
    Ok(())
}

async fn download_new_version(
    asset: &Asset,
    progress_updates: flume::Sender<Message>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    let mut response = client
        .get(asset.url.clone())
        .header(header::ACCEPT, "application/octet-stream")
        .header(header::USER_AGENT, "riposte")
        .send()
        .await?;

    println!("Download status: {}", response.status());

    let mut tarball_bytes = Vec::new();

    loop {
        let chunk = response.chunk().await?;

        match chunk {
            Some(bytes) => {
                tarball_bytes.extend_from_slice(&bytes);
                progress_updates
                    .send_async(Message::DownloadProgress(DownloadProgress {
                        downloaded: tarball_bytes.len() as u64,
                        needed: asset.size.try_into()?,
                    }))
                    .await?;
            }
            None => break,
        }
    }

    let mut archive = Archive::new(zstd::Decoder::new(Cursor::new(&tarball_bytes))?);

    let unpack_dir = riposte_unpack_dir();
    println!("Unpacking to {}", unpack_dir.display());
    fs::remove_dir_all(&unpack_dir).ok();
    fs::create_dir_all(&unpack_dir)?;
    archive.unpack(&unpack_dir)?;

    Ok(())
}

struct Positioner;

impl WindowPositioner for Positioner {
    fn compute_position(&self, available_space: Vec2) -> Rect {
        Rect::new(Vec2::ZERO, available_space)
    }
}

fn show_launcher_window(_download_size: u64, progress_updates: flume::Receiver<Message>) {
    let event_loop = EventLoop::new();
    let window = build_window(&event_loop);

    let mut ui = Ui::new();
    ui.add_spec(Spec::deserialize_from_str(include_str!("../assets/root.yml")).unwrap());
    ui.add_stylesheet(
        r#"
styles:
  text:
    default_font_family: Merriweather
  no_border:
    border_width: 0
  progress_bar:
    border_radius: 3
    "#
        .as_bytes(),
    )
    .unwrap();

    let (root, widget) = ui.create_spec_instance::<RiposteLauncher>();
    ui.create_window(widget, Positioner, 1);

    duit_platform::run(
        event_loop,
        window,
        ui,
        |cv| {
            cv.context()
                .add_font(include_bytes!("../../../assets/font/Merriweather-Regular.ttf").to_vec())
                .unwrap();
        },
        move |_| {
            if let Some(latest_progress) = progress_updates.try_iter().last() {
                match latest_progress {
                    Message::DownloadProgress(progress) => {
                        root.progress_bar
                            .get_mut()
                            .set_progress(progress.downloaded as f32 / progress.needed as f32);
                        root.progress_text.get_mut().set_text(duit::text!(
                            "{} / {} MiB",
                            format!("{:.1}", mib(progress.downloaded)),
                            format!("{:.1}", mib(progress.needed))
                        ));
                    }
                }
            }
        },
    );
}

fn build_window(event_loop: &EventLoop<()>) -> Window {
    WindowBuilder::new()
        .with_title("Riposte Launcher")
        .with_inner_size(LogicalSize::new(1920 / 4, 1080 / 4))
        .with_resizable(false)
        .with_decorations(false)
        .build(event_loop)
        .expect("failed to create window ")
}

fn launch_game() -> anyhow::Result<()> {
    let game_dir = riposte_unpack_dir();
    let executable = game_dir.join("Riposte");

    process::Command::new(executable)
        .current_dir(&game_dir)
        .env(LD_ENV, ".")
        .spawn()?
        .wait()?;

    Ok(())
}

#[derive(Copy, Clone, Debug, Default)]
struct DownloadProgress {
    downloaded: u64,
    needed: u64,
}

#[derive(Debug, Clone)]
enum Message {
    DownloadProgress(DownloadProgress),
}

fn mib(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0
}
