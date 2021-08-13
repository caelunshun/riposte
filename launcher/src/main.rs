use std::{
    convert::TryInto,
    fs,
    hash::{Hash, Hasher},
    io::Cursor,
    path::PathBuf,
    process::{self, exit},
};

use anyhow::anyhow;
use directories_next::ProjectDirs;
use iced::{
    executor,
    window::{self, Position},
    Align, Application, Clipboard, Column, Command, Element, Length, ProgressBar, Settings, Text,
};
use iced_futures::subscription::Recipe;
use octocrab::{models::repos::Asset, Octocrab};
use reqwest::header;
use serde::{Deserialize, Serialize};
use tar::Archive;

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

const GITHUB_ACCESS_TOKEN: &str = "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";

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
    let github = octocrab::OctocrabBuilder::new()
        .personal_token(GITHUB_ACCESS_TOKEN.to_owned())
        .build()?;

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

        show_launcher_window(download_size, progress_updates_rx).expect("failed to open GUI");
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
        .basic_auth("caelunshun", Some(GITHUB_ACCESS_TOKEN))
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

fn show_launcher_window(
    download_size: u64,
    progress_updates: flume::Receiver<Message>,
) -> anyhow::Result<()> {
    RiposteLauncher::run(Settings {
        window: window::Settings {
            size: (1920 / 4, 1080 / 4),
            resizable: false,
            decorations: false,
            position: Position::Centered,
            ..Default::default()
        },
        flags: (
            DownloadProgress {
                needed: download_size,
                downloaded: 0,
            },
            progress_updates,
        ),
        default_font: Some(include_bytes!("../../assets/font/Merriweather-Regular.ttf")),
        antialiasing: true,
        default_text_size: 20,
        text_multithreading: false,
        exit_on_close_request: true,
    })?;
    Ok(())
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

struct RiposteLauncher {
    download_progress: DownloadProgress,
    progress_updates: flume::Receiver<Message>,
}

impl Application for RiposteLauncher {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = (DownloadProgress, flume::Receiver<Message>);

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                download_progress: flags.0,
                progress_updates: flags.1,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Riposte Launcher".to_owned()
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::DownloadProgress(progress) => self.download_progress = progress,
        }

        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        Column::new()
            .align_items(Align::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .push(Text::new("Updating Riposte").size(30))
            .push(Text::new(format!(
                "{:.1} / {:.1} MiB",
                mib(self.download_progress.downloaded),
                mib(self.download_progress.needed)
            )))
            .push(ProgressBar::new(
                0.0..=1.0,
                self.download_progress.downloaded as f32 / self.download_progress.needed as f32,
            ))
            .into()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::from_recipe(DownloadRecipe {
            receiver: self.progress_updates.clone(),
        })
    }
}

struct DownloadRecipe {
    receiver: flume::Receiver<Message>,
}

impl<H: Hasher, E> Recipe<H, E> for DownloadRecipe {
    type Output = Message;

    fn hash(&self, state: &mut H) {
        0u8.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: iced_futures::BoxStream<E>,
    ) -> iced_futures::BoxStream<Self::Output> {
        Box::pin(self.receiver.into_stream())
    }
}

fn mib(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0
}
