use serde::{Deserialize, Deserializer, Serialize, de};
use std::fmt;

/// Version of the normalized open-top interior source description, not a
/// certification that any independently formed metric payload matches it.
pub const WORLD_SDF_OPEN_TOP_INTERIOR_SOURCE_SCHEMA_REVISION: u16 = 1;

/// Immutable five-face, axis-aligned free-space interior: positive inside
/// the X/Z rectangle and above the floor, negative outside, with no roof.
/// The source value carries geometry only; RunenSDF owns reusable field math.
#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub struct WorldSdfOpenTopRectInteriorSource {
    schema_revision: u16,
    x_bounds_meters: [f64; 2],
    floor_y_meters: f64,
    z_bounds_meters: [f64; 2],
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum WorldSdfOpenTopRectInteriorSourceError {
    UnsupportedSchemaRevision,
    NonFiniteCoordinate,
    InvalidBounds,
}

impl fmt::Display for WorldSdfOpenTopRectInteriorSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedSchemaRevision => "unsupported open-top interior source schema",
            Self::NonFiniteCoordinate => "open-top interior coordinates must be finite",
            Self::InvalidBounds => "open-top interior X/Z bounds must be strictly increasing",
        })
    }
}

impl std::error::Error for WorldSdfOpenTopRectInteriorSourceError {}

impl WorldSdfOpenTopRectInteriorSource {
    pub fn try_new(
        x_bounds_meters: [f64; 2],
        floor_y_meters: f64,
        z_bounds_meters: [f64; 2],
    ) -> Result<Self, WorldSdfOpenTopRectInteriorSourceError> {
        if !x_bounds_meters
            .iter()
            .chain(z_bounds_meters.iter())
            .chain(std::iter::once(&floor_y_meters))
            .all(|value| value.is_finite())
        {
            return Err(WorldSdfOpenTopRectInteriorSourceError::NonFiniteCoordinate);
        }
        if x_bounds_meters[0] >= x_bounds_meters[1]
            || z_bounds_meters[0] >= z_bounds_meters[1]
            || !(x_bounds_meters[1] - x_bounds_meters[0]).is_finite()
            || !(z_bounds_meters[1] - z_bounds_meters[0]).is_finite()
        {
            return Err(WorldSdfOpenTopRectInteriorSourceError::InvalidBounds);
        }
        let canonical = |value: f64| if value == 0.0 { 0.0 } else { value };
        Ok(Self {
            schema_revision: WORLD_SDF_OPEN_TOP_INTERIOR_SOURCE_SCHEMA_REVISION,
            x_bounds_meters: x_bounds_meters.map(canonical),
            floor_y_meters: canonical(floor_y_meters),
            z_bounds_meters: z_bounds_meters.map(canonical),
        })
    }

    pub const fn schema_revision(self) -> u16 {
        self.schema_revision
    }

    pub const fn x_bounds_meters(self) -> [f64; 2] {
        self.x_bounds_meters
    }

    pub const fn floor_y_meters(self) -> f64 {
        self.floor_y_meters
    }

    pub const fn z_bounds_meters(self) -> [f64; 2] {
        self.z_bounds_meters
    }
}

#[derive(Deserialize, Serialize)]
struct OpenTopInteriorWire {
    schema_revision: u16,
    x_bounds_meters: [f64; 2],
    floor_y_meters: f64,
    z_bounds_meters: [f64; 2],
}

impl<'de> Deserialize<'de> for WorldSdfOpenTopRectInteriorSource {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = OpenTopInteriorWire::deserialize(deserializer)?;
        if wire.schema_revision != WORLD_SDF_OPEN_TOP_INTERIOR_SOURCE_SCHEMA_REVISION {
            return Err(de::Error::custom(
                WorldSdfOpenTopRectInteriorSourceError::UnsupportedSchemaRevision,
            ));
        }
        Self::try_new(
            wire.x_bounds_meters,
            wire.floor_y_meters,
            wire.z_bounds_meters,
        )
        .map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_or_degenerate_source_at_construction() {
        use WorldSdfOpenTopRectInteriorSourceError::{InvalidBounds, NonFiniteCoordinate};
        assert_eq!(
            WorldSdfOpenTopRectInteriorSource::try_new([0.0, f64::INFINITY], 0.0, [0.0, 1.0]),
            Err(NonFiniteCoordinate)
        );
        assert_eq!(
            WorldSdfOpenTopRectInteriorSource::try_new([0.0, 1.0], f64::NAN, [0.0, 1.0]),
            Err(NonFiniteCoordinate)
        );
        assert_eq!(
            WorldSdfOpenTopRectInteriorSource::try_new([2.0, 1.0], 0.0, [0.0, 1.0]),
            Err(InvalidBounds)
        );
        assert_eq!(
            WorldSdfOpenTopRectInteriorSource::try_new([-1.0e308, 1.0e308], 0.0, [0.0, 1.0]),
            Err(InvalidBounds)
        );
        assert_eq!(
            WorldSdfOpenTopRectInteriorSource::try_new([0.0, 1.0], 0.0, [1.0, 1.0]),
            Err(InvalidBounds)
        );
    }

    #[test]
    fn canonical_source_roundtrips_and_rejects_invalid_wire() {
        let source = WorldSdfOpenTopRectInteriorSource::try_new([-0.0, 3.5], -0.0, [-0.0, 3.5])
            .expect("valid source");
        assert_eq!(source.x_bounds_meters()[0].to_bits(), 0.0_f64.to_bits());
        assert_eq!(source.floor_y_meters().to_bits(), 0.0_f64.to_bits());
        assert_eq!(source.z_bounds_meters()[0].to_bits(), 0.0_f64.to_bits());
        let bytes = postcard::to_allocvec(&source).expect("serialize normalized source");
        assert_eq!(
            postcard::from_bytes::<WorldSdfOpenTopRectInteriorSource>(&bytes).unwrap(),
            source
        );

        for wire in [
            OpenTopInteriorWire {
                schema_revision: 2,
                x_bounds_meters: [0.0, 3.5],
                floor_y_meters: 0.0,
                z_bounds_meters: [0.0, 3.5],
            },
            OpenTopInteriorWire {
                schema_revision: WORLD_SDF_OPEN_TOP_INTERIOR_SOURCE_SCHEMA_REVISION,
                x_bounds_meters: [0.0, f64::NAN],
                floor_y_meters: 0.0,
                z_bounds_meters: [0.0, 3.5],
            },
            OpenTopInteriorWire {
                schema_revision: WORLD_SDF_OPEN_TOP_INTERIOR_SOURCE_SCHEMA_REVISION,
                x_bounds_meters: [0.0, 3.5],
                floor_y_meters: 0.0,
                z_bounds_meters: [2.0, 2.0],
            },
        ] {
            let bytes = postcard::to_allocvec(&wire).expect("encode invalid source");
            assert!(postcard::from_bytes::<WorldSdfOpenTopRectInteriorSource>(&bytes).is_err());
        }
    }
}
