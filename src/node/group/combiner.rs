use crate::{Error, GraphNode, Message, Node, consts};

pub struct CombinerSource {
    node_id: u64,
    consumers: Vec<GraphNode>,
    intermediate_buffer: Vec<f32>,
}

impl CombinerSource {
    pub fn new(node_id: Option<u64>, consumers: Vec<GraphNode>) -> Self {
        Self {
            node_id: node_id.unwrap_or_else(<Self as Node>::new_node_id),
            consumers,
            intermediate_buffer: vec![0.0; consts::BUFFER_SIZE * consts::CHANNEL_COUNT],
        }
    }
}

impl Node for CombinerSource {
    fn get_node_id(&self) -> u64 {
        self.node_id
    }

    fn set_node_id(&mut self, node_id: u64) {
        self.node_id = node_id;
    }

    fn duplicate(&self) -> Result<GraphNode, Error> {
        let consumers: Result<Vec<GraphNode>, Error> =
            self.consumers.iter().map(|c| c.duplicate()).collect();
        let combiner = Self::new(Some(self.node_id), consumers?);
        Ok(Box::new(combiner))
    }

    fn try_consume_event(&mut self, _event: &Message) -> bool {
        false
    }

    fn propagate(&mut self, event: &Message) {
        for consumer in self.consumers.iter_mut() {
            consumer.on_event(event);
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        let buffer_size = buffer.len();
        let sample_count = buffer_size / consts::CHANNEL_COUNT;
        let intermediate_slice = &mut self.intermediate_buffer[0..buffer_size];
        for consumer in self.consumers.iter_mut() {
            intermediate_slice.fill(0.0);
            consumer.fill_buffer(intermediate_slice);
            for i in 0..sample_count {
                let index = i * 2;
                buffer[index] += intermediate_slice[index];
                buffer[index + 1] += intermediate_slice[index + 1];
            }
        }
    }

    fn replace_children(&mut self, children: &[GraphNode]) -> Result<(), Error> {
        self.consumers = children
            .iter()
            .map(|child| child.duplicate())
            .collect::<Result<Vec<GraphNode>, Error>>()?;
        Ok(())
    }
}
