use bytes::buf::Buf;
use log::error;
use num_cpus;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_longlong};
use std::path::Path;

use anyhow::Result;
use std::fs::File;
use std::io::copy;
use tempfile::Builder;

use crate::message::{SqipCreateMessage, SqipDoneMessage};

extern "C" {
    fn MakeSVG(
        path: GoString,
        number_of_primitives: c_longlong,
        mode: c_longlong,
        alpha: c_longlong,
        workers: c_longlong,
    ) -> *const c_char;
}

/// See [here](http://blog.ralch.com/tutorial/golang-sharing-libraries/) for `GoString` struct layout
// See the generated header file: libsqif.h
#[repr(C)]
struct GoString {
    a: *const c_char,
    b: isize,
}

pub async fn generate_sqip(request: &SqipCreateMessage) -> Result<SqipDoneMessage> {
    let mut done_message = SqipDoneMessage::new(request.film_id.clone());
    let tmp_dir = Builder::new().prefix(&request.film_id).tempdir()?;
    if let Some(backdrop_path) = &request.backdrop_path {
        let file_url = format!("https://image.tmdb.org/t/p/w1280{}", backdrop_path);
        done_message.backdrop_svg_base64encoded =
            Some(get_film_data(file_url, &backdrop_path, &request.film_id, &tmp_dir).await?);
    }
    if let Some(poster_path) = &request.poster_path {
        let file_url = format!("https://image.tmdb.org/t/p/w500{}", poster_path);
        done_message.poster_svg_base64encoded =
            Some(get_film_data(file_url, &poster_path, &request.film_id, &tmp_dir).await?);
    }
    tmp_dir.close()?;
    Ok(done_message)
}

async fn get_film_data(
    file_url: String,
    path: &str,
    film_id: &str,
    tmp_dir: &tempfile::TempDir,
) -> Result<String> {
    let response = reqwest::get(file_url).await?;
    let file_name = Path::new(&path).file_name().unwrap().to_string_lossy();
    let file_name = format!("{}-1280-backdrop-{}", film_id, file_name);
    let file_name = tmp_dir.path().join(file_name);
    let mut dest = File::create(&file_name)?;
    let content = response.bytes().await?;
    copy(&mut content.reader(), &mut dest)?;
    Ok(make_sqip(&file_name.to_str().unwrap())?)
}

fn make_sqip(path: &str) -> Result<String> {
    let c_path = CString::new(path).expect("CString::new failed");
    let ptr = c_path.as_ptr();
    let go_string = GoString {
        a: ptr,
        b: c_path.as_bytes().len() as isize,
    };
    let number_of_primitives: c_longlong = 10;
    let mode: c_longlong = 0;
    let alpha: c_longlong = 128;
    let workers: c_longlong = num_cpus::get() as c_longlong;

    let result = unsafe { MakeSVG(go_string, number_of_primitives, mode, alpha, workers) };
    let c_str = unsafe { CStr::from_ptr(result) };
    let string = c_str.to_str().expect("Error translating SQIP from library");
    if string.is_empty() || string.starts_with("Error") {
        error!("Failed to get SQIP from SQIP library: {}", string);
        Err(anyhow::anyhow!("Failed to get SQIP"))
    } else {
        Ok(base64::encode(&string))
    }
}
