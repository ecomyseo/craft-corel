//! Cost: the material a job takes (by area or by the length of roll it
//! uses) and the services billed on it, from editable price lists.
//!
//! Every object is a block: its bounding box grown on every side by the
//! technical reserve, and at least the minimum billed width and height.
//! Prices carry no currency; the application shows the one the user sets.

use super::csv::{self, Imported};
use super::{finite_rect, non_negative};
use tracedraw_core::geometry::Rect;

/// How a material is sold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaterialUnit {
    /// By the square metre of the blocks.
    #[default]
    Area,
    /// By the running metre of a roll of a given width.
    Roll,
}

/// A row of the material price list.
#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: String,
    pub unit: MaterialUnit,
    /// Price per square metre ([`MaterialUnit::Area`]) or per running
    /// metre of roll ([`MaterialUnit::Roll`]).
    pub price: f64,
    /// Width of the roll (mm); only used by [`MaterialUnit::Roll`].
    pub roll_width: f64,
}

/// What a service is billed by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServiceUnit {
    /// Per metre of outline cut.
    #[default]
    Metre,
    /// Per square metre of the blocks.
    SquareMetre,
    /// Per object.
    Piece,
    /// Per hour of cutting time.
    Hour,
}

impl ServiceUnit {
    pub const ALL: [ServiceUnit; 4] = [
        ServiceUnit::Metre,
        ServiceUnit::SquareMetre,
        ServiceUnit::Piece,
        ServiceUnit::Hour,
    ];
}

/// A row of the service price list.
#[derive(Debug, Clone, PartialEq)]
pub struct Service {
    pub name: String,
    pub unit: ServiceUnit,
    pub price: f64,
    /// Billed on the job (a choice of the job, not part of the CSV).
    pub selected: bool,
}

/// The example price lists a new installation starts with.
pub fn default_materials() -> Vec<Material> {
    let m = |name: &str, unit, price, roll_width| Material {
        name: name.into(),
        unit,
        price,
        roll_width,
    };
    vec![
        m("Self-adhesive vinyl 630 mm", MaterialUnit::Roll, 6.0, 630.0),
        m(
            "Self-adhesive vinyl 1260 mm",
            MaterialUnit::Roll,
            11.0,
            1260.0,
        ),
        m("Reflective film 1220 mm", MaterialUnit::Roll, 28.0, 1220.0),
        m("PVC foam board 3 mm", MaterialUnit::Area, 18.0, 0.0),
        m("Acrylic 3 mm", MaterialUnit::Area, 38.0, 0.0),
        m("Aluminium composite 3 mm", MaterialUnit::Area, 30.0, 0.0),
    ]
}

/// See [`default_materials`].
pub fn default_services() -> Vec<Service> {
    let s = |name: &str, unit, price, selected| Service {
        name: name.into(),
        unit,
        price,
        selected,
    };
    vec![
        s("Cutting", ServiceUnit::Metre, 0.5, true),
        s("Weeding", ServiceUnit::SquareMetre, 12.0, true),
        s("Application tape", ServiceUnit::SquareMetre, 3.0, false),
        s("Handling", ServiceUnit::Piece, 0.5, false),
        s("Machine time", ServiceUnit::Hour, 40.0, false),
    ]
}

/// The job settings of the cost.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CostSpec {
    /// Added on every side of each block (mm).
    pub reserve: f64,
    /// Smallest billed block (mm).
    pub min_width: f64,
    pub min_height: f64,
    /// The job is made this many times.
    pub copies: u32,
}

impl Default for CostSpec {
    fn default() -> Self {
        CostSpec {
            reserve: 5.0,
            min_width: 0.0,
            min_height: 0.0,
            copies: 1,
        }
    }
}

/// Most copies accepted.
pub const MAX_COPIES: u32 = 100_000;

/// What a job gives the price list to bill.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Quantities {
    /// The bounding box of each object (mm).
    pub blocks: Vec<Rect>,
    /// Outline length cut (mm, one pass).
    pub cut_length: f64,
    /// Cutting time (hours).
    pub hours: f64,
}

/// The unit of a cost line's quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QtyUnit {
    SquareMetre,
    Metre,
    Piece,
    Hour,
}

/// One line of the cost table.
#[derive(Debug, Clone, PartialEq)]
pub struct CostLine {
    pub name: String,
    /// The material's line (else a service).
    pub material: bool,
    pub quantity: f64,
    pub unit: QtyUnit,
    pub unit_price: f64,
    pub amount: f64,
}

/// How much of a roll a job takes.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RollUse {
    /// Roll length for one copy (mm).
    pub length: f64,
    /// The job lies across the roll (it fits only turned 90 degrees).
    pub turned: bool,
    /// Strips of roll side by side when the job is wider than the roll
    /// either way (1 when it fits).
    pub lanes: u32,
}

/// The cost table.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CostReport {
    pub lines: Vec<CostLine>,
    pub total: f64,
    /// Area of the blocks with reserve and minimum size, one copy (mm2).
    pub billed_area: f64,
    /// Set when the material is sold by the roll.
    pub roll: Option<RollUse>,
}

impl CostSpec {
    /// The spec with every value in range (not-a-number: zero).
    pub fn sanitized(&self) -> CostSpec {
        CostSpec {
            reserve: non_negative(self.reserve),
            min_width: non_negative(self.min_width),
            min_height: non_negative(self.min_height),
            copies: self.copies.clamp(1, MAX_COPIES),
        }
    }
}

/// Width and height of each billed block (mm): the box grown by the
/// reserve on every side, at least the minimum size. Boxes that are not
/// finite are left out.
pub fn billed_blocks(blocks: &[Rect], spec: &CostSpec) -> Vec<(f64, f64)> {
    let s = spec.sanitized();
    blocks
        .iter()
        .filter(|b| finite_rect(**b))
        .map(|b| {
            (
                (b.width().abs() + 2.0 * s.reserve).max(s.min_width),
                (b.height().abs() + 2.0 * s.reserve).max(s.min_height),
            )
        })
        .collect()
}

/// The roll a job takes: the bounding box of all blocks with the reserve
/// (at least the minimum size) laid along a roll `roll_width` mm wide.
/// It runs along the roll when its width fits, else across when its
/// height fits, else in lanes of the roll's width side by side, each as
/// long as the job's height. A roll width that is not a positive number
/// fits anything.
pub fn roll_use(blocks: &[Rect], roll_width: f64, spec: &CostSpec) -> RollUse {
    let s = spec.sanitized();
    let Some(b) = blocks
        .iter()
        .copied()
        .filter(|b| finite_rect(*b))
        .reduce(|a, b| a.union(b))
    else {
        return RollUse {
            lanes: 1,
            ..Default::default()
        };
    };
    let w = (b.width().abs() + 2.0 * s.reserve).max(s.min_width);
    let h = (b.height().abs() + 2.0 * s.reserve).max(s.min_height);
    let roll = non_negative(roll_width);
    if roll <= 0.0 || w <= roll + 1e-9 {
        return RollUse {
            length: h,
            turned: false,
            lanes: 1,
        };
    }
    if h <= roll + 1e-9 {
        return RollUse {
            length: w,
            turned: true,
            lanes: 1,
        };
    }
    // Lanes across the narrower side, so the fewest are needed.
    let (across, along) = if w <= h { (w, h) } else { (h, w) };
    let lanes = (across / roll).ceil().clamp(1.0, f64::from(u32::MAX)) as u32;
    RollUse {
        length: f64::from(lanes) * along,
        turned: w > h,
        lanes,
    }
}

/// The cost of a job: the material (`None`: no material line) and every
/// selected service, all times the copies.
///
/// - material by area: billed area (m2) x price
/// - material by roll: roll length (m) x price
/// - service per metre: cut length (m) x price
/// - per square metre: billed area (m2) x price
/// - per piece: blocks x price
/// - per hour: cutting time (h) x price
pub fn cost(
    q: &Quantities,
    material: Option<&Material>,
    services: &[Service],
    spec: &CostSpec,
) -> CostReport {
    let s = spec.sanitized();
    let copies = f64::from(s.copies);
    let sizes = billed_blocks(&q.blocks, &s);
    let billed_area: f64 = sizes.iter().map(|(w, h)| w * h).sum();
    let area_m2 = billed_area / 1_000_000.0 * copies;
    let mut report = CostReport {
        billed_area,
        ..Default::default()
    };
    let mut push = |name: &str, material: bool, quantity: f64, unit: QtyUnit, price: f64| {
        let unit_price = non_negative(price);
        let quantity = non_negative(quantity);
        let amount = quantity * unit_price;
        report.total += amount;
        report.lines.push(CostLine {
            name: name.to_string(),
            material,
            quantity,
            unit,
            unit_price,
            amount,
        });
    };
    let mut roll = None;
    if let Some(m) = material {
        match m.unit {
            MaterialUnit::Area => push(&m.name, true, area_m2, QtyUnit::SquareMetre, m.price),
            MaterialUnit::Roll => {
                let r = roll_use(&q.blocks, m.roll_width, &s);
                roll = Some(r);
                push(
                    &m.name,
                    true,
                    r.length / 1000.0 * copies,
                    QtyUnit::Metre,
                    m.price,
                );
            }
        }
    }
    for sv in services.iter().filter(|sv| sv.selected) {
        let (qty, unit) = match sv.unit {
            ServiceUnit::Metre => (non_negative(q.cut_length) / 1000.0 * copies, QtyUnit::Metre),
            ServiceUnit::SquareMetre => (area_m2, QtyUnit::SquareMetre),
            ServiceUnit::Piece => (sizes.len() as f64 * copies, QtyUnit::Piece),
            ServiceUnit::Hour => (non_negative(q.hours) * copies, QtyUnit::Hour),
        };
        push(&sv.name, false, qty, unit, sv.price);
    }
    report.roll = roll;
    report
}

impl MaterialUnit {
    /// The unit's name in CSV and settings files: `m2` or `roll`.
    pub fn id(self) -> &'static str {
        match self {
            MaterialUnit::Area => "m2",
            MaterialUnit::Roll => "roll",
        }
    }

    /// The unit for a name (see [`MaterialUnit::id`]; `m²`, `sqm`,
    /// `area`, `m`, `lm` and `running` are understood too).
    pub fn from_id(s: &str) -> Option<MaterialUnit> {
        match s.trim().to_lowercase().as_str() {
            "m2" | "m\u{b2}" | "sqm" | "area" => Some(MaterialUnit::Area),
            "roll" | "m" | "lm" | "running" => Some(MaterialUnit::Roll),
            _ => None,
        }
    }
}

impl ServiceUnit {
    /// The unit's name in CSV and settings files: `m`, `m2`, `piece` or
    /// `hour`.
    pub fn id(self) -> &'static str {
        match self {
            ServiceUnit::Metre => "m",
            ServiceUnit::SquareMetre => "m2",
            ServiceUnit::Piece => "piece",
            ServiceUnit::Hour => "hour",
        }
    }

    /// The unit for a name (see [`ServiceUnit::id`]; `metre`, `meter`,
    /// `lm`, `m²`, `sqm`, `pc`, `pcs`, `each`, `h` and `hr` are
    /// understood too).
    pub fn from_id(s: &str) -> Option<ServiceUnit> {
        match s.trim().to_lowercase().as_str() {
            "m" | "metre" | "meter" | "lm" => Some(ServiceUnit::Metre),
            "m2" | "m\u{b2}" | "sqm" => Some(ServiceUnit::SquareMetre),
            "piece" | "pc" | "pcs" | "each" => Some(ServiceUnit::Piece),
            "hour" | "h" | "hr" => Some(ServiceUnit::Hour),
            _ => None,
        }
    }
}

/// A price: a number not below zero.
fn read_price(s: &str) -> Option<f64> {
    csv::parse_number(s).filter(|v| *v >= 0.0)
}

/// The material list as CSV: `name,unit,price,roll_width_mm` with a
/// header row; unit is `m2` or `roll`.
pub fn materials_to_csv(list: &[Material]) -> String {
    let mut rows = vec![vec![
        "name".to_string(),
        "unit".into(),
        "price".into(),
        "roll_width_mm".into(),
    ]];
    for m in list {
        rows.push(vec![
            m.name.clone(),
            m.unit.id().into(),
            csv::number(m.price),
            csv::number(m.roll_width),
        ]);
    }
    csv::write(&rows)
}

/// Read a material list (see [`materials_to_csv`]). A row needs a name,
/// a unit and a price not below zero; a roll also a width above zero.
pub fn materials_from_csv(text: &str) -> Imported<Material> {
    csv::read_rows(text, |r| {
        let name = r.first()?.trim();
        if name.is_empty() {
            return None;
        }
        let unit = MaterialUnit::from_id(r.get(1)?)?;
        let price = read_price(r.get(2)?)?;
        let roll_width = r.get(3).and_then(|f| csv::parse_number(f));
        let roll_width = match unit {
            MaterialUnit::Roll => roll_width.filter(|w| *w > 0.0)?,
            MaterialUnit::Area => roll_width.map_or(0.0, non_negative),
        };
        Some(Material {
            name: name.to_string(),
            unit,
            price,
            roll_width,
        })
    })
}

/// The service list as CSV: `name,unit,price` with a header row; unit is
/// `m`, `m2`, `piece` or `hour`.
pub fn services_to_csv(list: &[Service]) -> String {
    let mut rows = vec![vec!["name".to_string(), "unit".into(), "price".into()]];
    for s in list {
        rows.push(vec![
            s.name.clone(),
            s.unit.id().into(),
            csv::number(s.price),
        ]);
    }
    csv::write(&rows)
}

/// Read a service list (see [`services_to_csv`]). Read services are not
/// selected.
pub fn services_from_csv(text: &str) -> Imported<Service> {
    csv::read_rows(text, |r| {
        let name = r.first()?.trim();
        if name.is_empty() {
            return None;
        }
        Some(Service {
            name: name.to_string(),
            unit: ServiceUnit::from_id(r.get(1)?)?,
            price: read_price(r.get(2)?)?,
            selected: false,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, w: f64, h: f64) -> Rect {
        Rect::new(x, y, x + w, y + h)
    }

    fn spec(reserve: f64) -> CostSpec {
        CostSpec {
            reserve,
            ..Default::default()
        }
    }

    fn by_area(price: f64) -> Material {
        Material {
            name: "Board".into(),
            unit: MaterialUnit::Area,
            price,
            roll_width: 0.0,
        }
    }

    #[test]
    fn blocks_grow_by_the_reserve_and_the_minimum() {
        let s = CostSpec {
            reserve: 5.0,
            min_width: 50.0,
            min_height: 0.0,
            copies: 1,
        };
        let b = billed_blocks(
            &[rect(0.0, 0.0, 100.0, 20.0), rect(0.0, 0.0, 10.0, 10.0)],
            &s,
        );
        assert_eq!(b, vec![(110.0, 30.0), (50.0, 20.0)]);
    }

    #[test]
    fn material_by_area_bills_the_blocks() {
        // Two blocks of 1000 x 500 with 0 reserve: 1 m2 at 20.
        let q = Quantities {
            blocks: vec![
                rect(0.0, 0.0, 1000.0, 500.0),
                rect(0.0, 600.0, 1000.0, 500.0),
            ],
            ..Default::default()
        };
        let r = cost(&q, Some(&by_area(20.0)), &[], &spec(0.0));
        assert_eq!(r.lines.len(), 1);
        assert!((r.lines[0].quantity - 1.0).abs() < 1e-12);
        assert!((r.total - 20.0).abs() < 1e-9);
        assert!((r.billed_area - 1_000_000.0).abs() < 1e-6);
        assert!(r.roll.is_none());
    }

    #[test]
    fn a_roll_is_billed_by_the_length_used() {
        let roll = Material {
            name: "Vinyl".into(),
            unit: MaterialUnit::Roll,
            price: 10.0,
            roll_width: 630.0,
        };
        // 400 wide, 1990 high, reserve 5: 410 x 2000 along the roll.
        let q = Quantities {
            blocks: vec![rect(0.0, 0.0, 400.0, 1990.0)],
            ..Default::default()
        };
        let r = cost(&q, Some(&roll), &[], &spec(5.0));
        let ru = r.roll.expect("roll");
        assert!((ru.length - 2000.0).abs() < 1e-9 && !ru.turned && ru.lanes == 1);
        assert!((r.total - 20.0).abs() < 1e-9);
    }

    #[test]
    fn a_wide_job_turns_or_takes_lanes() {
        let s = spec(0.0);
        // 1000 x 300 on 630: turned, 1000 long.
        let r = roll_use(&[rect(0.0, 0.0, 1000.0, 300.0)], 630.0, &s);
        assert!(r.turned && r.lanes == 1 && (r.length - 1000.0).abs() < 1e-9);
        // 1000 x 2000 on 630: two lanes across the 1000, each 2000 long.
        let r = roll_use(&[rect(0.0, 0.0, 1000.0, 2000.0)], 630.0, &s);
        assert_eq!(r.lanes, 2);
        assert!((r.length - 4000.0).abs() < 1e-9);
        // No width given: anything fits along.
        let r = roll_use(&[rect(0.0, 0.0, 1000.0, 2000.0)], f64::NAN, &s);
        assert!(!r.turned && (r.length - 2000.0).abs() < 1e-9);
        // Nothing to place.
        assert_eq!(roll_use(&[], 630.0, &s).length, 0.0);
    }

    #[test]
    fn services_bill_their_own_quantity_times_copies() {
        let q = Quantities {
            blocks: vec![rect(0.0, 0.0, 1000.0, 1000.0), rect(0.0, 0.0, 10.0, 10.0)],
            cut_length: 2500.0,
            hours: 0.5,
        };
        let mut services = default_services();
        for (s, on) in services.iter_mut().zip([true, false, false, false, false]) {
            s.selected = on;
        }
        services.push(Service {
            name: "Per piece".into(),
            unit: ServiceUnit::Piece,
            price: 2.0,
            selected: true,
        });
        services.push(Service {
            name: "Per hour".into(),
            unit: ServiceUnit::Hour,
            price: 30.0,
            selected: true,
        });
        services.push(Service {
            name: "Per m2".into(),
            unit: ServiceUnit::SquareMetre,
            price: 10.0,
            selected: true,
        });
        let s = CostSpec {
            reserve: 0.0,
            copies: 2,
            ..Default::default()
        };
        let r = cost(&q, None, &services, &s);
        assert_eq!(r.lines.len(), 4);
        // Cutting 2.5 m x 2 x 0.5 = 2.5; pieces 2 x 2 x 2 = 8;
        // hours 0.5 x 2 x 30 = 30; area 1.0001 x 2 x 10 = 20.002.
        let amounts: Vec<f64> = r.lines.iter().map(|l| l.amount).collect();
        assert!((amounts[0] - 2.5).abs() < 1e-9);
        assert!((amounts[1] - 8.0).abs() < 1e-9);
        assert!((amounts[2] - 30.0).abs() < 1e-9);
        assert!((amounts[3] - 20.002).abs() < 1e-9);
        assert!((r.total - 60.502).abs() < 1e-9);
    }

    #[test]
    fn bad_numbers_bill_nothing_and_do_not_panic() {
        let q = Quantities {
            blocks: vec![
                Rect::new(f64::NAN, 0.0, 1.0, 1.0),
                rect(0.0, 0.0, 1000.0, 1000.0),
            ],
            cut_length: f64::INFINITY,
            hours: f64::NAN,
        };
        let s = CostSpec {
            reserve: f64::NAN,
            min_width: -5.0,
            min_height: f64::INFINITY,
            copies: 0,
        };
        let mut sv = default_services();
        for x in &mut sv {
            x.selected = true;
            x.price = f64::NAN;
        }
        let r = cost(&q, Some(&by_area(-3.0)), &sv, &s);
        assert!(r.total.is_finite());
        assert_eq!(r.total, 0.0);
        assert_eq!(billed_blocks(&q.blocks, &s), vec![(1000.0, 1000.0)]);
    }

    #[test]
    fn material_lists_go_through_csv_and_back() {
        let list = default_materials();
        let text = materials_to_csv(&list);
        let back = materials_from_csv(&text);
        assert_eq!(back.items, list);
        assert_eq!(back.skipped, 0);
    }

    #[test]
    fn service_lists_go_through_csv_and_back() {
        let mut list = default_services();
        for s in &mut list {
            s.selected = false;
        }
        let back = services_from_csv(&services_to_csv(&list));
        assert_eq!(back.items, list);
    }

    #[test]
    fn spreadsheet_csv_with_semicolons_and_bad_rows() {
        let text = "Name;Unit;Price;Width\n\
                    \"Vinyl; white\";roll;8,50;630\n\
                    Board;m\u{b2};1 234,5\n\
                    No width;roll;5\n\
                    Negative;m2;-1\n\
                    ;m2;3\n\
                    Odd;litre;3\n";
        let got = materials_from_csv(text);
        assert_eq!(got.items.len(), 2);
        assert_eq!(got.items[0].name, "Vinyl; white");
        assert_eq!(got.items[0].price, 8.5);
        assert_eq!(got.items[0].roll_width, 630.0);
        assert_eq!(got.items[1].unit, MaterialUnit::Area);
        assert_eq!(got.items[1].price, 1234.5);
        assert_eq!(got.skipped, 4);
        let sv = services_from_csv("Cut,m,0.4\nTime,h,30\nJunk\n");
        assert_eq!(sv.items.len(), 2);
        assert_eq!(sv.items[1].unit, ServiceUnit::Hour);
        assert_eq!(sv.skipped, 1);
    }
}
