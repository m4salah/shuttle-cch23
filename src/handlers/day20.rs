use std::{
    fs::{read_dir, File},
    io::Read,
    process::Command,
};

use axum::{
    body::Bytes,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use bytes::Buf;
use tar::Archive;
use tempfile::tempdir;
use walkdir::WalkDir;

pub fn router() -> Router {
    Router::new()
        .route("/20/health", get(|| async { StatusCode::OK }))
        .route("/20/archive_files", post(archive_files))
        .route("/20/archive_files_size", post(archive_files_size))
        .route("/20/cookie", post(cookie))
}

async fn archive_files(file: Bytes) -> Result<String, StatusCode> {
    let extracted_temp_dir = tempdir().unwrap();
    tracing::info!("temp dir at {:?}", &extracted_temp_dir);
    let file_reader = file.reader();
    let mut archive = Archive::new(file_reader);

    archive.unpack(&extracted_temp_dir).map_err(|e| {
        tracing::error!("error while unpacking the archive file {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let count = read_dir(&extracted_temp_dir)
        .map_err(|e| {
            tracing::error!("error getting iter over the unpacked dir {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .count();
    Ok(format!("{count}"))
}

async fn archive_files_size(file: Bytes) -> Result<String, StatusCode> {
    let extracted_temp_dir = tempdir().unwrap();
    tracing::info!("temp dir at {:?}", &extracted_temp_dir);
    let file_reader = file.reader();
    let mut archive = Archive::new(file_reader);

    archive.unpack(&extracted_temp_dir).map_err(|e| {
        tracing::error!("error while unpacking the archive file {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let total_size = read_dir(&extracted_temp_dir)
        .map_err(|e| {
            tracing::error!("error getting iter over the unpacked dir {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .fold(0, |acc, file| file.unwrap().metadata().unwrap().len() + acc);
    Ok(format!("{total_size}"))
}

async fn cookie(file: Bytes) -> Result<String, StatusCode> {
    const BRANCH_NAME: &str = "christmas";
    let extracted_temp_dir = tempdir().unwrap();
    tracing::info!("temp dir at {:?}", &extracted_temp_dir);
    let file_reader = file.reader();
    let mut archive = Archive::new(file_reader);

    archive.unpack(&extracted_temp_dir).map_err(|e| {
        tracing::error!("error while unpacking the archive file {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let output = Command::new("git")
        .args([
            "log",
            "--format=%cn,%H",
            BRANCH_NAME, /* "--", "santa.txt"*/
        ])
        .current_dir(extracted_temp_dir.path())
        .output()
        .map_err(|e| {
            tracing::error!("error while unpacking the archive file {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !output.status.success() {
        tracing::error!("error in git command");
        return Err(StatusCode::OK);
    }

    for (author, commit) in String::from_utf8(output.clone().stdout)
        .unwrap()
        .lines()
        .map(|line| line.split_once(',').unwrap())
    {
        let output = Command::new("git")
            .args(["checkout", commit, "--force"])
            .current_dir(extracted_temp_dir.path())
            .output()
            .unwrap();

        if !output.status.success() {
            tracing::error!("error checking out {commit} with {author}");
            return Err(StatusCode::OK);
        }
        let found_santa = WalkDir::new(&extracted_temp_dir).into_iter().any(|f| {
            if let Ok(file) = f {
                if file.clone().file_type().is_file() && file.file_name() == "santa.txt" {
                    let mut temp_content = String::new();
                    File::open(file.path())
                        .unwrap()
                        .read_to_string(&mut temp_content)
                        .unwrap();
                    if temp_content.contains("COOKIE") {
                        return true;
                    }
                }
            }
            false
        });
        if found_santa {
            return Ok(format!("{author} {commit}"));
        }
    }

    Ok("".to_string())
}

#[cfg(test)]
mod tests {

    use super::*;
    use axum::http::StatusCode;
    use axum_test_helper::TestClient;

    #[tokio::test]
    async fn day20_health() {
        let app = router();
        let client = TestClient::new(app);
        let res = client.get("/20/health").send().await;
        assert_eq!(res.status(), StatusCode::OK);
    }
}
