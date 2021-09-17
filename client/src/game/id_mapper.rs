use ahash::AHashMap;

/// Maps netwowrk IDs for entities (players, cities, or units)
/// to slotmap IDs used internally.
#[derive(Debug)]
pub struct IdMapper<Id> {
    mapping: AHashMap<u32, Id>,
}

impl<Id> IdMapper<Id>
where
    Id: Copy,
{
    pub fn new() -> Self {
        Self {
            mapping: AHashMap::new(),
        }
    }

    pub fn insert(&mut self, network_id: u32, id: Id) {
        self.mapping.insert(network_id, id);
    }

    pub fn remove(&mut self, network_id: u32) {
        self.mapping.remove(&network_id);
    }

    pub fn get(&self, network_id: u32) -> Option<Id> {
        self.mapping.get(&network_id).copied()
    }
}
