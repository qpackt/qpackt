// SPDX-License-Identifier: AGPL-3.0
/*
   qpackt: Web & Analytics Server
   Copyright (C) 2023 Łukasz Wojtów

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU Affero General Public License as
   published by the Free Software Foundation, either version 3 of the
   License.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU Affero General Public License for more details.

   You should have received a copy of the GNU Affero General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::error::{QpacktError, Result};
use crate::password::hash_password;
use std::fmt::Write;
use std::path::{Path, PathBuf};
use tokio::fs;
use yaml_rust::{Yaml, YamlLoader};

const DOMAIN: &str = "domain";
const HTTP_PROXY: &str = "http_proxy";
const HTTPS_PROXY: &str = "https_proxy";
const PASSWORD: &str = "password";
const RUN_DIR: &str = "run_directory";

/// Main qpackt config.
#[derive(Clone, Debug)]
pub(crate) struct QpacktConfig {
    /// Domain for which traffic should be accepted. Needed to request SSL certificate.
    domain: String,
    /// Host and port for HTTP (not SSL) traffic
    http_proxy: String, // TODO change this to sockaddr or something.
    /// Host and port for HTTPS traffic
    https_proxy: Option<String>,
    /// Administrator's password encoded in `scrypt` format
    password: String,
    /// Directory to hold database, docker images etc...
    run_directory: PathBuf,
}

impl QpacktConfig {
    /// Creates config by asking questions via stdin/out.
    pub(super) async fn create() {
        let config = QpacktConfig::new().unwrap();
        let path = "qpackt.yaml";
        config.save(path).await.unwrap();
        println!("Config file saved in {}", path);
    }

    /// Saves config to the given file.
    pub(crate) async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut config = String::with_capacity(1024);
        write!(&mut config, "{}: {}\r\n", DOMAIN, self.domain)?;
        write!(&mut config, "{}: {}\r\n", HTTP_PROXY, self.http_proxy)?;
        if let Some(https_proxy) = self.https_proxy.as_ref() {
            write!(&mut config, "{}: {}\r\n", HTTPS_PROXY, https_proxy)?;
        }
        write!(&mut config, "{}: {}\r\n", PASSWORD, self.password)?;
        write!(
            &mut config,
            "{}: {}\r\n",
            RUN_DIR,
            self.run_directory.to_str().ok_or(QpacktError::InvalidConfig("Invalid run directory".to_string()))?
        )?;
        fs::write(path, config).await?;
        Ok(())
    }

    /// Read config from a file.
    pub(crate) async fn read(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let yaml = &YamlLoader::load_from_str(&content)?[0];
        Ok(Self {
            domain: from_yaml(DOMAIN, yaml)?.ok_or(QpacktError::InvalidConfig(format!("Missing config value `{}`", DOMAIN).to_string()))?,
            http_proxy: from_yaml(HTTP_PROXY, yaml)?
                .ok_or(QpacktError::InvalidConfig(format!("Missing config value `{}`", HTTP_PROXY).to_string()))?,
            https_proxy: from_yaml(HTTPS_PROXY, yaml)?,
            password: from_yaml(PASSWORD, yaml)?
                .ok_or(QpacktError::InvalidConfig(format!("Missing config value `{}`", PASSWORD).to_string()))?,
            run_directory: from_yaml(RUN_DIR, yaml)?
                .ok_or(QpacktError::InvalidConfig(format!("Missing config value `{}`", RUN_DIR).to_string()))?
                .into(),
        })
    }

    pub(crate) fn app_run_directory(&self) -> &PathBuf {
        &self.run_directory
    }


    pub(crate) fn http_proxy_addr(&self) -> &str {
        &self.http_proxy
    }

    /// Builds new config from stdin questions with some sane defaults.
    fn new() -> Result<QpacktConfig> {
        let domain = read_stdin("Domain")?;
        let http_proxy = read_stdin("Ip/port for HTTP traffic (default 0.0.0.0:8080)")?;
        let https_proxy = read_stdin("Ip/port for HTTPS traffic (leave empty for no HTTPS)")?;
        // TODO read twice, disable echoing.
        let password = read_stdin("Administrator's password")?;
        let run_directory = read_stdin("Run directory (default /var/run/qpackt)")?;
        Ok(QpacktConfig {
            domain,
            http_proxy: if_empty_then(http_proxy, "0.0.0.0:8080"),
            https_proxy: if https_proxy.is_empty() { None } else { Some(https_proxy) },
            password: hash_password(password)?,
            run_directory: if_empty_then(run_directory, "/var/run/qpackt").into(),
        })
    }

    pub(crate) fn domain(&self) -> &str {
        &self.domain
    }
    pub(crate) fn https_proxy_addr(&self) -> Option<&String> {
        self.https_proxy.as_ref()
    }
}

fn from_yaml(value: &str, yaml: &Yaml) -> Result<Option<String>> {
    Ok(yaml[value].clone().into_string())
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
