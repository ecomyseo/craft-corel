//! Display units belong to each ruler, independently of macro lengths.

use crate::app::{App, Units};
use tracedraw_core::Command;

impl Units {
    /// Identifier used by the drawing macro model, not the CDR record code.
    pub fn script_unit(self) -> u32 {
        match self {
            Self::TenthMicrons => 0,
            Self::Inches => 1,
            Self::Feet => 2,
            Self::Millimeters => 3,
            Self::Centimeters => 4,
            Self::Pixels => 5,
            Self::Miles => 6,
            Self::Meters => 7,
            Self::Kilometers => 8,
            Self::Didots => 9,
            Self::Agates => 10,
            Self::Yards => 11,
            Self::Picas => 12,
            Self::Ciceros => 13,
            Self::Points => 14,
            Self::QuarterMillimetersQ => 15,
            Self::QuarterMillimetersH => 16,
        }
    }

    pub fn from_script_unit(unit: u32) -> Option<Self> {
        Self::ALL.into_iter().find(|u| u.script_unit() == unit)
    }
}

impl App {
    /// A missing axis preference keeps the existing window unit. Unknown
    /// stored values are preserved; macro access rejects them explicitly.
    pub fn ruler_unit(&self, horizontal: bool) -> Units {
        let rulers = &self.doc().metadata.rulers;
        let stored = if horizontal {
            rulers.horizontal_unit
        } else {
            rulers.vertical_unit
        };
        stored
            .and_then(Units::from_script_unit)
            .unwrap_or(self.ruler_fallback_units.unwrap_or(self.units))
    }

    /// Change one ruler through history, leaving macro lengths unchanged.
    pub fn set_ruler_unit(&mut self, horizontal: bool, unit: Units) {
        let mut metadata = self.doc().metadata.clone();
        let axis = if horizontal {
            &mut metadata.rulers.horizontal_unit
        } else {
            &mut metadata.rulers.vertical_unit
        };
        let value = Some(unit.script_unit());
        if *axis != value {
            *axis = value;
            self.run(Command::SetMetadata { metadata });
        }
    }

    /// The common Units selector updates both display axes in one step.
    /// It does not set the independent automation unit.
    pub fn set_display_units(&mut self, unit: Units) {
        let mut metadata = self.doc().metadata.clone();
        let value = Some(unit.script_unit());
        metadata.rulers.horizontal_unit = value;
        metadata.rulers.vertical_unit = value;
        self.set_ruler_metadata(metadata, unit);
    }

    pub(crate) fn set_ruler_metadata(
        &mut self,
        metadata: tracedraw_core::document::Metadata,
        unit: Units,
    ) {
        let before = self.units;
        if unit != before {
            self.remember_ruler_fallback();
        }
        let old_step = self.engine.last_step_id();
        if metadata != self.doc().metadata {
            self.run(Command::SetMetadata { metadata });
            if before != unit && self.engine.last_step_id() != old_step {
                if let Some(step) = self.engine.last_step_id() {
                    self.display_unit_history
                        .insert((self.engine.instance(), step), (before, unit));
                }
            }
        }
        self.units = unit;
    }

    pub(crate) fn remember_ruler_fallback(&mut self) {
        if self.ruler_fallback_units.is_none()
            && (self.doc().metadata.rulers.horizontal_unit.is_none()
                || self.doc().metadata.rulers.vertical_unit.is_none())
        {
            self.ruler_fallback_units = Some(self.units);
        }
    }
}
