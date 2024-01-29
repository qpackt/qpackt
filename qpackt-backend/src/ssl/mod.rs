// SPDX-License-Identifier: AGPL-3.0
/*
   qpackt: Web & Analytics Server
   Copyright (C) 2024 Łukasz Wojtów

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

/// Force panel and proxy redirect when ssl becomes available.
pub(crate) static FORCE_HTTPS_REDIRECT: AtomicBool = AtomicBool::new(false);

pub(crate) mod challenge;
pub(crate) mod resolver;

use crate::ssl::challenge::AcmeChallenge;
use acme_lib::create_p384_key;
use acme_lib::persist::FilePersist;
use acme_lib::{Account, Certificate};
use acme_lib::{Directory, DirectoryUrl};
use log::{debug, info};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

/// Tries to load existing certificate from run directory. If not found then ask LetsEncrypt.
pub(crate) async fn get_certificate(domain: &str, path: &PathBuf, acme_challenge: AcmeChallenge) -> Certificate {
    debug!("Getting TLS certificate");
    let acc = get_account(path, domain);
    if let Ok(Some(certificate)) = acc.certificate(domain) {
        if certificate.valid_days_left() > 1 {
            debug!("Using existing certificate ({} days left)", certificate.valid_days_left());
            return certificate;
        }
    }
    debug!("Getting new TLS certificate");
    // Order a new TLS certificate for a domain.
    let mut ord_new = acc.new_order(domain, &[]).unwrap();

    // If the ownership of the domain(s) have already been
    // authorized in a previous order, you might be able to
    // skip validation. The ACME API provider decides.
    let ord_csr = loop {
        debug!("Looping ACME validation");
        // are we done?
        if let Some(ord_csr) = ord_new.confirm_validations() {
            debug!("Found ACME validated order");
            break ord_csr;
        }

        // Get the possible authorizations (for a single domain
        // this will only be one element).
        let auths = ord_new.authorizations().unwrap();
        debug!("Found {} possible ACME validations", auths.len());
        // For HTTP, the challenge is a text file that needs to
        // be placed in your web server's root:
        //
        // /var/www/.well-known/acme-challenge/<token>
        //
        // The important thing is that it's accessible over the
        // web for the domain(s) you are trying to get a
        // certificate for:
        //
        // http://mydomain.io/.well-known/acme-challenge/<token>
        let challenge = auths[0].http_challenge();

        // The token is the filename.
        let token = challenge.http_token().to_string();
        debug!("Got ACME token: {}", token);

        // The proof is the contents of the file
        let proof = challenge.http_proof();
        debug!("Got ACME proof ({} characters)", proof.len());
        acme_challenge.set_challenge(token, proof).await;
        // Here you must do "something" to place
        // the file/contents in the correct place.
        // update_my_web_server(&path, &proof);

        // After the file is accessible from the web, the calls
        // this to tell the ACME API to start checking the
        // existence of the proof.
        //
        // The order at ACME will change status to either
        // confirm ownership of the domain, or fail due to the
        // not finding the proof. To see the change, we poll
        // the API with 5000 milliseconds wait between.
        debug!("Awaiting ACME validation");
        challenge.validate(5_000).unwrap();
        debug!("Refreshing ACME state");
        // Update the state against the ACME API.
        ord_new.refresh().unwrap();
    };
    info!("Received new TLS certificate");
    // Ownership is proven. Create a private key for
    // the certificate. These are provided for convenience, you
    // can provide your own keypair instead if you want.
    let pkey_pri = create_p384_key();

    // Submit the CSR. This causes the ACME provider to enter a
    // state of "processing" that must be polled until the
    // certificate is either issued or rejected. Again we poll
    // for the status change.
    debug!("Finalizing ACME key");
    let ord_cert = ord_csr.finalize_pkey(pkey_pri, 5000).unwrap();

    // Now download the certificate. Also stores the cert in
    // the persistence.
    debug!("Downloading TLS certificate");
    let cert = ord_cert.download_and_save_cert().unwrap();
    info!("Finished ACME challenge");
    cert
}


/// Reads the private account key from persistence, or
/// creates a new one before accessing the API to establish
/// that it's there.
fn get_account(path: &PathBuf, domain: &str) -> Account<FilePersist> {
    debug!("Getting TLS account");
    // Use DirectoryUrl::LetsEncryptStaging for dev/testing.
    // let url = DirectoryUrl::LetsEncryptStaging;
    let url = DirectoryUrl::LetsEncrypt;

    // Save/load keys and certificates to current dir.
    let persist = FilePersist::new(path);

    // Create a directory entrypoint.
    let dir = Directory::from_url(persist, url).unwrap();

    dir.account(format!("admin@{}", domain).as_str()).unwrap()
}
