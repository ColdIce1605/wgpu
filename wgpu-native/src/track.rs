use {Stored, BufferId, TextureId};
use resource::{BufferUsageFlags, TextureUsageFlags};

use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;
use std::ops::BitOr;


#[derive(Clone, Debug, PartialEq)]
pub enum Tracktion<T> {
    Init,
    Keep,
    Extend { old: T },
    Replace { old: T },
}

bitflags! {
    pub struct TrackPermit: u32 {
        const EXTEND = 1;
        const REPLACE = 2;
    }
}


pub trait GenericUsage {
    fn is_exclusive(&self) -> bool;
}
impl GenericUsage for BufferUsageFlags {
    fn is_exclusive(&self) -> bool {
        BufferUsageFlags::WRITE_ALL.intersects(*self)
    }
}
impl GenericUsage for TextureUsageFlags {
    fn is_exclusive(&self) -> bool {
        TextureUsageFlags::WRITE_ALL.intersects(*self)
    }
}


pub struct Tracker<I, U> {
    map: HashMap<Stored<I>, U>,
}
pub type BufferTracker = Tracker<BufferId, BufferUsageFlags>;
pub type TextureTracker = Tracker<TextureId, TextureUsageFlags>;

impl<
    I: Hash + Eq,
    U: Copy + GenericUsage + BitOr<Output = U> + PartialEq,
> Tracker<I, U> {
    pub fn new() -> Self {
        Tracker {
            map: HashMap::new(),
        }
    }

    pub fn track(&mut self, id: I, usage: U, permit: TrackPermit) -> Result<Tracktion<U>, U> {
        match self.map.entry(Stored(id)) {
            Entry::Vacant(e) => {
                e.insert(usage);
                Ok(Tracktion::Init)
            }
            Entry::Occupied(mut e) => {
                let old = *e.get();
                if usage == old {
                    Ok(Tracktion::Keep)
                } else if permit.contains(TrackPermit::EXTEND) && !(old | usage).is_exclusive() {
                    Ok(Tracktion::Extend { old: e.insert(old | usage) })
                } else if permit.contains(TrackPermit::REPLACE) {
                    Ok(Tracktion::Replace { old: e.insert(usage) })
                } else {
                    Err(old)
                }
            }
        }
    }

    pub(crate) fn finish(self) -> HashMap<Stored<I>, U> {
        self.map
    }
}
