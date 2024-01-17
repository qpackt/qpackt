// SPDX-License-Identifier: AGPL-3.0
/*
   Vaden: Versioned Application Deployment Engine
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

use std::collections::HashMap;
use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Clone)]
pub(crate) struct AcmeChallenge {
    sender: Sender<AcmeChallengeMessage>,
}

impl AcmeChallenge {
    pub(crate) async fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(16);
        spawn_receiver_task(receiver);
        Self { sender }
    }

    pub(crate) async fn get_proof(&self, token: String) -> Option<String> {
        let (s, r) = tokio::sync::oneshot::channel();
        self.sender.send(AcmeChallengeMessage::GetProof(token, s)).await.unwrap();
        r.await.unwrap()
    }

    pub(crate) async fn set_challenge(&self, token: String, proof: String) {
        self.sender.send(AcmeChallengeMessage::SetProof(token, proof)).await.unwrap();
    }

    pub(crate) async fn clear(&self) {
        self.sender.send(AcmeChallengeMessage::Clear).await.unwrap();
    }
}

enum AcmeChallengeMessage {
    GetProof(String, tokio::sync::oneshot::Sender<Option<String>>),
    SetProof(String, String),
    Clear,
}

fn spawn_receiver_task(mut receiver: Receiver<AcmeChallengeMessage>) {
    tokio::spawn(async move {
        let mut challenges: HashMap<String, String> = HashMap::new();
        while let Some(message) = receiver.recv().await {
            match message {
                AcmeChallengeMessage::GetProof(token, sender) => {
                    let proof = challenges.get(&token).map(|s| s.to_owned());
                    sender.send(proof).unwrap();
                }
                AcmeChallengeMessage::SetProof(token, proof) => {
                    challenges.insert(token, proof);
                }
                AcmeChallengeMessage::Clear => challenges.clear(),
            }
        }
    });
}
