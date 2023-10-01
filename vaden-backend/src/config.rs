use crate::error::{Result, VadenError};
use crate::password::hash_password;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use tokio::fs;
use yaml_rust::{Yaml, YamlLoader};

const DOMAIN: &str = "domain";
const HTTP_PROXY: &str = "http_proxy";
const HTTPS_PROXY: &str = "https_proxy";
const CERT: &str = "cert";
const USERNAME: &str = "username";
const PASSWORD: &str = "password";
const ADMIN: &str = "admin";

/// Main Vaden config.
#[derive(Debug)]
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
            password: hash_password(password)?,
            admin: if_empty_then(admin, "0.0.0.0:8444"),
        })
    }

    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut config = String::with_capacity(1024);
        write!(&mut config, "{}: {}\r\n", DOMAIN, self.domain)?;
        write!(&mut config, "{}: {}\r\n", HTTP_PROXY, self.http_proxy)?;
        write!(&mut config, "{}: {}\r\n", HTTPS_PROXY, self.https_proxy)?;
        write!(
            &mut config,
            "{}: {}\r\n",
            CERT,
            self.cert.to_str().ok_or(VadenError::InvalidConfig(
                "Invalid certificate path".to_string()
            ))?
        )?;
        write!(&mut config, "{}: {}\r\n", USERNAME, self.username)?;
        write!(&mut config, "{}: {}\r\n", PASSWORD, self.password)?;
        write!(&mut config, "{}: {}\r\n", ADMIN, self.admin)?;
        fs::write(path, config).await?;
        Ok(())
    }

    pub async fn read(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let yaml = &YamlLoader::load_from_str(&content)?[0];
        println!("{:?}", yaml);
        Ok(Self {
            domain: from_yaml(DOMAIN, yaml)?,
            http_proxy: from_yaml(HTTP_PROXY, yaml)?,
            https_proxy: from_yaml(HTTPS_PROXY, yaml)?,
            cert: from_yaml(CERT, yaml)?.into(),
            username: from_yaml(USERNAME, yaml)?,
            password: from_yaml(PASSWORD, yaml)?,
            admin: from_yaml(ADMIN, yaml)?,
        })
    }
}

fn from_yaml(value: &str, yaml: &Yaml) -> Result<String> {
    Ok(yaml[value]
        .as_str()
        .ok_or(VadenError::InvalidConfig(format!(
            "Missing config value `{}`",
            value
        )))?
        .to_string())
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
