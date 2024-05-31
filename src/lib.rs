use bytes::Bytes;
use reqwest;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use url::Url;

pub async fn get(url_str: &str) -> Result<Bytes, String> {
    println!("Fetching URL: {}", url_str);
    if let Ok(url) = Url::parse(url_str) {
        match url.scheme() {
            "http" | "https" => get_http(url_str).await,
            #[cfg(feature = "ftp")]
            "ftp" | "ftps" => get_ftp(url_str).await,
            "file" => {
                let mut path = PathBuf::new();
                if let Some(host) = url.host() {
                    path.push(host.to_string());
                }
                let path_str = url.path();
                get_file(path_str).await
            }
            _ => Err(format!("Unsupported scheme: {}", url.scheme())),
        }
    } else {
        get_file(url_str).await
    }
}

async fn get_file(path_str: &str) -> Result<Bytes, String> {
    let mut path = PathBuf::new();
    if let Some(stripped_path_str) = path_str.strip_prefix("/") {
        path.push(stripped_path_str);
    } else {
        path.push(path_str);
    }
    println!("Path: {}", path.display());
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }
    let mut file = match fs::File::open(&path) {
        Ok(file) => file,
        Err(err) => return Err(format!("Failed to open file: {}", err)),
    };

    let mut contents = Vec::new();
    match file.read_to_end(&mut contents) {
        Ok(_) => Ok(Bytes::copy_from_slice(&contents)),
        Err(err) => Err(format!("Failed to read file: {}", err)),
    }
}

async fn get_http(url: &str) -> Result<Bytes, String> {
    let response = match reqwest::get(url).await {
        Ok(response) => response,
        Err(err) => return Err(format!("Request failed: {}", err)),
    };
    response
        .bytes()
        .await
        .map_err(|err| format!("Failed to read response: {}", err))
}

#[cfg(feature = "ftp")]
async fn get_ftp(url: &str) -> Result<Bytes, String> {
    use suppaftp;
    let url_parts = Url::parse(url).map_err(|err| format!("Invalid URL: {}", err))?;
    let path = &url_parts.path();

    use suppaftp::native_tls::TlsConnector;
    use suppaftp::{NativeTlsConnector, NativeTlsFtpStream};

    if let None = url_parts.host() {
        return Err(format!("Invalid FTP URL: missing host"));
    }
    let host;
    if let url::Host::Domain(inner_host) = url_parts.host().unwrap() {
        host = inner_host;
    } else {
        return Err(format!("Invalid FTP URL: missing host"));
    }

    let port;
    if let Some(port_result) = url_parts.port() {
        port = port_result;
    } else {
        port = 21;
    }

    let ftp_stream = NativeTlsFtpStream::connect(format!("{}:{}", host, port))
        .map_err(|err| format!("Failed to connect to FTP server: {}", err))?;

    let tls_connector = NativeTlsConnector::from(TlsConnector::new().unwrap());

    let secured_ftp_stream = ftp_stream.into_secure(tls_connector, host);
    let mut ftp_stream = match secured_ftp_stream {
        Ok(secured_ftp_stream) => secured_ftp_stream,
        Err(err) => {
            println!("Failed to secure FTP connection: {}", err);
            println!("Attempting unsecured connection...");
            NativeTlsFtpStream::connect(format!("{}:{}", host, port))
                .map_err(|err| format!("Failed to connect to FTP server: {}", err))?
        }
    };

    let _ = ftp_stream.login(url_parts.username(), url_parts.password().unwrap_or(""));

    let data = ftp_stream
        .retr_as_buffer(path)
        .map_err(|err| format!("Error retrieving file: {}", err))?;

    if ftp_stream.quit().is_err() {
        Err("Failed to close FTP connection".to_string())
    } else {
        Ok(Bytes::from(data.into_inner()))
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;
    use tokio;

    #[tokio::test]
    async fn get_http_test() {
        let url = "http://example.com";
        println!("url: {}", url);
        let result = get(url).await;
        match result {
            Ok(bytes) => assert!(bytes.len() > 0),
            Err(err) => panic!("Error: {}", err),
        }
    }

    #[tokio::test]
    async fn get_https_test() {
        let url = "https://example.com";
        println!("url: {}", url);
        let result = get(url).await;
        match result {
            Ok(bytes) => assert!(bytes.len() > 0),
            Err(err) => panic!("Error: {}", err),
        }
    }

    #[cfg(feature = "ftp")]
    #[tokio::test]
    async fn get_ftp_test() {
        let url = "ftp://anonymous:guest@ftp.x.org:21/pub/current/index.html";
        println!("url: {}", url);
        let result = get(url).await;
        match result {
            Ok(bytes) => assert!(bytes.len() > 0),
            Err(err) => panic!("Error: {}", err),
        }
    }

    #[tokio::test]
    async fn get_file_test() {
        let result = get("Cargo.toml").await;
        if result.is_err() {
            panic!("Error: {}", result.err().unwrap());
        }
    }

    #[tokio::test]
    async fn get_file_url_test() {
        let current_directory = env::current_dir().unwrap();
        let path = current_directory.join("Cargo.toml");
        let url = Url::from_file_path(path.as_path()).unwrap();
        let result = get(&url.to_string()).await;
        if result.is_err() {
            panic!("Error: {}", result.err().unwrap());
        }
    }
}
