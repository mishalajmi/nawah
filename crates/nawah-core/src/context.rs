use std::{
    ffi::OsStr,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum OrchestratorKind {
    K8S,
    Docker,
}

#[derive(Debug, Clone)]
pub struct NawahContext {
    orchestrator: OrchestratorKind,
    // route: RouteSpec,
    // apps: Vec<AppSpec>,
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

impl Default for NawahContext {
    fn default() -> Self {
        Self {
            orchestrator: OrchestratorKind::Docker,
            //     route: RouteSpec::default(),
            //     apps: Vec::new(),
        }
    }
}

impl NawahContext {
    pub fn new(orchestrator: OrchestratorKind) -> Self {
        NawahContext {
            orchestrator,
            // route: RouteSpec::default(),
            // apps: Vec::new(),
        }
    }

    pub fn load_application(&self, app_path: &str, build: bool) -> Result<String, AppNotFound> {
        // TODO: first check if the application exists in that path
        let path = Path::new(app_path);
        if !path.exists() || !path.is_dir() {
            return Err(AppNotFound::InvalidApplicationPath);
        }
        let image_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("app")
            .to_string();
        let image_tag = format!("{}:latest", image_name);

        // Look for candidates
        // Accept "Dockerfile", "dockerfile"
        let candidates: [PathBuf; 2] = [
            path.join("Dockerfile"),
            path.join("dockerfile").join("Dockerfile"),
        ];
        let dockerfile = candidates.iter().find(|p| p.exists());
        if dockerfile.is_none() {
            return Err(AppNotFound::DockerfileMissing);
        }

        if build {
            match Command::new("docker").arg("--version").output() {
                Ok(o) if o.status.success() => {
                    println!("Yay");
                }
                Ok(o) => {
                    return Err(AppNotFound::DockerMissing(format!(
                        "docker --version returned non-zero. stdout: {}, stderr: {}",
                        String::from_utf8_lossy(&o.stdout),
                        String::from_utf8_lossy(&o.stderr)
                    )));
                }
                Err(e) => {
                    return Err(AppNotFound::DockerMissing(format!(
                        "failed to execute docker: {}",
                        e
                    )));
                }
            }
        }

        let mut cmd = Command::new("docker");
        cmd.arg("build")
            .arg("-t")
            .arg(&image_tag)
            .arg(".")
            .current_dir(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(AppNotFound::IoError)?;

        if let Some(mut out) = child.stdout.take() {
            let mut buf = [0u8; 1024];
            loop {
                match out.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Err(e) = io::stdout().write_all(&buf[..n]) {
                            eprintln!("failed to write stdout: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("error reading docker stdout: {}", e);
                        break;
                    }
                }
            }
        }

        let mut stderr_capture = String::new();
        if let Some(mut err) = child.stderr.take() {
            let mut tmp_buf = Vec::new();
            if let Err(e) = err.read_to_end(&mut tmp_buf) {
                eprintln!("error reading docker stderr: {}", e);
            }
            stderr_capture = String::from_utf8_lossy(&tmp_buf).to_string();
            let _ = io::stderr().write_all(tmp_buf.as_slice());
        }

        let status = child.wait().map_err(AppNotFound::IoError)?;
        if !status.success() {
            return Err(AppNotFound::BuildFailed(stderr_capture));
        }

        // TODO: then check if it contains a Dockerfile
        Ok(image_tag)
    }
}
