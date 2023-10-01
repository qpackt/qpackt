use crate::error::{Result, VadenError};
use std::fmt::Write;
use std::path::{Path, PathBuf};
use tokio::fs;

/// Main Vaden config.
pub struct Config {
    /// Domain for which traffic should be accepted. Needed to request SSL certificate.
    domain: String,
    /// Host and port for HTTP (not SSL) traffic
    http_proxy: String,
    /// Host and port for HTTPS traffic
    https_proxy: String,
    /// Path to file with SSL certificate. Should be writable for vaden user.
    cert: PathBuf,
    /// Administrator's user name (for logging into admin panel)
    username: String,
    /// Administrator's password encoded in `scrypt` format
    password: String,
    /// Host and port for administrator's panel.
    admin: String,
}

impl Config {
    pub fn new() -> Result<Config> {
        let domain = read_stdin("Domain")?;
        let http_proxy = read_stdin("Ip/port for HTTP traffic (default 0.0.0.0:8080)")?;
        let https_proxy = read_stdin("Ip/port for HTTPS traffic (default 0.0.0.0:8443)")?;
        let cert = read_stdin("Path to SSL certificate file (may not exist yet)")?;
        let username = read_stdin("Administrator's username (default 'admin')")?;
        let password = read_stdin("Administrator's password")?;
        let admin = read_stdin("Ip/port for admin panel (default 0.0.0.0:8444)")?;
        Ok(Config {
            domain,

            http_proxy: if_empty_then(http_proxy, "0.0.0.0:8080"),
            https_proxy: if_empty_then(https_proxy, "0.0.0.0:8443"),
            cert: cert.into(),
            username: if_empty_then(username, "admin"),
            password,
            admin: if_empty_then(admin, "0.0.0.0:8444"),
        })
    }

    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut config = String::with_capacity(1024);
        write!(&mut config, "domain: {}\r\n", self.domain)?;
        write!(&mut config, "http_proxy: {}\r\n", self.http_proxy)?;
        write!(&mut config, "https_proxy: {}\r\n", self.https_proxy)?;
        write!(
            &mut config,
            "cert: {}\r\n",
            self.cert.to_str().ok_or(VadenError::InvalidConfig(
                "Invalid certificate path".to_string()
            ))?
        )?;
        write!(&mut config, "username: {}\r\n", self.username)?;
        write!(&mut config, "password: {}\r\n", self.password)?;
        write!(&mut config, "admin: {}\r\n", self.admin)?;
        fs::write(path, config).await?;
        Ok(())
    }
}

fn read_stdin(prompt: &str) -> Result<String> {
    use std::io::Write;

    let mut buffer = String::new();
    print!("{}: ", prompt);
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut buffer)?;
    Ok(buffer.trim().to_string())
}

fn if_empty_then(value: String, default: &str) -> String {
    if !value.is_empty() {
        value
    } else {
        default.to_string()
    }
}
