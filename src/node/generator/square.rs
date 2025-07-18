use crate::{
    AssetLoader, Balance, Error, Event, EventTarget, GraphNode, Message, Node,
    abstraction::{NodeConfig, NodeConfigData, defaults},
    consts, util,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SquareWave {
    #[serde(default = "defaults::none_id")]
    pub node_id: Option<u64>,
    #[serde(default = "defaults::source_balance")]
    pub balance: Balance,
    #[serde(default = "defaults::amplitude")]
    pub amplitude: f32,
    #[serde(default = "defaults::duty_cycle")]
    pub duty_cycle: f32,
}

impl SquareWave {
    pub fn stock() -> NodeConfigData {
        NodeConfigData(Box::new(Self {
            node_id: defaults::none_id(),
            balance: Balance::Both,
            amplitude: defaults::amplitude(),
            duty_cycle: defaults::duty_cycle(),
        }))
    }
}

impl NodeConfig for SquareWave {
    fn to_node(&self, _asset_loader: &dyn AssetLoader) -> Result<GraphNode, Error> {
        Ok(Box::new(SquareWaveNode::new(
            self.node_id,
            self.balance,
            self.amplitude,
            self.duty_cycle,
        )))
    }

    fn clone_child_configs(&self) -> Option<Vec<crate::abstraction::NodeConfigData>> {
        None
    }

    fn asset_source(&self) -> Option<&str> {
        None
    }

    fn duplicate(&self) -> Box<dyn NodeConfig + Send + Sync + 'static> {
        Box::new(self.clone())
    }
}

pub struct SquareWaveNode {
    node_id: u64,
    is_on: bool,
    current_note: u8,
    current_frequency: f32,
    balance: Balance,
    cycle_progress_samples: f32,
    period_samples_a440: f32,
    peak_amplitude: f32,
    note_velocity: f32,
    modulated_volume: f32,
    duty_cycle: f32,
}

impl SquareWaveNode {
    pub fn new(node_id: Option<u64>, balance: Balance, amplitude: f32, duty_cycle: f32) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            is_on: false,
            current_note: 0,
            current_frequency: 10.0,
            balance,
            cycle_progress_samples: 0.0,
            period_samples_a440: consts::PLAYBACK_SAMPLE_RATE as f32 / 440.0,
            peak_amplitude: amplitude,
            note_velocity: 1.0,
            modulated_volume: 1.0,
            duty_cycle,
        }
    }
}

impl Node for SquareWaveNode {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let source = Self::new(
            Some(self.node_id),
            self.balance,
            self.peak_amplitude,
            self.duty_cycle,
        );
        Ok(Box::new(source))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        match event.data {
            Event::NoteOff { note, .. } => {
                if note == self.current_note || event.target == EventTarget::Broadcast {
                    self.is_on = false;
                }
            }
            Event::NoteOn { note, vel } => {
                self.is_on = true;
                self.current_note = note;
                self.current_frequency = util::frequency_of(note);
                self.note_velocity = vel;
            }
            Event::PitchMultiplier(multiplier) => {
                self.current_frequency = multiplier * util::frequency_of(self.current_note);
            }
            Event::SourceBalance(balance) => {
                self.balance = balance;
            }
            Event::Volume(volume) => {
                self.modulated_volume = volume;
            }
            _ => {}
        }
        true
    }

    fn propagate(&mut self, _event: &Message) {}

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        if !self.is_on {
            return;
        }
        let size = buffer.len();
        let pitch_period_samples = consts::PLAYBACK_SAMPLE_RATE as f32 / self.current_frequency;
        let mut stretched_progress =
            self.cycle_progress_samples * pitch_period_samples / self.period_samples_a440;

        #[cfg(debug_assertions)]
        assert_eq!(size % consts::CHANNEL_COUNT, 0);

        // Currently only-supported channel configuration
        #[cfg(debug_assertions)]
        assert_eq!(consts::CHANNEL_COUNT, 2);

        let current_amplitude = self.peak_amplitude * self.note_velocity * self.modulated_volume;
        let (left_amplitude, right_amplitude) = match self.balance {
            Balance::Both => (1.0, 1.0),
            Balance::Left => (1.0, 0.0),
            Balance::Right => (0.0, 1.0),
            Balance::Pan(pan) => (1.0 - pan, pan),
        };
        for i in (0..size).step_by(consts::CHANNEL_COUNT) {
            stretched_progress += 1.0;
            if stretched_progress >= pitch_period_samples {
                stretched_progress -= pitch_period_samples;
            }
            let duty = stretched_progress / pitch_period_samples;
            let amplitude = match duty > self.duty_cycle {
                true => current_amplitude,
                false => -current_amplitude,
            };
            buffer[i] += left_amplitude * amplitude;
            buffer[i + 1] += right_amplitude * amplitude;
        }

        self.cycle_progress_samples =
            stretched_progress * self.period_samples_a440 / pitch_period_samples;
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        match children.is_empty() {
            true => Ok(()),
            false => Err(Error::User(
                "SquareWaveSource cannot have children".to_owned(),
            )),
        }
    }
}
