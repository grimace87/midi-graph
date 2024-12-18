use crate::{BufferConsumer, BufferConsumerNode, Error, Node, NodeEvent, NoteEvent, NoteRange};

pub struct RangeData {
    node_id: u64,
    pub range: NoteRange,
    pub next_on_index: usize,
    pub consumers: Vec<Box<dyn BufferConsumerNode + Send + 'static>>,
}

impl RangeData {
    pub fn new(
        range: NoteRange,
        consumers: Vec<Box<dyn BufferConsumerNode + Send + 'static>>,
    ) -> Self {
        Self {
            node_id: <Self as Node>::new_node_id(),
            range,
            next_on_index: 0,
            consumers,
        }
    }

    fn turn_note_on(&mut self, note: u8, vel: f32) {
        if !self.range.contains(note) {
            return;
        }
        let event = NodeEvent::Note {
            note,
            event: NoteEvent::NoteOn { vel },
        };
        self.consumers[self.next_on_index].on_event(&event);
        self.next_on_index = (self.next_on_index + 1) % self.consumers.len();
    }

    fn turn_note_off(&mut self, note: u8, vel: f32) {
        if !self.range.contains(note) {
            return;
        }
        let event = NodeEvent::Note {
            note,
            event: NoteEvent::NoteOff { vel },
        };
        for consumer in self.consumers.iter_mut() {
            consumer.on_event(&event);
        }
    }
}

impl BufferConsumerNode for RangeData {}

impl Node for RangeData {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn on_event(&mut self, event: &NodeEvent) {
        match event {
            NodeEvent::Broadcast(_) => {
                for source in self.consumers.iter_mut() {
                    source.on_event(event);
                }
            }
            NodeEvent::Note { note, event } => match event {
                NoteEvent::NoteOn { vel } => self.turn_note_on(*note, *vel),
                NoteEvent::NoteOff { vel } => self.turn_note_off(*note, *vel),
            },
            NodeEvent::NodeControl { .. } => {
                for consumer in self.consumers.iter_mut() {
                    consumer.on_event(event);
                }
            }
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for consumer in self.consumers.iter_mut() {
            consumer.fill_buffer(buffer);
        }
    }
}

impl BufferConsumer for RangeData {
    fn duplicate(&self) -> Result<Box<dyn BufferConsumerNode + Send + 'static>, Error> {
        let mut consumers = vec![];
        for consumer in self.consumers.iter() {
            consumers.push(consumer.duplicate()?);
        }
        let source = Self {
            node_id: self.node_id,
            range: self.range.clone(),
            next_on_index: 0,
            consumers,
        };
        Ok(Box::new(source))
    }
}
