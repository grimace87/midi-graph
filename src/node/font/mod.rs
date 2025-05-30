use crate::{Error, Event, GraphNode, Message, Node, NoteRange};

pub struct SoundFontBuilder {
    node_id: Option<u64>,
    ranges: Vec<(NoteRange, GraphNode)>,
}

impl Default for SoundFontBuilder {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SoundFontBuilder {
    pub fn new(node_id: Option<u64>) -> Self {
        Self {
            node_id,
            ranges: vec![],
        }
    }

    pub fn add_range(mut self, range: NoteRange, consumer: GraphNode) -> Result<Self, Error> {
        self.ranges.push((range, consumer));
        Ok(self)
    }

    pub fn build(self) -> SoundFont {
        SoundFont::new(self.node_id, self.ranges)
    }
}

pub struct SoundFont {
    node_id: u64,
    ranges: Vec<(NoteRange, GraphNode)>,
}

impl SoundFont {
    fn new(node_id: Option<u64>, ranges: Vec<(NoteRange, GraphNode)>) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            ranges,
        }
    }
}

impl Node for SoundFont {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        Err(Error::User("SoundFont cannot be duplicated".to_owned()))
    }

    fn try_consume_event(&mut self, event: &Message) -> bool {
        let note = match event.data {
            Event::NoteOn { note, .. } => Some(note),
            Event::NoteOff { note, .. } => Some(note),
            _ => None,
        };
        if note.is_some() {
            let note = note.unwrap();
            for (range, consumer) in self.ranges.iter_mut() {
                if !range.contains(note) {
                    continue;
                }
                consumer.on_event(event);
            }
        } else {
            for (_, consumer) in self.ranges.iter_mut() {
                consumer.on_event(event);
            }
        }
        true
    }

    fn propagate(&mut self, event: &Message) {
        for (_, consumer) in self.ranges.iter_mut() {
            consumer.on_event(event);
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for (_, consumer) in self.ranges.iter_mut() {
            consumer.fill_buffer(buffer);
        }
    }

    fn replace_children(&mut self, _children: &[GraphNode]) -> Result<(), Error> {
        Err(Error::User(
            "SoundFont does not support replacing its children".to_owned(),
        ))
    }
}
