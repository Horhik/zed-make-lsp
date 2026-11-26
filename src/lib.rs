use std::env;
use std::path::PathBuf;

use zed_extension_api::{
    self as zed, Architecture, Command, DownloadedFileType, LanguageServerId,
    LanguageServerInstallationStatus, Os, Result,
};

const SERVER_ID: &str = "make-ls";
const SERVER_VERSION: &str = "0.1.25";
const SERVER_DIR: &str = "make-ls-0.1.25";

struct MakeLsExtension {
    binary_path: Option<PathBuf>,
}

impl MakeLsExtension {
    fn resolve_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<PathBuf> {
        // A user-managed binary always wins (`go install`, package manager, ...).
        if let Some(path) = worktree.which(SERVER_ID) {
            return Ok(PathBuf::from(path));
        }

        if let Some(path) = &self.binary_path {
            if path.is_file() {
                return Ok(path.clone());
            }
        }

        let relative = PathBuf::from(SERVER_DIR).join(SERVER_ID);
        if !relative.is_file() {
            self.download(language_server_id)?;
        }

        let binary = env::current_dir()
            .map_err(|error| format!("cannot resolve the extension work directory: {error}"))?
            .join(relative);
        if !binary.is_file() {
            return Err(format!(
                "make-ls binary not found at {} after installation",
                binary.display()
            ));
        }

        self.binary_path = Some(binary.clone());
        Ok(binary)
    }

    fn download(&self, language_server_id: &LanguageServerId) -> Result<()> {
        let target = match zed::current_platform() {
            (Os::Linux, Architecture::X8664) => "linux_amd64",
            (Os::Linux, Architecture::Aarch64) => "linux_arm64",
            (Os::Mac, Architecture::X8664) => "darwin_amd64",
            (Os::Mac, Architecture::Aarch64) => "darwin_arm64",
            (os, arch) => {
                return Err(format!(
                    "make-ls has no prebuilt binary for {os:?}/{arch:?}; install it with \
                     `go install github.com/owenrumney/make-ls/cmd/make-ls@latest` and make sure \
                     `make-ls` is on PATH"
                ));
            }
        };

        let url = format!(
            "https://github.com/owenrumney/make-ls/releases/download/v{SERVER_VERSION}/make-ls_{target}.tar.gz"
        );

        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::Downloading,
        );

        if let Err(error) = zed::download_file(&url, SERVER_DIR, DownloadedFileType::GzipTar) {
            zed::set_language_server_installation_status(
                language_server_id,
                &LanguageServerInstallationStatus::Failed(error.clone()),
            );
            return Err(format!(
                "failed to download make-ls v{SERVER_VERSION} from {url}: {error}"
            ));
        }

        zed::make_file_executable(&format!("{SERVER_DIR}/{SERVER_ID}"))?;
        Ok(())
    }
}

impl zed::Extension for MakeLsExtension {
    fn new() -> Self {
        Self { binary_path: None }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Command> {
        let binary = self.resolve_binary(language_server_id, worktree)?;
        Ok(Command {
            command: binary.to_string_lossy().into_owned(),
            args: Vec::new(),
            env: Default::default(),
        })
    }
}

zed::register_extension!(MakeLsExtension);
