use bollard::{Docker, body_full};
use futures_util::stream::StreamExt;
use std::{ffi::OsStr, io, path::PathBuf};
use tar::Builder;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum OrchestratorKind {
    K8S,
    Docker,
}

#[derive(Debug, Clone)]
pub struct Context {
    _orchestrator: OrchestratorKind,
}

#[derive(Debug, Error)]
pub enum AppNotFound {
    #[error("Could not find application source")]
    InvalidApplicationPath,
    #[error("No Dockerfile found at given path")]
    DockerfileMissing,
    #[error("Docker ClI is not available: {0}")]
    DockerMissing(String),
    #[error("Docker build failed: {0}")]
    BuildFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

impl Default for Context {
    fn default() -> Self {
        Self::new(OrchestratorKind::Docker)
    }
}

impl Context {
    pub fn new(orchestrator: OrchestratorKind) -> Self {
        Self { _orchestrator: orchestrator }
    }

    pub async fn load_application(&self, path: &PathBuf, _build: bool) -> Result<(), AppNotFound> {
        if !path.exists() || !path.is_dir() {
            return Err(AppNotFound::InvalidApplicationPath);
        }

        let image_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("app")
            .to_string()
            .to_lowercase();

        let image_tag = format!("{}:latest", image_name);

        let docker = Docker::connect_with_defaults()
            .map_err(|e| AppNotFound::DockerMissing(e.to_string()))?;

        docker
            .version()
            .await
            .map_err(|e| AppNotFound::DockerMissing(e.to_string()))?;

        let build_image_options = bollard::query_parameters::BuildImageOptionsBuilder::default()
            .dockerfile("Dockerfile")
            .t(&image_tag)
            .pull("true")
            .rm(true);

        let mut contents = Vec::new();
        {
            let mut tar = Builder::new(&mut contents);
            tar.append_dir_all(".", path)?;
            tar.finish()?;
        }

        let mut image_build_stream = docker.build_image(
            build_image_options.build(),
            None,
            Some(body_full(contents.into())),
        );

        while let Some(msg) = image_build_stream.next().await {
            println!("message: {msg:?}");
        }

        Ok(())
    }
}
