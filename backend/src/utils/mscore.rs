use std::{
    io::Error,
    path::PathBuf,
    process::{Command, Output},
};

use axum::http::StatusCode;

use crate::errors::ApiError;

#[derive(Clone)]
pub struct MscoreManager {
    soundfont_path: PathBuf,
}

impl MscoreManager {
    pub fn new(soundfont_path: PathBuf) -> Self {
        Self { soundfont_path }
    }

    pub async fn generate_audio(
        &self,
        midi_path: &PathBuf,
        output_path: &PathBuf,
    ) -> Result<Output, Error> {
        Command::new("fluidsynth")
            .arg("-ni")
            .arg("-T")
            .arg("flac")
            .arg("-F")
            .arg(output_path)
            .arg("-r")
            .arg("44100")
            .arg(&self.soundfont_path)
            .arg(midi_path)
            .output()
    }

    pub async fn generate(
        &self,
        midi_path: &PathBuf,
        output_path: &PathBuf,
    ) -> Result<Output, Error> {
        Command::new("mscore")
            .env("QT_LOGGING_RULES", "qt.qml.typeregistration=false")
            .arg(&midi_path)
            .arg("-o")
            .arg(&output_path)
            .output()
    }
    pub fn make_temp_paths(&self, ext: &str) -> (PathBuf, PathBuf) {
        let tmp_dir = std::env::temp_dir();
        let id = uuid::Uuid::new_v4();

        let midi_path = tmp_dir.join(format!("{}.mid", id));
        let out_path = tmp_dir.join(format!("{}.{}", id, ext));

        (midi_path, out_path)
    }

    pub async fn write_midi_file(&self, path: &PathBuf, midi_bytes: &[u8]) -> Result<(), ApiError> {
        tokio::fs::write(path, midi_bytes).await.map_err(|e| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Write MIDI failed: {}", e),
            )
        })
    }

    pub async fn read_generated_file(
        &self,
        path: &PathBuf,
        stderr: &[u8],
        label: &str,
    ) -> Result<Vec<u8>, ApiError> {
        let meta = tokio::fs::metadata(path).await.map_err(|_| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Renderer failed generating {}: {}",
                    label,
                    String::from_utf8_lossy(stderr),
                ),
            )
        })?;

        if meta.len() == 0 {
            return Err(ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Renderer produced empty {}", label),
            ));
        }

        tokio::fs::read(path).await.map_err(|e| {
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Read {} failed: {}", label, e),
            )
        })
    }

    pub async fn cleanup_files(&self, paths: &[PathBuf]) {
        for p in paths {
            let _ = tokio::fs::remove_file(p).await;
        }
    }
}
