// SPDX-License-Identifier: AGPL-3.0
/*
   Vaden: Versioned Application Deployment Engine
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

use crate::dao::requests::DailySeed;
use crate::dao::Dao;
use crate::error::Result;
use log::debug;
use rand::{thread_rng, Rng, RngCore};
use std::net::IpAddr;
use std::ops::Add;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

/// Visitor's hash. Created from daily seed, IP address and User-Agent
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct VisitorHash(u64);

impl From<VisitorHash> for i64 {
    fn from(value: VisitorHash) -> Self {
        value.0 as i64
    }
}

/// Currently used value to initiate calculating [VisitorHash].
static CURRENT_INIT: AtomicU64 = AtomicU64::new(0);

/// Refresh [DailySeed] used in calculating [VisitorHash]
const SEED_REFRESH_SECONDS: u64 = 24 * 60 * 60;

/// Reads [DailySeed] from the DB. If doesn't exist - creates one.
/// Also, starts a background task to refresh the seed once a day.
pub(crate) async fn init(dao: Dao) -> Result<()> {
    let seed = if let Some(seed) = dao.get_daily_seed().await? { seed } else { create_daily_seed(&dao).await? };
    CURRENT_INIT.store(seed.init, Ordering::Relaxed);
    spawn_refresh_loop(dao, seed);
    Ok(())
}

/// Creates a new [VisitorHash] from least significant bits of IP octet bytes and provided ident.
pub(crate) fn create(ip: IpAddr, ident: Vec<u8>) -> VisitorHash {
    let mut hash = CURRENT_INIT.load(Ordering::Relaxed);
    match &ip {
        IpAddr::V4(addr) => multiply(&mut hash, addr.octets().as_slice()),
        IpAddr::V6(addr) => multiply(&mut hash, addr.octets().as_slice()),
    };
    multiply(&mut hash, &ident);
    VisitorHash(hash)
}

fn spawn_refresh_loop(dao: Dao, seed: DailySeed) {
    tokio::spawn(async move {
        debug!("Started hash seed refresh task");
        let now = SystemTime::now();
        let next_refresh = if seed.expiration > now { seed.expiration } else { now };
        let mut delay = next_refresh.duration_since(now).unwrap();
        debug!("Calculated next hash seed refresh in: {:?}", delay);
        loop {
            tokio::time::sleep(delay).await;
            let init = thread_rng().gen_range(1..=u64::MAX);
            let expiration = SystemTime::now().add(Duration::from_secs(SEED_REFRESH_SECONDS));
            let seed = DailySeed { init, expiration };
            CURRENT_INIT.swap(init, Ordering::Relaxed);
            debug!("Updated hash seed value");
            delay = Duration::from_secs(SEED_REFRESH_SECONDS);
            dao.save_daily_seed(&seed).await.unwrap();
        }
    });
}

fn multiply(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        let (new, _) = hash.overflowing_mul((byte & 0b00001111) as u64);
        if new != 0 {
            *hash = new;
        }
    }
}

async fn create_daily_seed(dao: &Dao) -> Result<DailySeed> {
    let seed = DailySeed { init: thread_rng().next_u64(), expiration: SystemTime::now().add(Duration::from_secs(SEED_REFRESH_SECONDS)) };
    dao.save_daily_seed(&seed).await?;
    Ok(seed)
}
