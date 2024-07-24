//! Animations



use std::{cell::RefCell, rc::Rc, time::Duration};

use rand::{Rng as _, SeedableRng as _};

use crate::prelude::*;



// ================================================================================================



pub struct Animation {
    effect: Box<dyn Effect>,
}

impl Effect for Animation {
    fn process(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) -> Option<Duration> {
        let area = self.effect.area().unwrap_or(area);
        self.effect.process(duration, buf, area)
    }

    fn execute(&mut self, alpha: f32, area: Rect, cell_iter: CellIterator){
        self.effect.execute(alpha, area, cell_iter);
    }

    fn timer_mut(&mut self) -> Option<&mut AnimationTimer> {
        self.effect.timer_mut()
    }

    fn cell_filter(&self) -> Option<CellFilter> {
        self.effect.cell_filter()
    }

    fn set_cell_filter(&mut self, strategy: CellFilter) {
        self.effect.set_cell_filter(strategy)
    }

    fn done(&self) -> bool {
        self.effect.done()
    }

    fn running(&self) -> bool {
        self.effect.running()
    }

    fn clone_box(&self) -> Box<dyn Effect> {
        self.effect.clone_box()
    }

    fn area(&self) -> Option<Rect> {
        self.effect.area()
    }

    fn set_area(&mut self, area: Rect) {
        self.effect.set_area(area)
    }
}

impl Animation {
    pub fn new<S>(effect: S) -> Self
        where S: Effect + 'static
    {
        Self { effect: Box::new(effect) }
    }
}

#[derive(Clone, Copy, Default)]
pub struct AnimationTimer {
    remaining: Duration,
    total: Duration,
    interpolation: Interpolation,
    reverse: bool
}

impl AnimationTimer {
    pub fn new(duration: Duration) -> Self {
        Self {
            remaining: duration,
            total: duration,
            interpolation: Interpolation::Linear,
            reverse: false
        }
    }

    pub fn process(&mut self, duration: Duration) -> Option<Duration> {
        if self.remaining >= duration {
            self.remaining -= duration;
            None
        } else {
            let overflow = duration - self.remaining;
            self.remaining = Duration::ZERO;
            Some(overflow)
        }
    }

    pub fn alpha(&self) -> f32 {
        let total = self.total.as_secs_f32();
        if total == 0.0 {
            return 1.0;
        }

        let remaining = self.remaining.as_secs_f32();
        let inv_alpha = remaining / total;

        let a = if self.reverse { inv_alpha } else { 1.0 - inv_alpha };
        self.interpolation.alpha(a)
    }
    
    pub fn done(&self) -> bool {
        self.remaining.is_zero()
    }

    pub fn reversed(self) -> Self {
        Self { reverse: !self.reverse, ..self }
    }
}


// ================================================================================================



pub trait Effect {
    fn process(
        &mut self,
        duration: Duration,
        buf: &mut Buffer,
        area: Rect,
    ) -> Option<Duration> {
        let (overflow, alpha) = self.timer_mut()
            .map(|t| (t.process(duration), t.alpha()))
            .unwrap_or((None, 1.0));

        let requested_cells = self.cell_iter(buf, area);
        self.execute(alpha, area, requested_cells);

        overflow
    }

    /// Executes the effect with the given alpha value and cells. This is where
    /// the actual effect logic should be implemented.
    ///
    /// # Arguments
    /// * `alpha` - The alpha value indicating the progress of the effect animation.
    /// * `area` - The rectangular area within the buffer where the effect will be applied.
    /// * `cell_iter` - An iterator over the cells in the specified area.
    fn execute(
        &mut self,
        alpha: f32,
        area: Rect,
        cell_iter: CellIterator,
    );

    fn cell_iter<'a>(
        &mut self,
        buf: &'a mut Buffer,
        area: Rect,
    ) -> CellIterator<'a> {
        CellIterator::new(buf, area, self.cell_filter())
    }

    fn timer_mut(&mut self) -> Option<&mut AnimationTimer> { None }

    fn cell_filter(&self) -> Option<CellFilter> { None }

    fn set_cell_filter(&mut self, strategy: CellFilter);

    fn done(&self) -> bool;

    fn running(&self) -> bool { !self.done() }

    fn clone_box(&self) -> Box<dyn Effect>;

    fn area(&self) -> Option<Rect>;

    fn set_area(&mut self, area: Rect);
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Interpolation {
    #[default]
    Linear,
}

impl Interpolation {
    pub fn alpha(&self, a: f32) -> f32 {
        match self {
            Interpolation::Linear => a,
        }
    }
}


// ================================================================================================


pub struct CellIterator<'a> {
    current: u16,
    area: Rect,
    buf: &'a mut Buffer,
    filter: Option<CellFilter>,
}

impl<'a> CellIterator<'a> {
    pub fn new(
        buf: &'a mut Buffer,
        area: Rect,
        filter: Option<CellFilter>,
    ) -> Self {
        Self { current: 0, area, buf, filter }
    }
    
    fn cell_mut(&mut self) -> (Pos, &mut Cell) {
        let x = self.current % self.area.width;
        let y = self.current / self.area.width;

        let pos = Pos::new(self.area.x + x, self.area.y + y);
        let cell = self.buf.get_mut(pos.x(), pos.y());
        (pos, cell)
    }
}

impl<'a> Iterator for CellIterator<'a> {
    type Item = (Pos, &'a mut Cell);

    fn next(&mut self) -> Option<Self::Item> {
        let selector = self.filter.as_ref().map(|f| f.selector(self.area));
        while self.current < self.area.area() {
            let (pos, cell) = self.cell_mut();
            // enforce cell's lifetime. this is safe because `buf` is guaranteed to outlive `'a`
            let cell: &'a mut Cell = unsafe { std::mem::transmute(cell) };
            self.current += 1;

            if let Some(filter) = &selector {
                if filter.is_valid(pos, cell) {
                    return Some((pos, cell));
                }
            } else {
                return Some((pos, cell));
            }
        }

        None
    }
}

/// A filter mode enables effects to operate on specific cells.
#[derive(Clone, Default)]
pub enum CellFilter {
    /// Selects every cell
    #[default]
    All,
    /// Selects cells with matching foreground color
    FgColor(Color),
    /// Selects cells with matching background color
    BgColor(Color),
    /// Selects cells within the inner margin of the area
    Inner(Margin),
    /// Selects cells outside the inner margin of the area
    Outer(Margin),
    /// Selects cells with text
    Text,
    /// Selects cells that match all the given filters
    AllOf(Vec<CellFilter>),
    /// Selects cells that match any of the given filters
    AnyOf(Vec<CellFilter>),
    /// Selects cells that do not match any of the given filters
    NoneOf(Vec<CellFilter>),
    /// Negates the given filter
    Not(Box<CellFilter>),
    /// Selects cells within the specified layout, denoted by the index
    Layout(Layout, u16),
    /// Selects cells by predicate function
    PositionFn(Rc<RefCell<dyn Fn(Pos) -> bool>>),
}

impl CellFilter {
    pub fn position_fn<F>(f: F) -> Self
        where F: Fn(Pos) -> bool + 'static
    {
        CellFilter::PositionFn(Rc::new(RefCell::new(f)))
    }

    pub fn selector(&self, area: Rect) -> CellSelector {
        CellSelector::new(area, self.clone())
    }
}

pub struct CellSelector {
    inner_area: Rect,
    strategy: CellFilter,
}

impl CellSelector {
    fn new(area: Rect, strategy: CellFilter) -> Self {
        let inner_area = Self::resolve_area(area, &strategy);

        Self { inner_area, strategy }
    }

    fn resolve_area(area: Rect, mode: &CellFilter) -> Rect {
        match mode {
            CellFilter::All                  => area,
            CellFilter::Inner(margin)        => area.inner(*margin),
            CellFilter::Outer(margin)        => area.inner(*margin),
            CellFilter::Text                 => area,
            CellFilter::AllOf(_)             => area,
            CellFilter::AnyOf(_)             => area,
            CellFilter::NoneOf(_)            => area,
            CellFilter::Not(m)               => Self::resolve_area(area, m.as_ref()),
            CellFilter::FgColor(_)           => area,
            CellFilter::BgColor(_)           => area,
            CellFilter::Layout(layout, idx)  => layout.split(area)[*idx as usize],
            CellFilter::PositionFn(_)        => area,
        }
    }

    pub fn is_valid(&self, pos: Pos, cell: &Cell) -> bool {
        let mode = &self.strategy;

        self.valid_position(pos, mode)
            && self.is_valid_cell(cell, mode)
    }

    fn valid_position(&self, pos: Pos, mode: &CellFilter) -> bool {
        match mode {
            CellFilter::All           => self.inner_area.contains(pos),
            CellFilter::Layout(_, _)  => self.inner_area.contains(pos),
            CellFilter::Inner(_)      => self.inner_area.contains(pos),
            CellFilter::Outer(_)      => !self.inner_area.contains(pos),
            CellFilter::Text          => self.inner_area.contains(pos),
            CellFilter::AllOf(s)      => s.iter()
                .all(|mode| mode.selector(self.inner_area).valid_position(pos, mode)),
            CellFilter::AnyOf(s)      => s.iter()
                .any(|mode| mode.selector(self.inner_area).valid_position(pos, mode)),
            CellFilter::NoneOf(s)     => s.iter()
                .all(|mode| !mode.selector(self.inner_area).valid_position(pos, mode)),
            CellFilter::Not(m)        => self.valid_position(pos, m.as_ref()),
            CellFilter::FgColor(_)    => self.inner_area.contains(pos),
            CellFilter::BgColor(_)    => self.inner_area.contains(pos),
            CellFilter::PositionFn(f) => f.borrow()(pos),
        }
    }

    fn is_valid_cell(&self, cell: &Cell, mode: &CellFilter) -> bool {
        match mode {
            CellFilter::Text => {
                if cell.symbol().len() == 1 {
                    let ch = cell.symbol().chars().next().unwrap();
                    ch.is_alphabetic() || ch.is_numeric() || ch == ' ' || "?!.,:;".contains(ch)
                } else {
                    false
                }
            },

            CellFilter::AllOf(s) => {
                s.iter()
                    .all(|s| s.selector(self.inner_area).is_valid_cell(cell, s))
            },

            CellFilter::FgColor(color) => cell.fg == *color,
            CellFilter::BgColor(color) => cell.bg == *color,

            CellFilter::Not(m) => !self.is_valid_cell(cell, m.as_ref()),

            _ => true,
        }
    }
}



// ================================================================================================



#[derive(Clone)]
pub struct Dissolve {
    timer: AnimationTimer,
    cyclic_cell_activation: Vec<f32>,
    area: Option<Rect>,
    cell_filter: CellFilter,
}

impl Dissolve {
    pub fn new(
        lifetime: AnimationTimer,
        cell_cycle: usize,
    ) -> Self {
        let mut rng = rand::rngs::SmallRng::from_rng(rand::thread_rng()).unwrap();

        Self {
            timer: lifetime,
            cyclic_cell_activation: (0..cell_cycle).map(|_| rng.gen_range(0.0..1.0)).collect(),
            area: None,
            cell_filter: CellFilter::All,
        }
    }

    fn is_cell_idx_active(&self, idx: usize, a: f32) -> bool {
        a > self.cyclic_cell_activation[idx % self.cyclic_cell_activation.len()]
    }
}

impl Effect for Dissolve {
    fn execute(&mut self, alpha: f32, _area: Rect, cell_iter: CellIterator) {
        cell_iter.enumerate()
            .filter(|(idx, _)| self.is_cell_idx_active(*idx, alpha))
            .for_each(|(_, (_, c))| { c.set_char(' '); });
    }

    fn done(&self) -> bool {
          self.timer.done()
     }

     fn clone_box(&self) -> Box<dyn Effect> {
          Box::new(self.clone())
     }

    fn area(&self) -> Option<Rect> {
        self.area
    }

    fn set_area(&mut self, area: Rect) {
        self.area = Some(area)
    }

    fn timer_mut(&mut self) -> Option<&mut AnimationTimer> {
        Some(&mut self.timer)
    }

    fn cell_filter(&self) -> Option<CellFilter> {
        Some(self.cell_filter.clone())
    }

    fn set_cell_filter(&mut self, strategy: CellFilter) {
        self.cell_filter = strategy
    }
}

pub fn dissolve<T: Into<AnimationTimer>>(cycle_len: usize, timer: T) -> Animation {
    Animation::new(Dissolve::new(timer.into(), cycle_len))
}

pub fn coalesce<T: Into<AnimationTimer>>(cycle_len: usize, timer: T) -> Animation {
    Animation::new(Dissolve::new(timer.into().reversed(), cycle_len))
}



// ================================================================================================
