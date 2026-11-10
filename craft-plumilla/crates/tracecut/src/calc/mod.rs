//! Calculations: cutting time and cost.
//!
//! Pure geometry over page-space outlines in millimetres, y up, read by
//! TraceCut's Measure tools. Nothing here changes a drawing: the
//! calculations only read outlines.
//!
//! - [`job`] measures the objects once ([`Job`]): length and areas from
//!   [`crate::measure`], one block (bounding box) per object and the
//!   travel of the cutting head between outlines.
//! - [`time`]: cutting time from a [`time::TimeProfile`].
//! - [`cost`]: material and services from editable price lists.
//! - [`csv`]: the CSV text the price lists are imported from and
//!   exported to.

pub mod cost;
pub mod csv;
pub mod time;

use crate::measure::{self, Measurement};
use time::{Travel, TravelOrder};
use tracedraw_core::geometry::{BezPath, Point, Rect};

/// What the calculations need to know about the objects, measured once.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Job {
    /// Length and areas of all objects together.
    pub measurement: Measurement,
    /// The bounding box of each object with at least one segment, in the
    /// order given.
    pub blocks: Vec<Rect>,
    /// The head's moves between outlines.
    pub travel: Travel,
    /// Each move with the tool up (from, to), in order.
    pub moves: Vec<(Point, Point)>,
}

/// Measure `objects` (each given by its outlines, see
/// [`crate::outlines`]) for the calculations. The head starts at `start`
/// and visits the outlines in `order`.
pub fn job(objects: &[Vec<BezPath>], start: Point, order: TravelOrder) -> Job {
    let mut m = Measurement::default();
    let mut blocks = Vec::new();
    for o in objects {
        let one = measure::measure_object(o);
        // An object without segments (an empty path, a lone node) has
        // nothing to bill.
        if let (Some(b), true) = (one.bbox, one.closed_paths + one.open_paths > 0) {
            if finite_rect(b) {
                blocks.push(b);
            }
        }
        m.add(&one);
    }
    let paths: Vec<BezPath> = objects.iter().flatten().cloned().collect();
    let (travel, moves) = time::travel_moves(&paths, start, order);
    Job {
        measurement: m,
        blocks,
        travel,
        moves,
    }
}

pub(crate) fn finite_rect(r: Rect) -> bool {
    r.x0.is_finite() && r.y0.is_finite() && r.x1.is_finite() && r.y1.is_finite()
}

/// `v` when it is finite and not below zero, else zero.
pub(crate) fn non_negative(v: f64) -> f64 {
    if v.is_finite() && v > 0.0 {
        v
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracedraw_core::geometry::Shape as _;

    #[test]
    fn a_job_has_one_block_per_object_and_the_measured_length() {
        let a = vec![Rect::new(0.0, 0.0, 10.0, 10.0).to_path(0.01)];
        let b = vec![Rect::new(20.0, 0.0, 30.0, 5.0).to_path(0.01)];
        let j = job(&[a, b], Point::ZERO, TravelOrder::AsDrawn);
        assert_eq!(j.blocks.len(), 2);
        assert!((j.measurement.length - 70.0).abs() < 1e-9);
        assert!((j.measurement.area - 150.0).abs() < 1e-9);
        assert_eq!(j.travel.outlines, 2);
        // From the origin to (0,0), then from (0,0) to (20,0).
        assert!((j.travel.travel_length - 20.0).abs() < 1e-9);
        assert_eq!(j.moves.len(), 2);
    }

    #[test]
    fn an_empty_job_is_all_zero() {
        let j = job(&[], Point::ZERO, TravelOrder::Nearest);
        assert_eq!(j, Job::default());
        let j = job(&[vec![BezPath::new()]], Point::ZERO, TravelOrder::Nearest);
        assert!(j.blocks.is_empty());
        assert_eq!(j.travel.outlines, 0);
    }

    #[test]
    fn non_finite_outlines_do_not_panic() {
        let mut p = BezPath::new();
        p.move_to((f64::NAN, 0.0));
        p.line_to((f64::INFINITY, 1.0));
        p.line_to((2.0, 2.0));
        let j = job(&[vec![p]], Point::ZERO, TravelOrder::Nearest);
        assert!(j.travel.travel_length.is_finite());
        assert!(j.blocks.iter().all(|b| finite_rect(*b)));
    }
}
