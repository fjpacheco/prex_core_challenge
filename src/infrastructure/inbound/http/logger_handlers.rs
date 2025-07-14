use actix_web::{HttpResponse, http::header, web::Path};
use chrono::Local;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, BufReader};

const LOGS_DIR: &str = "logs";

// GET /logs/files/list - List all log files in ./logs
#[macro_export]
macro_rules! GET_LOG_FILES_METHOD {
    () => {
        actix_web::web::get()
            .to($crate::infrastructure::inbound::http::logger_handlers::get_log_files)
    };
}
pub const GET_LOG_FILES_ROUTE: &str = "/logs/files/list";
pub async fn get_log_files() -> HttpResponse {
    tracing::info!("Requesting list log files");
    match fs::read_dir(LOGS_DIR).await {
        Ok(mut entries) => {
            let mut files = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(ft) = entry.file_type().await {
                    if ft.is_file() {
                        files.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
            files.sort();
            HttpResponse::Ok().json(files)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error reading log dir: {e}")),
    }
}

// GET /logs/files/{filename} - Download a specific log file
#[macro_export]
macro_rules! GET_LOG_FILE_METHOD {
    () => {
        actix_web::web::get()
            .to($crate::infrastructure::inbound::http::logger_handlers::get_log_file)
    };
}
pub const GET_LOG_FILE_ROUTE: &str = "/logs/files/{filename}";
pub async fn get_log_file(path: Path<(String,)>) -> HttpResponse {
    tracing::info!("Requesting download log file");
    let filename = &path.0;
    let file_path = PathBuf::from(LOGS_DIR).join(filename);
    if fs::metadata(&file_path)
        .await
        .map(|m| !m.is_file())
        .unwrap_or(true)
    {
        return HttpResponse::NotFound().body("File not found");
    }
    match fs::read(&file_path).await {
        Ok(data) => HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, "text/plain"))
            .insert_header((
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ))
            .body(data),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error reading file: {e}")),
    }
}

// GET /logs/files/download - Download all log files as a zip
#[macro_export]
macro_rules! GET_ALL_LOGS_ZIP_METHOD {
    () => {
        actix_web::web::get()
            .to($crate::infrastructure::inbound::http::logger_handlers::get_all_logs_zip)
    };
}
pub const GET_ALL_LOGS_ZIP_ROUTE: &str = "/logs/files/download";
pub async fn get_all_logs_zip() -> HttpResponse {
    tracing::info!("Requesting download all logs");
    use std::io::Write;
    let mut buffer = Vec::new();
    let writer = std::io::Cursor::new(&mut buffer);
    let mut zip = zip::ZipWriter::new(writer);
    let options: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    match fs::read_dir(LOGS_DIR).await {
        Ok(mut entries) => {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if fs::metadata(&path)
                    .await
                    .map(|m| m.is_file())
                    .unwrap_or(false)
                {
                    if let Ok(data) = fs::read(&path).await {
                        let fname = path.file_name().unwrap().to_string_lossy();
                        let _ = zip.start_file(fname, options);
                        let _ = zip.write_all(&data);
                    }
                }
            }
            let _ = zip.finish();
            let filename = format!("logs_{}.zip", Local::now().format("%Y-%m-%d_%H-%M-%S"));
            HttpResponse::Ok()
                .insert_header((header::CONTENT_TYPE, "application/zip"))
                .insert_header((header::CONTENT_DISPOSITION, format!("attachment; filename={filename}")))
                .body(buffer)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error reading log dir: {e}")),
    }
}

// DELETE /logs/files/{filename} - Delete or truncate a specific log file
#[macro_export]
macro_rules! DELETE_LOG_FILE_METHOD {
    () => {
        actix_web::web::delete()
            .to($crate::infrastructure::inbound::http::logger_handlers::delete_log_file)
    };
}
pub const DELETE_LOG_FILE_ROUTE: &str = "/logs/files/{filename}";
pub async fn delete_log_file(path: Path<(String,)>) -> HttpResponse {
    tracing::info!("Requesting delete log file");
    let filename = &path.0;
    let file_path = PathBuf::from(LOGS_DIR).join(filename);
    if fs::metadata(&file_path)
        .await
        .map(|m| !m.is_file())
        .unwrap_or(true)
    {
        return HttpResponse::NotFound().body("File not found");
    }
    let today = Local::now().format("app.log.%Y-%m-%d").to_string();
    let is_today = filename == &today;
    if is_today {
        // Truncar (vaciar) el archivo de hoy en vez de borrarlo
        match fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&file_path)
            .await
        {
            Ok(_) => HttpResponse::Ok().body("Today's log file truncated (emptied)"),
            Err(e) => HttpResponse::InternalServerError()
                .body(format!("Could not truncate today's log file: {e}")),
        }
    } else {
        match fs::remove_file(&file_path).await {
            Ok(_) => HttpResponse::Ok().body("File deleted"),
            Err(e) => HttpResponse::InternalServerError().body(format!("Error deleting file: {e}")),
        }
    }
}

// DELETE /logs/files/delete - Delete all log files (truncate today's)
#[macro_export]
macro_rules! DELETE_ALL_LOG_FILES_METHOD {
    () => {
        actix_web::web::delete()
            .to($crate::infrastructure::inbound::http::logger_handlers::delete_all_log_files)
    };
}
pub const DELETE_ALL_LOG_FILES_ROUTE: &str = "/logs/files/delete";
pub async fn delete_all_log_files() -> HttpResponse {
    tracing::info!("Requesting delete all logs");
    let today = Local::now().format("app.log.%Y-%m-%d").to_string();
    let mut truncated_today = false;
    match fs::read_dir(LOGS_DIR).await {
        Ok(mut entries) => {
            let mut errors = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if fs::metadata(&path)
                    .await
                    .map(|m| m.is_file())
                    .unwrap_or(false)
                {
                    let fname = path.file_name().map(|f| f.to_string_lossy().to_string());
                    if let Some(fname) = fname {
                        if fname == today {
                            // Truncar el archivo de hoy en vez de borrarlo
                            match fs::OpenOptions::new()
                                .write(true)
                                .truncate(true)
                                .open(&path)
                                .await
                            {
                                Ok(_) => truncated_today = true,
                                Err(e) => {
                                    errors.push(format!("Could not truncate today's log file: {e}"))
                                }
                            }
                        } else if let Err(e) = fs::remove_file(&path).await {
                            errors.push(format!("{}: {e}", path.display()));
                        }
                    }
                }
            }
            if errors.is_empty() {
                if truncated_today {
                    HttpResponse::Ok()
                        .body("All log files deleted, today's log file truncated (emptied)")
                } else {
                    HttpResponse::Ok().body("All log files deleted")
                }
            } else {
                HttpResponse::InternalServerError().body(format!(
                    "Some files could not be deleted or truncated: {}",
                    errors.join(", ")
                ))
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Error reading log dir: {e}")),
    }
}

// GET /logs/files/tail/{n} - Get the last N lines from all log files (concatenated, ordered by filename)
#[macro_export]
macro_rules! GET_LOG_TAIL_METHOD {
    () => {
        actix_web::web::get()
            .to($crate::infrastructure::inbound::http::logger_handlers::get_log_tail)
    };
}
pub const GET_LOG_TAIL_ROUTE: &str = "/logs/files/tail/{n}";
pub async fn get_log_tail(path: Path<(usize,)>) -> HttpResponse {
    tracing::info!("Requesting tail logs");
    let n = path.0;
    let mut lines: Vec<String> = Vec::new();
    let mut files: BTreeMap<String, PathBuf> = BTreeMap::new();
    match fs::read_dir(LOGS_DIR).await {
        Ok(mut entries) => {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if fs::metadata(&path)
                    .await
                    .map(|m| m.is_file())
                    .unwrap_or(false)
                {
                    if let Some(fname) = path.file_name().map(|f| f.to_string_lossy().to_string()) {
                        files.insert(fname, path);
                    }
                }
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Error reading log dir: {e}"));
        }
    }
    for (_fname, path) in files.iter() {
        if let Ok(file) = fs::File::open(path).await {
            let reader = BufReader::new(file);
            let mut lines_stream = reader.lines();
            while let Ok(Some(line)) = lines_stream.next_line().await {
                lines.push(line);
            }
        }
    }
    let total = lines.len();
    let start = total.saturating_sub(n);
    let tail: Vec<&str> = lines[start..].iter().map(|s| s.as_str()).collect();
    HttpResponse::Ok().body(tail.join("\n"))
}
