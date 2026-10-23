//! Dense typed ids, assigned at content load in sorted string-id order so the same files
//! always produce the same numbers.
use serde::{Deserialize, Serialize};

macro_rules! id {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(
            Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub u16);

        impl $name {
            #[must_use]
            pub fn index(self) -> usize {
                usize::from(self.0)
            }
        }
    };
}

id!(/// A villager, by load order.
    VillagerId);
id!(/// A place: an exterior zone or one floor of a building.
    PlaceId);
id!(/// A door record, both directions.
    DoorId);
id!(/// A smart object (bed, counter, oven).
    ObjectId);
