
use handlebars::template::Parameter::Path;
use std::io::Error;
use std::path::PathBuf;
use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_files::NamedFile;
use http::header::SET_COOKIE;
use http::HeaderValue;

#[get("/download/{filename}")]
async fn download_apk(
    req: HttpRequest,
    path: web::Path<String>
) -> Result<HttpResponse, actix_web::Error> {
    let filename = path.into_inner();
    log::debug!("Requested APK filename: {}", filename);

    // 1. Extract clickId from the query string, if present
    //    e.g. /download/foo.apk?clickId=XYZ
    let click_id_opt = req
        .uri()
        .query()                       // Option<&str>
        .and_then(|q| {
            // parse query string into key/value pairs
            url::form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "clickId")
                .map(|(_, v)| v.into_owned())
        });

    // 2. Build the path to your APK
    let file_path = PathBuf::from("assets").join(&filename);

    // 3. Open the NamedFile
    let named_file = NamedFile::open(&file_path)?;

    // 4. Convert to HttpResponse so we can modify headers
    let mut response = named_file
        .set_content_type(mime::APPLICATION_OCTET_STREAM)
        .into_response(&req);

    // 5. If we found a clickId, set it in a cookie
    if let Some(click_id) = click_id_opt {
        // Basic HttpOnly cookie; adjust attributes (Max-Age, Secure, etc.) as needed
        let cookie_value = format!("clickId={}; Path=/; Max-Age=86400; HttpOnly", click_id);
        response.headers_mut().insert(
            SET_COOKIE,
            HeaderValue::from_str(&cookie_value)?
        );
    }
    Ok(response)
}
