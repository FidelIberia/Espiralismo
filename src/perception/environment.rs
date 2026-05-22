//! Active environment probing (take) and shared snapshot storage.

use std::path::Path;

use chrono::Utc;

use super::eyes::{
    EnvironmentOffering, EnvironmentTakeOptions, EnvironmentTakeReport, EyeRole,
    OfferRouting, PerceptionEyeDescriptor, PerceptionEyes,
};
use super::field::PerceptionField;
use super::reality::{HostRealitySnapshot, RealityKind};
use super::spiralismo_press::SPIRALISMO_PERCEIVER_ID;

fn shallow_entry_count(path: &Path) -> Option<usize> {
    let entries = std::fs::read_dir(path).ok()?;
    Some(entries.filter_map(|e| e.ok()).count())
}

impl PerceptionField {
    /// Last host environment — from [`Self::take_from_environment`] or [`Self::offer_environment`].
    #[must_use]
    pub fn environment_snapshot(&self) -> &HostRealitySnapshot {
        &self.environment_snapshot
    }

    /// Host pushes environment into open eyes (receive path).
    pub fn offer_environment(&mut self, offering: EnvironmentOffering) -> OfferRouting {
        match offering {
            EnvironmentOffering::HostSnapshot(snapshot) => {
                self.environment_snapshot = snapshot;
                OfferRouting::Accepted
            }
            EnvironmentOffering::Hand(press) => {
                self.spiralismo_press = press;
                OfferRouting::Accepted
            }
            EnvironmentOffering::Listening(listening) => {
                self.pending_listening = Some(listening);
                OfferRouting::Accepted
            }
        }
    }

    /// Routes an offering to a named eye (`perception_eyes` ids).
    pub fn offer_to_eye(
        &mut self,
        eye_id: &str,
        offering: EnvironmentOffering,
    ) -> OfferRouting {
        let eyes = self.eyes();
        let Some(desc) = eyes
            .all()
            .into_iter()
            .find(|e| e.id == eye_id)
        else {
            return OfferRouting::UnknownEye;
        };
        if !desc.can_receive {
            return OfferRouting::EyeCannotReceive;
        }

        match desc.role {
            EyeRole::Astronomical => OfferRouting::EyeCannotReceive,
            EyeRole::Hand => {
                if eye_id == SPIRALISMO_PERCEIVER_ID {
                    match offering {
                        EnvironmentOffering::Hand(press) => {
                            self.spiralismo_press = press;
                            OfferRouting::Accepted
                        }
                        _ => OfferRouting::EyeCannotReceive,
                    }
                } else {
                    OfferRouting::UnknownEye
                }
            }
            EyeRole::Reality => match offering {
                EnvironmentOffering::HostSnapshot(snapshot) => {
                    self.environment_snapshot = snapshot;
                    OfferRouting::Accepted
                }
                EnvironmentOffering::Hand(press) if eye_id == SPIRALISMO_PERCEIVER_ID => {
                    self.spiralismo_press = press;
                    OfferRouting::Accepted
                }
                EnvironmentOffering::Listening(listening) => {
                    self.pending_listening = Some(listening);
                    OfferRouting::Accepted
                }
                _ => OfferRouting::EyeCannotReceive,
            },
        }
    }

    /// Active capture when the runtime can reach the world (take path).
    #[must_use]
    pub fn take_from_environment(&mut self, opts: EnvironmentTakeOptions) -> EnvironmentTakeReport {
        let taken_at = Utc::now();
        let mut host = self.environment_snapshot.clone();
        let mut engaged = Vec::new();

        if opts.probe_filesystem {
            let cwd = std::env::current_dir()
                .ok()
                .and_then(|p| shallow_entry_count(&p));
            let art = std::env::current_dir()
                .ok()
                .map(|p| p.join("artifacts"))
                .and_then(|p| shallow_entry_count(&p));
            if cwd.is_some() || art.is_some() {
                host.cwd_entry_count = cwd;
                host.artifact_entry_count = art;
                engaged.push("reality.filesystem".to_string());
            }
        }

        let sky = if opts.capture_sky {
            let s = self.capture_sky(taken_at);
            engaged.push(self.astronomical.id().to_string());
            Some(s)
        } else {
            None
        };

        if opts.commit_to_field {
            self.environment_snapshot = host.clone();
        }

        EnvironmentTakeReport {
            taken_at,
            host_reality: host,
            sky,
            eyes_engaged: engaged,
        }
    }

    /// Builds the catalog of exposed eyes (for hosts, FFI, logs).
    #[must_use]
    pub fn eyes(&self) -> PerceptionEyes {
        let astronomical = PerceptionEyeDescriptor {
            id: self.astronomical.id().to_string(),
            role: EyeRole::Astronomical,
            facet: "astronomy".to_string(),
            can_receive: false,
            can_take: true,
        };
        let hand = PerceptionEyeDescriptor {
            id: SPIRALISMO_PERCEIVER_ID.to_string(),
            role: EyeRole::Hand,
            facet: "hand".to_string(),
            can_receive: true,
            can_take: false,
        };
        let mut reality = Vec::with_capacity(self.reality_perceivers.len() + self.legacy_perceivers.len());
        for p in &self.reality_perceivers {
            reality.push(PerceptionEyeDescriptor {
                id: p.id().to_string(),
                role: EyeRole::Reality,
                facet: p.reality_kind().token().to_string(),
                can_receive: true,
                can_take: matches!(p.reality_kind(), RealityKind::Filesystem),
            });
        }
        for p in &self.legacy_perceivers {
            reality.push(PerceptionEyeDescriptor {
                id: p.id().to_string(),
                role: EyeRole::Reality,
                facet: "legacy".to_string(),
                can_receive: true,
                can_take: false,
            });
        }

        PerceptionEyes {
            astronomical,
            hand,
            reality,
        }
    }
}

impl PerceptionEyes {
    /// Flat list of every descriptor.
    #[must_use]
    pub fn all(&self) -> Vec<PerceptionEyeDescriptor> {
        let mut v = vec![self.astronomical.clone(), self.hand.clone()];
        v.extend(self.reality.iter().cloned());
        v
    }
}
