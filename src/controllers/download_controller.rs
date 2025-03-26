
use handlebars::template::Parameter::Path;
use std::io::Error;
use std::path::PathBuf;
use actix_web::{get, web};
use actix_files::NamedFile;
#[get("/download/{filename}")]
async fn download_apk(filename: web::Path<String>) -> Result<NamedFile, Error> {
    log::debug!("filename .. {}", filename);
    let file_path = format!("assets/{}", filename);
    let pathx = PathBuf::from(file_path);
    
    Ok(NamedFile::open(pathx)?)
}
