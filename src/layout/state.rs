//! Size cache for the flex containers.
//!
//! The cache is keyed by the container's persistent [`Id`] and holds one natural size per item, so a
//! layout can be placed exactly on every frame after the one that measured it.

use super::solver::{Column, ItemSpec};
use egui::{Context, Id, Vec2};

/// The natural sizes a container measured on an earlier frame, plus the specs they belong to.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FlexState {
   /// One spec per item, in the order the host added them.
   pub specs: Vec<ItemSpec>,
   /// The natural size measured for each item.
   pub sizes: Vec<Vec2>,
   /// The space the sizes were measured in.
   pub avail: Vec2,
}

impl FlexState {
   pub(crate) fn load(ctx: &Context, id: Id) -> Option<Self> {
      ctx.data(|d| d.get_temp::<Self>(id))
   }

   pub(crate) fn store(&self, ctx: &Context, id: Id) {
      ctx.data_mut(|d| d.insert_temp(id, self.clone()));
   }

   /// True when these sizes were measured in the same space we have available now.
   pub(crate) fn same_space(&self, avail: Vec2) -> bool {
      (self.avail.x - avail.x).abs() < 0.5 && (self.avail.y - avail.y).abs() < 0.5
   }
}

/// The column widths and uniform row height a table measured on an earlier frame.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TableState {
   pub columns: Vec<Column>,
   pub widths: Vec<f32>,
   /// How many body rows the state was measured with.
   pub row_count: usize,
   /// The tallest measured row: the height every row is placed at unless the host pins one.
   pub row_height: f32,
   pub header_height: f32,
   pub key: u64,
   pub avail: egui::Vec2,
}

impl TableState {
   pub(crate) fn load(ctx: &Context, id: Id) -> Option<Self> {
      ctx.data(|d| d.get_temp::<Self>(id))
   }

   pub(crate) fn store(&self, ctx: &Context, id: Id) {
      ctx.data_mut(|d| d.insert_temp(id, self.clone()));
   }

   /// True when these sizes were measured in the same space we have available now.
   pub(crate) fn same_space(&self, avail: Vec2) -> bool {
      (self.avail.x - avail.x).abs() < 0.5 && (self.avail.y - avail.y).abs() < 0.5
   }
}
