//! Layout Functionality



use std::{cell::RefCell, collections::HashMap, num::NonZeroUsize, rc::Rc, sync::OnceLock};

use cassowary::{
    strength::REQUIRED,
    AddConstraintError, Expression, Solver, Variable,
    WeightedRelation::{EQ, GE, LE},
};
use itertools::Itertools as _;
use strengths::*;

use crate::prelude::*;



type Rects = Rc<[Rect]>;
type Segments = Rects;
type Spacers = Rects;
// The solution to a Layout solve contains two `Rects`, where `Rects` is effectively a `[Rect]`.
//
// 1. `[Rect]` that contains positions for the segments corresponding to user provided constraints
// 2. `[Rect]` that contains spacers around the user provided constraints
//
// <------------------------------------80 px------------------------------------->
// ┌   ┐┌──────────────────┐┌   ┐┌──────────────────┐┌   ┐┌──────────────────┐┌   ┐
//   1  │        a         │  2  │         b        │  3  │         c        │  4
// └   ┘└──────────────────┘└   ┘└──────────────────┘└   ┘└──────────────────┘└   ┘
//
// Number of spacers will always be one more than number of segments.
type Cache = lru::LruCache<(Rect, Layout), (Segments, Spacers)>;

// Multiplier that decides floating point precision when rounding.
// The number of zeros in this number is the precision for the rounding of f64 to u16 in layout
// calculations.
const FLOAT_PRECISION_MULTIPLIER: f64 = 100.0;

thread_local! {
    static LAYOUT_CACHE: OnceLock<RefCell<Cache>> = const { OnceLock::new() };
}



// ================================================================================================



#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Layout {
    orient: Orientation,
    constraints: Vec<Constraint>,
    margin: Margin,
    flex: Flex,
    spacing: u16,
}

impl Layout {
    /// This is a somewhat arbitrary size for the layout cache. This gives enough entries to store 
    /// a layout for every row and every column, twice over, which should be enough for most apps. 
    /// 
    /// For those that need more, the cache size can be set with [`Layout::init_cache()`].
    pub const DEFAULT_CACHE_SIZE: usize = 500;

    /// Creates a new layout with default values.
    ///
    /// The `constraints` parameter accepts any type that implements `IntoIterator<Item =
    /// Into<Constraint>>`. This includes arrays, slices, vectors, iterators. `Into<Constraint>` is
    /// implemented on `u16`, so you can pass an array, `Vec`, etc. of `u16` to this function to
    /// create a layout with fixed size chunks.
    ///
    /// Default values for the other fields are:
    ///
    /// - `margin`: 0, 0
    /// - `flex`: [`Flex::Start`]
    /// - `spacing`: 0
    pub fn new<I>(orient: Orientation, constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        Self {
            orient,
            constraints: constraints.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    /// Creates a new vertical layout with default values.
    ///
    /// The `constraints` parameter accepts any type that implements `IntoIterator<Item =
    /// Into<Constraint>>`. This includes arrays, slices, vectors, iterators, etc.
    pub fn vertical<I>(constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        Self::new(Orientation::Vertical, constraints.into_iter().map(Into::into))
    }

    /// Creates a new horizontal layout with default values.
    ///
    /// The `constraints` parameter accepts any type that implements `IntoIterator<Item =
    /// Into<Constraint>>`. This includes arrays, slices, vectors, iterators, etc.
    pub fn horizontal<I>(constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        Self::new(
            Orientation::Horizontal,
            constraints.into_iter().map(Into::into),
        )
    }

    /// Initialize an empty cache with a custom size. The cache is keyed on the layout and area, so
    /// that subsequent calls with the same parameters are faster. The cache is a `LruCache`, and
    /// grows until `cache_size` is reached.
    ///
    /// Returns true if the cell's value was set by this call.
    /// Returns false if the cell's value was not set by this call, this means that another thread
    /// has set this value or that the cache size is already initialized.
    ///
    /// Note that a custom cache size will be set only if this function:
    /// * is called before [`Layout::split()`] otherwise, the cache size is
    ///   [`Self::DEFAULT_CACHE_SIZE`].
    /// * is called for the first time, subsequent calls do not modify the cache size.
    pub fn init_cache(cache_size: usize) -> bool {
        LAYOUT_CACHE
            .with(|c| {
                c.set(RefCell::new(lru::LruCache::new(
                    NonZeroUsize::new(cache_size).unwrap(),
                )))
            })
            .is_ok()
    }

    pub fn constraints<I>(mut self, constraints: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Constraint>,
    {
        self.constraints = constraints.into_iter().map(Into::into).collect();
        self
    }

    pub const fn margin(mut self, margin: u16) -> Self {
        self.margin = Margin {
            horizontal: margin,
            vertical: margin,
        };
        self
    }

    pub const fn horizontal_margin(mut self, horizontal: u16) -> Self {
        self.margin.horizontal = horizontal;
        self
    }

    pub const fn vertical_margin(mut self, vertical: u16) -> Self {
        self.margin.vertical = vertical;
        self
    }

    pub const fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }
}

impl Layout {
    pub fn areas<const N: usize>(&self, area: Rect) -> [Rect; N] {
        let (areas, _) = self.split_with_spacers(area);
        areas.as_ref().try_into().expect("invalid number of rects")
    }

    pub fn spacers<const N: usize>(&self, area: Rect) -> [Rect; N] {
        let (_, spacers) = self.split_with_spacers(area);
        spacers
            .as_ref()
            .try_into()
            .expect("invalid number of rects")
    }

    pub fn split(&self, area: Rect) -> Rects {
        self.split_with_spacers(area).0
    }

    pub fn split_with_spacers(&self, area: Rect) -> (Segments, Spacers) {
        LAYOUT_CACHE.with(|c| {
            c.get_or_init(|| {
                RefCell::new(lru::LruCache::new(
                    NonZeroUsize::new(Self::DEFAULT_CACHE_SIZE).unwrap(),
                ))
            })
            .borrow_mut()
            .get_or_insert((area, self.clone()), || {
                self.try_split(area).expect("failed to split")
            })
            .clone()
        })
    }

    fn try_split(&self, area: Rect) -> Result<(Segments, Spacers), AddConstraintError> {
        // To take advantage of all of cassowary features, we would want to store the `Solver` in
        // one of the fields of the Layout struct. And we would want to set it up such that we could
        // add or remove constraints as and when needed.
        // The advantage of doing it as described above is that it would allow users to
        // incrementally add and remove constraints efficiently.
        // Solves with just one constraint different would not need to resolve the entire layout.
        //
        // The disadvantage of this approach is that it requires tracking which constraints were
        // added, and which variables they correspond to.
        // This will also require introducing and maintaining the API for users to do so.
        //
        // Currently we don't support that use case and do not intend to support it in the future,
        // and instead we require that the user re-solve the layout every time they call `split`.
        // To minimize the time it takes to solve the same problem over and over again, we
        // cache the `Layout` struct along with the results.
        //
        // `try_split` is the inner method in `split` that is called only when the LRU cache doesn't
        // match the key. So inside `try_split`, we create a new instance of the solver.
        //
        // This is equivalent to storing the solver in `Layout` and calling `solver.reset()` here.
        let mut solver = Solver::new();

        let inner_area = area.inner(self.margin);
        let (area_start, area_end) = match self.orient {
            Orientation::Horizontal => (
                f64::from(inner_area.x) * FLOAT_PRECISION_MULTIPLIER,
                f64::from(inner_area.right()) * FLOAT_PRECISION_MULTIPLIER,
            ),
            Orientation::Vertical => (
                f64::from(inner_area.y) * FLOAT_PRECISION_MULTIPLIER,
                f64::from(inner_area.bottom()) * FLOAT_PRECISION_MULTIPLIER,
            ),
        };

        // ```plain
        // <───────────────────────────────────area_width─────────────────────────────────>
        // ┌─area_start                                                          area_end─┐
        // V                                                                              V
        // ┌────┬───────────────────┬────┬─────variables─────┬────┬───────────────────┬────┐
        // │    │                   │    │                   │    │                   │    │
        // V    V                   V    V                   V    V                   V    V
        // ┌   ┐┌──────────────────┐┌   ┐┌──────────────────┐┌   ┐┌──────────────────┐┌   ┐
        //      │     Max(20)      │     │      Max(20)     │     │      Max(20)     │
        // └   ┘└──────────────────┘└   ┘└──────────────────┘└   ┘└──────────────────┘└   ┘
        // ^    ^                   ^    ^                   ^    ^                   ^    ^
        // │    │                   │    │                   │    │                   │    │
        // └─┬──┶━━━━━━━━━┳━━━━━━━━━┵─┬──┶━━━━━━━━━┳━━━━━━━━━┵─┬──┶━━━━━━━━━┳━━━━━━━━━┵─┬──┘
        //   │            ┃           │            ┃           │            ┃           │
        //   └────────────╂───────────┴────────────╂───────────┴────────────╂──Spacers──┘
        //                ┃                        ┃                        ┃
        //                ┗━━━━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━Segments━━━━━━━━┛
        // ```

        let variable_count = self.constraints.len() * 2 + 2;
        let variables = std::iter::repeat_with(Variable::new)
            .take(variable_count)
            .collect_vec();
        let spacers = variables
            .iter()
            .tuples()
            .map(|(a, b)| Element::from((*a, *b)))
            .collect_vec();
        let segments = variables
            .iter()
            .skip(1)
            .tuples()
            .map(|(a, b)| Element::from((*a, *b)))
            .collect_vec();

        let flex = self.flex;
        let spacing = self.spacing;
        let constraints = &self.constraints;

        let area_size = Element::from((*variables.first().unwrap(), *variables.last().unwrap()));
        configure_area(&mut solver, area_size, area_start, area_end)?;
        configure_variable_constraints(&mut solver, &variables, area_size)?;
        configure_flex_constraints(&mut solver, area_size, &spacers, flex, spacing)?;
        configure_constraints(&mut solver, area_size, &segments, constraints, flex)?;
        configure_fill_constraints(&mut solver, &segments, constraints, flex)?;

        if !flex.is_legacy() {
            for (left, right) in segments.iter().tuple_windows() {
                solver.add_constraint(left.has_size(right, ALL_SEGMENT_GROW))?;
            }
        }

        // `solver.fetch_changes()` can only be called once per solve
        let changes: HashMap<Variable, f64> = solver.fetch_changes().iter().copied().collect();
        // debug_segments(&segments, &changes);

        let segment_rects = changes_to_rects(&changes, &segments, inner_area, self.orient);
        let spacer_rects = changes_to_rects(&changes, &spacers, inner_area, self.orient);

        Ok((segment_rects, spacer_rects))
    }
}

fn configure_area(
    solver: &mut Solver,
    area: Element,
    area_start: f64,
    area_end: f64,
) -> Result<(), AddConstraintError> {
    solver.add_constraint(area.start | EQ(REQUIRED) | area_start)?;
    solver.add_constraint(area.end | EQ(REQUIRED) | area_end)?;
    Ok(())
}

fn configure_variable_constraints(
    solver: &mut Solver,
    variables: &[Variable],
    area: Element,
) -> Result<(), AddConstraintError> {
    // all variables are in the range [area.start, area.end]
    for &variable in variables {
        solver.add_constraint(variable | GE(REQUIRED) | area.start)?;
        solver.add_constraint(variable | LE(REQUIRED) | area.end)?;
    }

    // all variables are in ascending order
    for (&left, &right) in variables.iter().tuple_windows() {
        solver.add_constraint(left | LE(REQUIRED) | right)?;
    }

    Ok(())
}

fn configure_constraints(
    solver: &mut Solver,
    area: Element,
    segments: &[Element],
    constraints: &[Constraint],
    flex: Flex,
) -> Result<(), AddConstraintError> {
    for (&constraint, &element) in constraints.iter().zip(segments.iter()) {
        match constraint {
            Constraint::Max(max) => {
                solver.add_constraint(element.has_max_size(max, MAX_SIZE_LE))?;
                solver.add_constraint(element.has_int_size(max, MAX_SIZE_EQ))?;
            }
            Constraint::Min(min) => {
                solver.add_constraint(element.has_min_size(min, MIN_SIZE_GE))?;
                if flex.is_legacy() {
                    solver.add_constraint(element.has_int_size(min, MIN_SIZE_EQ))?;
                } else {
                    solver.add_constraint(element.has_size(area, FILL_GROW))?;
                }
            }
            Constraint::Length(length) => {
                solver.add_constraint(element.has_int_size(length, LENGTH_SIZE_EQ))?;
            }
            Constraint::Percentage(p) => {
                let size = area.size() * f64::from(p) / 100.00;
                solver.add_constraint(element.has_size(size, PERCENTAGE_SIZE_EQ))?;
            }
            Constraint::Ratio(num, den) => {
                // avoid division by zero by using 1 when denominator is 0
                let size = area.size() * f64::from(num) / f64::from(den.max(1));
                solver.add_constraint(element.has_size(size, RATIO_SIZE_EQ))?;
            }
            Constraint::Fill(_) => {
                // given no other constraints, this segment will grow as much as possible.
                solver.add_constraint(element.has_size(area, FILL_GROW))?;
            }
        }
    }
    Ok(())
}

fn configure_flex_constraints(
    solver: &mut Solver,
    area: Element,
    spacers: &[Element],
    flex: Flex,
    spacing: u16,
) -> Result<(), AddConstraintError> {
    let spacers_except_first_and_last = spacers.get(1..spacers.len() - 1).unwrap_or(&[]);
    let spacing_f64 = f64::from(spacing) * FLOAT_PRECISION_MULTIPLIER;
    match flex {
        Flex::Legacy => {
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_size(spacing_f64, SPACER_SIZE_EQ))?;
            }
            if let (Some(first), Some(last)) = (spacers.first(), spacers.last()) {
                solver.add_constraint(first.is_empty())?;
                solver.add_constraint(last.is_empty())?;
            }
        }
        // all spacers are the same size and will grow to fill any remaining space after the
        // constraints are satisfied
        Flex::SpaceAround => {
            for (left, right) in spacers.iter().tuple_combinations() {
                solver.add_constraint(left.has_size(right, SPACER_SIZE_EQ))?;
            }
            for spacer in spacers {
                solver.add_constraint(spacer.has_min_size(spacing, SPACER_SIZE_EQ))?;
                solver.add_constraint(spacer.has_size(area, SPACE_GROW))?;
            }
        }
        
        // all spacers are the same size and will grow to fill any remaining space after the
        // constraints are satisfied, but the first and last spacers are zero size
        Flex::SpaceBetween => {
            for (left, right) in spacers_except_first_and_last.iter().tuple_combinations() {
                solver.add_constraint(left.has_size(right.size(), SPACER_SIZE_EQ))?;
            }
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_min_size(spacing, SPACER_SIZE_EQ))?;
            }
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_size(area, SPACE_GROW))?;
            }
            if let (Some(first), Some(last)) = (spacers.first(), spacers.last()) {
                solver.add_constraint(first.is_empty())?;
                solver.add_constraint(last.is_empty())?;
            }
        }
        Flex::Start => {
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_size(spacing_f64, SPACER_SIZE_EQ))?;
            }
            if let (Some(first), Some(last)) = (spacers.first(), spacers.last()) {
                solver.add_constraint(first.is_empty())?;
                solver.add_constraint(last.has_size(area, GROW))?;
            }
        }
        Flex::Center => {
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_size(spacing_f64, SPACER_SIZE_EQ))?;
            }
            if let (Some(first), Some(last)) = (spacers.first(), spacers.last()) {
                solver.add_constraint(first.has_size(area, GROW))?;
                solver.add_constraint(last.has_size(area, GROW))?;
                solver.add_constraint(first.has_size(last, SPACER_SIZE_EQ))?;
            }
        }
        Flex::End => {
            for spacer in spacers_except_first_and_last {
                solver.add_constraint(spacer.has_size(spacing_f64, SPACER_SIZE_EQ))?;
            }
            if let (Some(first), Some(last)) = (spacers.first(), spacers.last()) {
                solver.add_constraint(last.is_empty())?;
                solver.add_constraint(first.has_size(area, GROW))?;
            }
        }
    }
    Ok(())
}

/// Make every `Fill` constraint proportionally equal to each other
/// This will make it fill up empty spaces equally
///
/// [Fill(1), Fill(1)]
/// ┌──────┐┌──────┐
/// │abcdef││abcdef│
/// └──────┘└──────┘
///
/// [Min(0), Fill(2)]
/// ┌──────┐┌────────────┐
/// │abcdef││abcdefabcdef│
/// └──────┘└────────────┘
///
/// `size == base_element * scaling_factor`
fn configure_fill_constraints(
    solver: &mut Solver,
    segments: &[Element],
    constraints: &[Constraint],
    flex: Flex,
) -> Result<(), AddConstraintError> {
    for ((&left_constraint, &left_element), (&right_constraint, &right_element)) in constraints
        .iter()
        .zip(segments.iter())
        .filter(|(c, _)| c.is_fill() || (!flex.is_legacy() && c.is_min()))
        .tuple_combinations()
    {
        let left_scaling_factor = match left_constraint {
            Constraint::Fill(scale) => f64::from(scale).max(1e-6),
            Constraint::Min(_) => 1.0,
            _ => unreachable!(),
        };
        let right_scaling_factor = match right_constraint {
            Constraint::Fill(scale) => f64::from(scale).max(1e-6),
            Constraint::Min(_) => 1.0,
            _ => unreachable!(),
        };
        solver.add_constraint(
            (right_scaling_factor * left_element.size())
                | EQ(GROW)
                | (left_scaling_factor * right_element.size()),
        )?;
    }
    Ok(())
}

fn changes_to_rects(
    changes: &HashMap<Variable, f64>,
    elements: &[Element],
    area: Rect,
    orient: Orientation,
) -> Rects {
    // convert to Rects
    elements
        .iter()
        .map(|element| {
            let start = changes.get(&element.start).unwrap_or(&0.0);
            let end = changes.get(&element.end).unwrap_or(&0.0);
            let start = (start.round() / FLOAT_PRECISION_MULTIPLIER).round() as u16;
            let end = (end.round() / FLOAT_PRECISION_MULTIPLIER).round() as u16;
            let size = end.saturating_sub(start);
            match orient {
                Orientation::Horizontal => Rect {
                    x: start,
                    y: area.y,
                    width: size,
                    height: area.height,
                },
                Orientation::Vertical => Rect {
                    x: area.x,
                    y: start,
                    width: area.width,
                    height: size,
                },
            }
        })
        .collect::<Rects>()
}

/// Please leave this here, as it's useful for debugging unit tests when we make any changes to
/// layout code - we should replace this with tracing in the future.
#[allow(dead_code)]
fn debug_segments(segments: &[Element], changes: &HashMap<Variable, f64>) {
    let ends = format!(
        "{:?}",
        segments
            .iter()
            .map(|e| changes.get(&e.end).unwrap_or(&0.0))
            .collect::<Vec<&f64>>()
    );
    dbg!(ends);
}

/// A container used by the solver inside split
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct Element {
    start: Variable,
    end: Variable,
}

impl From<(Variable, Variable)> for Element {
    fn from((start, end): (Variable, Variable)) -> Self {
        Self { start, end }
    }
}

impl Element {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            start: Variable::new(),
            end: Variable::new(),
        }
    }

    fn size(&self) -> Expression {
        self.end - self.start
    }

    fn has_max_size(&self, size: u16, strength: f64) -> cassowary::Constraint {
        self.size() | LE(strength) | (f64::from(size) * FLOAT_PRECISION_MULTIPLIER)
    }

    fn has_min_size(&self, size: u16, strength: f64) -> cassowary::Constraint {
        self.size() | GE(strength) | (f64::from(size) * FLOAT_PRECISION_MULTIPLIER)
    }

    fn has_int_size(&self, size: u16, strength: f64) -> cassowary::Constraint {
        self.size() | EQ(strength) | (f64::from(size) * FLOAT_PRECISION_MULTIPLIER)
    }

    fn has_size<E: Into<Expression>>(&self, size: E, strength: f64) -> cassowary::Constraint {
        self.size() | EQ(strength) | size.into()
    }

    fn is_empty(&self) -> cassowary::Constraint {
        self.size() | EQ(REQUIRED - 1.0) | 0.0
    }
}

/// Allow the element to represent its own size in expressions
impl From<Element> for Expression {
    fn from(element: Element) -> Self {
        element.size()
    }
}

/// Allow the element to represent its own size in expressions
impl From<&Element> for Expression {
    fn from(element: &Element) -> Self {
        element.size()
    }
}

mod strengths {
    use cassowary::strength::{MEDIUM, REQUIRED, STRONG, WEAK};
    /// The strength to apply to Spacers to ensure that their sizes are equal.
    ///
    /// ┌     ┐┌───┐┌     ┐┌───┐┌     ┐
    ///   ==x  │   │  ==x  │   │  ==x
    /// └     ┘└───┘└     ┘└───┘└     ┘
    pub const SPACER_SIZE_EQ: f64 = REQUIRED - 1.0;

    /// The strength to apply to Min inequality constraints.
    ///
    /// ┌────────┐
    /// │Min(>=x)│
    /// └────────┘
    pub const MIN_SIZE_GE: f64 = STRONG * 100.0;

    /// The strength to apply to Max inequality constraints.
    ///
    /// ┌────────┐
    /// │Max(<=x)│
    /// └────────┘
    pub const MAX_SIZE_LE: f64 = STRONG * 100.0;

    /// The strength to apply to Length constraints.
    ///
    /// ┌───────────┐
    /// │Length(==x)│
    /// └───────────┘
    pub const LENGTH_SIZE_EQ: f64 = STRONG * 10.0;

    /// The strength to apply to Percentage constraints.
    ///
    /// ┌───────────────┐
    /// │Percentage(==x)│
    /// └───────────────┘
    pub const PERCENTAGE_SIZE_EQ: f64 = STRONG;

    /// The strength to apply to Ratio constraints.
    ///
    /// ┌────────────┐
    /// │Ratio(==x,y)│
    /// └────────────┘
    pub const RATIO_SIZE_EQ: f64 = STRONG / 10.0;

    /// The strength to apply to Min equality constraints.
    ///
    /// ┌────────┐
    /// │Min(==x)│
    /// └────────┘
    pub const MIN_SIZE_EQ: f64 = MEDIUM * 10.0;

    /// The strength to apply to Max equality constraints.
    ///
    /// ┌────────┐
    /// │Max(==x)│
    /// └────────┘
    pub const MAX_SIZE_EQ: f64 = MEDIUM * 10.0;

    /// The strength to apply to Fill growing constraints.
    ///
    /// ┌─────────────────────┐
    /// │<=     Fill(x)     =>│
    /// └─────────────────────┘
    pub const FILL_GROW: f64 = MEDIUM;

    /// The strength to apply to growing constraints.
    ///
    /// ┌────────────┐
    /// │<= Min(x) =>│
    /// └────────────┘
    pub const GROW: f64 = MEDIUM / 10.0;

    /// The strength to apply to Spacer growing constraints.
    ///
    /// ┌       ┐
    ///  <= x =>
    /// └       ┘
    pub const SPACE_GROW: f64 = WEAK * 10.0;

    /// The strength to apply to growing the size of all segments equally.
    ///
    /// ┌───────┐
    /// │<= x =>│
    /// └───────┘
    pub const ALL_SEGMENT_GROW: f64 = WEAK;
}



// ================================================================================================



/// Defines the options for layout flex justify content in a container.
///
/// This enumeration controls the distribution of space when layout constraints are met.
///
/// - `Legacy`: Fills the available space within the container, putting excess space into the last
///   element.
/// - `Start`: Aligns items to the start of the container.
/// - `End`: Aligns items to the end of the container.
/// - `Center`: Centers items within the container.
/// - `SpaceBetween`: Adds excess space between each element.
/// - `SpaceAround`: Adds excess space around each element.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Flex {
    /// Fills the available space within the container, putting excess space into the last
    /// constraint of the lowest priority. This matches the default behavior of ratatui and tui
    /// applications without [`Flex`]
    ///
    /// The following examples illustrate the allocation of excess in various combinations of
    /// constraints. As a refresher, the priorities of constraints are as follows:
    ///
    /// 1. [`Constraint::Min`]
    /// 2. [`Constraint::Max`]
    /// 3. [`Constraint::Length`]
    /// 4. [`Constraint::Percentage`]
    /// 5. [`Constraint::Ratio`]
    /// 6. [`Constraint::Fill`]
    ///
    /// When every constraint is `Length`, the last element gets the excess.
    ///
    /// ```plain
    /// <----------------------------------- 80 px ------------------------------------>
    /// ┌──────20 px───────┐┌──────20 px───────┐┌────────────────40 px─────────────────┐
    /// │    Length(20)    ││    Length(20)    ││              Length(20)              │
    /// └──────────────────┘└──────────────────┘└──────────────────────────────────────┘
    ///                                         ^^^^^^^^^^^^^^^^ EXCESS ^^^^^^^^^^^^^^^^
    /// ```
    ///
    /// Fill constraints have the lowest priority amongst all the constraints and hence
    /// will always take up any excess space available.
    ///
    /// ```plain
    /// <----------------------------------- 80 px ------------------------------------>
    /// ┌──────20 px───────┐┌──────20 px───────┐┌──────20 px───────┐┌──────20 px───────┐
    /// │      Fill(0)     ││      Max(20)     ││    Length(20)    ││     Length(20)   │
    /// └──────────────────┘└──────────────────┘└──────────────────┘└──────────────────┘
    /// ^^^^^^ EXCESS ^^^^^^
    /// ```
    ///
    /// # Examples
    ///
    /// ```plain
    /// <------------------------------------80 px------------------------------------->
    /// ┌──────────────────────────60 px───────────────────────────┐┌──────20 px───────┐
    /// │                          Min(20)                         ││      Max(20)     │
    /// └──────────────────────────────────────────────────────────┘└──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    /// ┌────────────────────────────────────80 px─────────────────────────────────────┐
    /// │                                    Max(20)                                   │
    /// └──────────────────────────────────────────────────────────────────────────────┘
    /// ```
    Legacy,

    /// Aligns items to the start of the container.
    ///
    /// # Examples
    ///
    /// ```plain
    /// <------------------------------------80 px------------------------------------->
    /// ┌────16 px─────┐┌──────20 px───────┐┌──────20 px───────┐
    /// │Percentage(20)││    Length(20)    ││     Fixed(20)    │
    /// └──────────────┘└──────────────────┘└──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    /// ┌──────20 px───────┐┌──────20 px───────┐
    /// │      Max(20)     ││      Max(20)     │
    /// └──────────────────┘└──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    /// ┌──────20 px───────┐
    /// │      Max(20)     │
    /// └──────────────────┘
    /// ```
    #[default]
    Start,

    /// Aligns items to the end of the container.
    ///
    /// # Examples
    ///
    /// ```plain
    /// <------------------------------------80 px------------------------------------->
    ///                         ┌────16 px─────┐┌──────20 px───────┐┌──────20 px───────┐
    ///                         │Percentage(20)││    Length(20)    ││     Length(20)   │
    ///                         └──────────────┘└──────────────────┘└──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    ///                                         ┌──────20 px───────┐┌──────20 px───────┐
    ///                                         │      Max(20)     ││      Max(20)     │
    ///                                         └──────────────────┘└──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    ///                                                             ┌──────20 px───────┐
    ///                                                             │      Max(20)     │
    ///                                                             └──────────────────┘
    /// ```
    End,

    /// Centers items within the container.
    ///
    /// # Examples
    ///
    /// ```plain
    /// <------------------------------------80 px------------------------------------->
    ///             ┌────16 px─────┐┌──────20 px───────┐┌──────20 px───────┐
    ///             │Percentage(20)││    Length(20)    ││     Length(20)   │
    ///             └──────────────┘└──────────────────┘└──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    ///                     ┌──────20 px───────┐┌──────20 px───────┐
    ///                     │      Max(20)     ││      Max(20)     │
    ///                     └──────────────────┘└──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    ///                               ┌──────20 px───────┐
    ///                               │      Max(20)     │
    ///                               └──────────────────┘
    /// ```
    Center,

    /// Adds excess space between each element.
    ///
    /// # Examples
    ///
    /// ```plain
    /// <------------------------------------80 px------------------------------------->
    /// ┌────16 px─────┐            ┌──────20 px───────┐            ┌──────20 px───────┐
    /// │Percentage(20)│            │    Length(20)    │            │     Length(20)   │
    /// └──────────────┘            └──────────────────┘            └──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    /// ┌──────20 px───────┐                                        ┌──────20 px───────┐
    /// │      Max(20)     │                                        │      Max(20)     │
    /// └──────────────────┘                                        └──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    /// ┌────────────────────────────────────80 px─────────────────────────────────────┐
    /// │                                    Max(20)                                   │
    /// └──────────────────────────────────────────────────────────────────────────────┘
    /// ```
    SpaceBetween,

    /// Adds excess space around each element.
    ///
    /// # Examples
    ///
    /// ```plain
    /// <------------------------------------80 px------------------------------------->
    ///       ┌────16 px─────┐      ┌──────20 px───────┐      ┌──────20 px───────┐
    ///       │Percentage(20)│      │    Length(20)    │      │     Length(20)   │
    ///       └──────────────┘      └──────────────────┘      └──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    ///              ┌──────20 px───────┐              ┌──────20 px───────┐
    ///              │      Max(20)     │              │      Max(20)     │
    ///              └──────────────────┘              └──────────────────┘
    ///
    /// <------------------------------------80 px------------------------------------->
    ///                               ┌──────20 px───────┐
    ///                               │      Max(20)     │
    ///                               └──────────────────┘
    /// ```
    SpaceAround,
}

impl Flex {
    pub const fn is_legacy(&self) -> bool {
        matches!(self, Flex::Legacy)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Constraint {
    /// Applies a minimum size constraint to the element
    ///
    /// The element size is set to at least the specified amount.
    ///
    /// # Examples
    ///
    /// `[Percentage(100), Min(20)]`
    ///
    /// ```plain
    /// ┌────────────────────────────┐┌──────────────────┐
    /// │            30 px           ││       20 px      │
    /// └────────────────────────────┘└──────────────────┘
    /// ```
    ///
    /// `[Percentage(100), Min(10)]`
    ///
    /// ```plain
    /// ┌──────────────────────────────────────┐┌────────┐
    /// │                 40 px                ││  10 px │
    /// └──────────────────────────────────────┘└────────┘
    /// ```
    Min(u16),

    /// Applies a maximum size constraint to the element
    ///
    /// The element size is set to at most the specified amount.
    ///
    /// # Examples
    ///
    /// `[Percentage(0), Max(20)]`
    ///
    /// ```plain
    /// ┌────────────────────────────┐┌──────────────────┐
    /// │            30 px           ││       20 px      │
    /// └────────────────────────────┘└──────────────────┘
    /// ```
    ///
    /// `[Percentage(0), Max(10)]`
    ///
    /// ```plain
    /// ┌──────────────────────────────────────┐┌────────┐
    /// │                 40 px                ││  10 px │
    /// └──────────────────────────────────────┘└────────┘
    /// ```
    Max(u16),

    /// Applies a length constraint to the element
    ///
    /// The element size is set to the specified amount.
    ///
    /// # Examples
    ///
    /// `[Length(20), Length(20)]`
    ///
    /// ```plain
    /// ┌──────────────────┐┌──────────────────┐
    /// │       20 px      ││       20 px      │
    /// └──────────────────┘└──────────────────┘
    /// ```
    ///
    /// `[Length(20), Length(30)]`
    ///
    /// ```plain
    /// ┌──────────────────┐┌────────────────────────────┐
    /// │       20 px      ││            30 px           │
    /// └──────────────────┘└────────────────────────────┘
    /// ```
    Length(u16),

    /// Applies a percentage of the available space to the element
    ///
    /// Converts the given percentage to a floating-point value and multiplies that with area.
    /// This value is rounded back to a integer as part of the layout split calculation.
    ///
    /// # Examples
    ///
    /// `[Percentage(75), Fill(1)]`
    ///
    /// ```plain
    /// ┌────────────────────────────────────┐┌──────────┐
    /// │                38 px               ││   12 px  │
    /// └────────────────────────────────────┘└──────────┘
    /// ```
    ///
    /// `[Percentage(50), Fill(1)]`
    ///
    /// ```plain
    /// ┌───────────────────────┐┌───────────────────────┐
    /// │         25 px         ││         25 px         │
    /// └───────────────────────┘└───────────────────────┘
    /// ```
    Percentage(u16),

    /// Applies a ratio of the available space to the element
    ///
    /// Converts the given ratio to a floating-point value and multiplies that with area.
    /// This value is rounded back to a integer as part of the layout split calculation.
    ///
    /// # Examples
    ///
    /// `[Ratio(1, 2) ; 2]`
    ///
    /// ```plain
    /// ┌───────────────────────┐┌───────────────────────┐
    /// │         25 px         ││         25 px         │
    /// └───────────────────────┘└───────────────────────┘
    /// ```
    ///
    /// `[Ratio(1, 4) ; 4]`
    ///
    /// ```plain
    /// ┌───────────┐┌──────────┐┌───────────┐┌──────────┐
    /// │   13 px   ││   12 px  ││   13 px   ││   12 px  │
    /// └───────────┘└──────────┘└───────────┘└──────────┘
    /// ```
    Ratio(u32, u32),

    /// Applies the scaling factor proportional to all other [`Constraint::Fill`] elements
    /// to fill excess space
    ///
    /// The element will only expand or fill into excess available space, proportionally matching
    /// other [`Constraint::Fill`] elements while satisfying all other constraints.
    ///
    /// # Examples
    ///
    ///
    /// `[Fill(1), Fill(2), Fill(3)]`
    ///
    /// ```plain
    /// ┌──────┐┌───────────────┐┌───────────────────────┐
    /// │ 8 px ││     17 px     ││         25 px         │
    /// └──────┘└───────────────┘└───────────────────────┘
    /// ```
    ///
    /// `[Fill(1), Percentage(50), Fill(1)]`
    ///
    /// ```plain
    /// ┌───────────┐┌───────────────────────┐┌──────────┐
    /// │   13 px   ││         25 px         ││   12 px  │
    /// └───────────┘└───────────────────────┘└──────────┘
    /// ```
    Fill(u16),
}

impl Constraint {
    pub const fn is_min(&self) -> bool {
        matches!(self, Constraint::Min(_))
    }

    pub const fn is_fill(&self) -> bool {
        matches!(self, Constraint::Fill(_))
    }
}

impl Default for Constraint {
    fn default() -> Self {
        Self::Fill(1)
    }
}

impl std::fmt::Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Percentage(p) => write!(f, "Percentage({p})"),
            Self::Ratio(n, d) => write!(f, "Ratio({n}, {d})"),
            Self::Length(l) => write!(f, "Length({l})"),
            Self::Fill(l) => write!(f, "Fill({l})"),
            Self::Max(m) => write!(f, "Max({m})"),
            Self::Min(m) => write!(f, "Min({m})"),
        }
    }
}

impl From<u16> for Constraint {
    /// Convert a `u16` into a [`Constraint::Length`]
    ///
    /// This is useful when you want to specify a fixed size for a layout, but don't want to
    /// explicitly create a [`Constraint::Length`] yourself.
    fn from(length: u16) -> Self {
        Self::Length(length)
    }
}

impl From<&Self> for Constraint {
    fn from(constraint: &Self) -> Self {
        *constraint
    }
}

impl AsRef<Self> for Constraint {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Orientation {
    Horizontal,
    #[default]
    Vertical,
}



// ================================================================================================
