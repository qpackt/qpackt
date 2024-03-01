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

mod reverse_proxy;
mod token;

use crate::run_app;
use std::env;
use std::path::{Path, PathBuf};
use tmpdir::TmpDir;
use tokio::fs;

pub(super) async fn write_config_file(dir: &Path) -> PathBuf {
    let content =  format!("domain: qpackt.com\nhttp_proxy: 0.0.0.0:8080\npassword: $scrypt$ln=17,r=8,p=1$H63UY378M+ql3bpQMQ37aQ$XXt3kOaWrW/CQr+/lPIDtPlPTLJSHbaaGBEVo3l3wFY\nrun_directory: {}", dir.to_str().unwrap());
    let config = dir.join("qpackt.yaml");
    fs::write(&config, content).await.expect("Unable to write config");
    config
}

/// We need to hold on to [TmpDir] for the duration of the test to ensure it's not dropped (and therefore deleted).
#[must_use]
pub(super) async fn build_config_and_run_app() -> TmpDir {
    let dir = TmpDir::new("qpackt_dir").await.expect("Unable to create temp dir");
    let config = write_config_file(&dir.to_path_buf()).await;
    env::set_var("QPACKT_HTML_DIR", ".");
    run_app(&config).await;
    dir
}
