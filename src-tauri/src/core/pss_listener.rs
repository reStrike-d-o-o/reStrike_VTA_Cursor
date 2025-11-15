use std::sync::Arc;
use std::time::SystemTime;

use std::sync::OnceLock;
use tokio::sync::broadcast;

use crate::pss::protocol::PssEvent;

/// Global registry for the shared PSS listener instance
static GLOBAL_PSS_LISTENER: OnceLock<Arc<PssListener>> = OnceLock::new();

/// Envelope that carries both the raw parsed UDP event and the JSON payload
#[derive(Clone, Debug)]
pub struct PssEventEnvelope {
    pub raw: PssEvent,
    pub json: serde_json::Value,
    pub received_at: SystemTime,
}

/// Central dispatcher for all PSS events.
///
/// * Guarantees a single fan-out pipeline for UDP traffic.
/// * Provides separate broadcast channels for JSON consumers (existing overlays/React app)
///   and rich envelopes (future OBS automation, analytics, etc.).
pub struct PssListener {
    json_sender: broadcast::Sender<serde_json::Value>,
    envelope_sender: broadcast::Sender<PssEventEnvelope>,
}

impl PssListener {
    #[must_use]
    pub fn new(buffer: usize) -> Self {
        let (json_sender, _) = broadcast::channel(buffer);
        let (envelope_sender, _) = broadcast::channel(buffer);

        Self {
            json_sender,
            envelope_sender,
        }
    }

    pub fn set_global(instance: Arc<Self>) {
        if GLOBAL_PSS_LISTENER.set(instance).is_err() {
            log::warn!("Global PSS listener already initialised; skipping override");
        }
    }

    pub fn global() -> Option<&'static Arc<Self>> {
        GLOBAL_PSS_LISTENER.get()
    }

    pub fn broadcast(&self, raw: Option<PssEvent>, event_json: serde_json::Value) {
        if let Err(err) = self.json_sender.send(event_json.clone()) {
            log::debug!("No JSON subscribers to receive PSS event: {err}");
        }

        if let Some(raw_event) = raw {
            let envelope = PssEventEnvelope {
                raw: raw_event,
                json: event_json,
                received_at: SystemTime::now(),
            };

            if let Err(err) = self.envelope_sender.send(envelope) {
                log::debug!("No envelope subscribers to receive PSS event: {err}");
            }
        }
    }

    pub fn subscribe_json(&self) -> broadcast::Receiver<serde_json::Value> {
        self.json_sender.subscribe()
    }

    pub fn subscribe_envelope(&self) -> broadcast::Receiver<PssEventEnvelope> {
        self.envelope_sender.subscribe()
    }

    pub fn subscribe_json_global() -> Option<broadcast::Receiver<serde_json::Value>> {
        Self::global().map(|listener| listener.subscribe_json())
    }

    pub fn subscribe_envelope_global() -> Option<broadcast::Receiver<PssEventEnvelope>> {
        Self::global().map(|listener| listener.subscribe_envelope())
    }

    pub fn broadcast_global(raw: Option<PssEvent>, event_json: serde_json::Value) {
        if let Some(listener) = Self::global() {
            listener.broadcast(raw, event_json);
        } else {
            log::warn!("PSS listener not initialised; dropping event");
        }
    }

    /// Encode a `PssEvent` back into its wire representation for legacy consumers.
    #[must_use]
    pub fn encode_event(event: &PssEvent) -> Option<String> {
        match event {
            PssEvent::Points {
                athlete,
                point_type,
            } => Some(format!("pt{};{};", athlete, point_type)),
            PssEvent::HitLevel { athlete, level } => Some(format!("hl{};{};", athlete, level)),
            PssEvent::Warnings {
                athlete1_warnings,
                athlete2_warnings,
            } => Some(format!(
                "wg1;{};wg2;{};",
                athlete1_warnings, athlete2_warnings
            )),
            PssEvent::Injury {
                athlete,
                time,
                action,
            } => {
                let code = match athlete {
                    0 => "ij0",
                    1 => "ij1",
                    2 => "ij2",
                    _ => "ij0",
                };
                let mut segments = vec![code.to_string(), time.clone()];
                if let Some(action) = action.as_ref() {
                    segments.push(action.clone());
                }
                Some(format!("{};", segments.join(";")))
            }
            PssEvent::Challenge {
                source,
                accepted,
                won,
                ..
            } => {
                let code = match source {
                    0 => "ch0",
                    1 => "ch1",
                    2 => "ch2",
                    _ => "ch0",
                };
                let mut segments = vec![code.to_string()];
                if let Some(flag) = accepted {
                    segments.push(if *flag { "1" } else { "0" }.to_string());
                    if let Some(won_flag) = won {
                        segments.push(if *won_flag { "1" } else { "0" }.to_string());
                    }
                }
                Some(format!("{};", segments.join(";")))
            }
            PssEvent::Break { time, action } => {
                let mut segments = vec!["brk".to_string(), time.clone()];
                if let Some(action) = action.as_ref() {
                    segments.push(action.clone());
                }
                Some(format!("{};", segments.join(";")))
            }
            PssEvent::WinnerRounds {
                round1_winner,
                round2_winner,
                round3_winner,
            } => Some(format!(
                "wrd;rd1;{};rd2;{};rd3;{};",
                round1_winner, round2_winner, round3_winner
            )),
            PssEvent::Winner {
                name,
                classification,
            } => {
                if let Some(classification) = classification {
                    Some(format!("wmh;{};{};", name, classification))
                } else {
                    Some(format!("win;{};", name))
                }
            }
            PssEvent::Athletes {
                athlete1_short,
                athlete1_long,
                athlete1_country,
                athlete2_short,
                athlete2_long,
                athlete2_country,
            } => Some(format!(
                "at1;{};{};{};at2;{};{};{};",
                athlete1_short,
                athlete1_long,
                athlete1_country,
                athlete2_short,
                athlete2_long,
                athlete2_country
            )),
            PssEvent::MatchConfig {
                number,
                category,
                weight,
                rounds,
                colors,
                match_id,
                division,
                total_rounds,
                round_duration,
                countdown_type,
                count_up,
                format,
            } => Some(format!(
                "mch;{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};",
                number,
                category,
                weight,
                rounds,
                colors.0,
                colors.1,
                colors.2,
                colors.3,
                match_id,
                division,
                total_rounds,
                round_duration,
                countdown_type,
                count_up,
                format
            )),
            PssEvent::Scores {
                athlete1_r1,
                athlete2_r1,
                athlete1_r2,
                athlete2_r2,
                athlete1_r3,
                athlete2_r3,
            } => Some(format!(
                "s11;{};s21;{};s12;{};s22;{};s13;{};s23;{};",
                athlete1_r1, athlete2_r1, athlete1_r2, athlete2_r2, athlete1_r3, athlete2_r3
            )),
            PssEvent::CurrentScores {
                athlete1_score,
                athlete2_score,
            } => Some(format!("sc1;{};sc2;{};", athlete1_score, athlete2_score)),
            PssEvent::Clock { time, action } => {
                let mut segments = vec!["clk".to_string(), time.clone()];
                if let Some(action) = action {
                    segments.push(action.clone());
                }
                Some(format!("{};", segments.join(";")))
            }
            PssEvent::Round { current_round } => Some(format!("rnd;{};", current_round)),
            PssEvent::FightLoaded => Some("pre;FightLoaded;".to_string()),
            PssEvent::FightReady => Some("rdy;FightReady;".to_string()),
            PssEvent::Supremacy { value } => Some(format!("sup;{};", value)),
            PssEvent::Raw(message) => Some(message.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast::error::TryRecvError;

    #[test]
    fn broadcast_sends_json_and_envelope() {
        let listener = PssListener::new(4);
        let mut json_rx = listener.subscribe_json();
        let mut envelope_rx = listener.subscribe_envelope();

        let payload = serde_json::json!({ "type": "fight_loaded" });
        let raw_event = PssEvent::FightLoaded;

        listener.broadcast(Some(raw_event.clone()), payload);

        let json = json_rx.try_recv().expect("json receiver should get event");
        assert_eq!(
            json.get("type"),
            Some(&serde_json::Value::from("fight_loaded"))
        );

        let envelope = envelope_rx
            .try_recv()
            .expect("envelope receiver should get event");
        assert!(matches!(envelope.raw, PssEvent::FightLoaded));
        assert_eq!(
            envelope.json.get("type"),
            Some(&serde_json::Value::from("fight_loaded"))
        );
    }

    #[test]
    fn broadcast_without_raw_only_notifies_json() {
        let listener = PssListener::new(4);
        let mut json_rx = listener.subscribe_json();
        let mut envelope_rx = listener.subscribe_envelope();

        listener.broadcast(None, serde_json::json!({ "type": "clock" }));

        assert!(json_rx.try_recv().is_ok());
        assert!(matches!(envelope_rx.try_recv(), Err(TryRecvError::Empty)));
    }
}
