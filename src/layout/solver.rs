use egui::{Pos2, Rect, Vec2, pos2, vec2};

/// Flex direction: `row` places items left-to-right, `column` top-to-bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
   Row,
   Column,
}

impl Axis {
   /// The main-axis component of `size`.
   pub fn main_of(self, size: Vec2) -> f32 {
      match self {
         Self::Row => size.x,
         Self::Column => size.y,
      }
   }

   /// The cross-axis component of `size`.
   pub fn cross_of(self, size: Vec2) -> f32 {
      match self {
         Self::Row => size.y,
         Self::Column => size.x,
      }
   }

   /// A size built from a main extent and a cross extent.
   pub fn size(self, main: f32, cross: f32) -> Vec2 {
      match self {
         Self::Row => vec2(main, cross),
         Self::Column => vec2(cross, main),
      }
   }

   /// A position built from a main offset and a cross offset.
   fn pos(self, main: f32, cross: f32) -> Pos2 {
      match self {
         Self::Row => pos2(main, cross),
         Self::Column => pos2(cross, main),
      }
   }
}

/// `align-items` / `align-self`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossAlign {
   Start,
   Center,
   End,
   Stretch,
}

/// `justify-content`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Justify {
   Start,
   Center,
   End,
   SpaceBetween,
   SpaceAround,
}

/// How one item asks for main-axis space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ItemSize {
   /// Content sized — the measured natural main extent.
   Auto,
   /// A fixed main extent in points.
   Fixed(f32),
   /// Basis 0, shares the free main space by weight (CSS `flex: <weight>`).
   Grow(f32),
}

/// What the solver needs to know about one item.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemSpec {
   pub size: ItemSize,
   /// `align-self` override; `None` inherits the container's `align-items`.
   pub align: Option<CrossAlign>,
}

/// The placed container: one rect per item (relative to the container's top-left) plus the
/// total size.
#[derive(Clone, Debug, PartialEq)]
pub struct LayoutOut {
   pub rects: Vec<Rect>,
   pub size: Vec2,
}

pub fn resolve(
   axis: Axis,
   items: &[ItemSpec],
   sizes: &[Vec2],
   container: Vec2,
   gap: f32,
   justify: Justify,
   align: CrossAlign,
) -> LayoutOut {
   debug_assert_eq!(items.len(), sizes.len());
   let n = items.len().min(sizes.len());
   if n == 0 {
      return LayoutOut {
         rects: vec![],
         size: axis.size(axis.main_of(container).max(0.0), 0.0),
      };
   }

   let main_avail = axis.main_of(container).max(0.0);
   let gap = gap.max(0.0);
   let gaps = gap * (n - 1) as f32;

   let mut basis_total = 0.0;
   let mut weight_total = 0.0;
   let mut grow_count = 0;
   let mut last_grow = None;
   for (i, item) in items.iter().take(n).enumerate() {
      match item.size {
         ItemSize::Auto => basis_total += axis.main_of(sizes[i]),
         ItemSize::Fixed(v) => basis_total += v.max(0.0),
         ItemSize::Grow(w) => {
            weight_total += w.max(0.0);
            grow_count += 1;
            last_grow = Some(i);
         }
      }
   }

   let free = (main_avail - basis_total - gaps).max(0.0);

   let mut lengths = Vec::with_capacity(n);
   let mut assigned = 0.0;
   for (i, item) in items.iter().take(n).enumerate() {
      let length = match item.size {
         ItemSize::Auto => axis.main_of(sizes[i]),
         ItemSize::Fixed(v) => v.max(0.0),
         ItemSize::Grow(_) => {
            if last_grow == Some(i) {
               (free - assigned).max(0.0)
            } else {
               let share = if weight_total > 0.0 {
                  match item.size {
                     ItemSize::Grow(w) => free * w.max(0.0) / weight_total,
                     _ => 0.0,
                  }
               } else {
                  free / grow_count as f32
               };
               assigned += share;
               share
            }
         }
      };
      lengths.push(length);
   }

   // The container's cross extent is a floor for the line, so a column can align across its full width.
   let line_cross = sizes[..n]
      .iter()
      .map(|s| axis.cross_of(*s))
      .fold(axis.cross_of(container).max(0.0), f32::max);

   let cross_extent = |i: usize| -> f32 {
      let effective = items[i].align.unwrap_or(align);
      if effective == CrossAlign::Stretch {
         line_cross
      } else {
         axis.cross_of(sizes[i])
      }
   };

   let used = lengths.iter().sum::<f32>() + gaps;
   let leftover = (main_avail - used).max(0.0);
   let (start, between) = match justify {
      Justify::Start => (0.0, gap),
      Justify::Center => (leftover / 2.0, gap),
      Justify::End => (leftover, gap),
      Justify::SpaceBetween => {
         if n > 1 {
            (0.0, gap + leftover / (n - 1) as f32)
         } else {
            (0.0, gap)
         }
      }
      Justify::SpaceAround => (
         leftover / (2.0 * n as f32),
         gap + leftover / n as f32,
      ),
   };

   let mut rects = Vec::with_capacity(n);
   let mut cursor = start;
   for i in 0..n {
      let extent = cross_extent(i);
      let cross_offset = match items[i].align.unwrap_or(align) {
         CrossAlign::Start | CrossAlign::Stretch => 0.0,
         CrossAlign::Center => (line_cross - extent) / 2.0,
         CrossAlign::End => line_cross - extent,
      };
      rects.push(Rect::from_min_size(
         axis.pos(cursor, cross_offset),
         axis.size(lengths[i], extent),
      ));
      cursor += lengths[i] + between;
   }

   LayoutOut {
      rects,
      size: axis.size(main_avail, line_cross),
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   fn spec(size: ItemSize) -> ItemSpec {
      ItemSpec { size, align: None }
   }

   fn sizes(v: &[(f32, f32)]) -> Vec<Vec2> {
      v.iter().map(|(x, y)| vec2(*x, *y)).collect()
   }

   fn close(a: f32, b: f32) -> bool {
      (a - b).abs() < 0.01
   }

   #[test]
   fn row_auto_and_grow_distribute_leftover_by_weight() {
      let items = [
         spec(ItemSize::Auto),
         spec(ItemSize::Grow(1.0)),
         spec(ItemSize::Grow(3.0)),
      ];
      let sizes = sizes(&[(100.0, 20.0), (50.0, 20.0), (50.0, 20.0)]);
      let out = resolve(
         Axis::Row,
         &items,
         &sizes,
         vec2(400.0, 0.0),
         10.0,
         Justify::Start,
         CrossAlign::Start,
      );

      assert!(close(out.rects[0].min.x, 0.0));
      assert!(close(out.rects[1].min.x, 110.0));
      assert!(close(out.rects[2].min.x, 190.0));
      assert!(close(out.rects[0].width(), 100.0));
      assert!(close(out.rects[1].width(), 70.0));
      assert!(close(out.rects[2].width(), 210.0));
      assert!(close(out.size.x, 400.0));
   }

   #[test]
   fn row_fixed_items_keep_their_size() {
      let items = [spec(ItemSize::Fixed(80.0)), spec(ItemSize::Fixed(80.0))];
      let sizes = sizes(&[(10.0, 20.0), (10.0, 20.0)]);
      let out = resolve(
         Axis::Row,
         &items,
         &sizes,
         vec2(400.0, 0.0),
         20.0,
         Justify::Start,
         CrossAlign::Start,
      );

      assert!(close(out.rects[0].min.x, 0.0));
      assert!(close(out.rects[1].min.x, 100.0));
      assert!(close(out.rects[0].width(), 80.0));
      assert!(close(out.rects[1].width(), 80.0));
      assert_eq!(out.size, vec2(400.0, 20.0));
   }

   #[test]
   fn gap_applies_between_items_only() {
      let items = [
         spec(ItemSize::Auto),
         spec(ItemSize::Auto),
         spec(ItemSize::Auto),
      ];
      let sizes = sizes(&[(50.0, 20.0), (50.0, 20.0), (50.0, 20.0)]);
      let out = resolve(
         Axis::Row,
         &items,
         &sizes,
         vec2(174.0, 0.0),
         8.0,
         Justify::Start,
         CrossAlign::Start,
      );

      assert!(close(out.rects[0].min.x, 0.0));
      assert!(close(out.rects[1].min.x, 58.0));
      assert!(close(out.rects[2].min.x, 116.0));
      assert!(close(out.size.x, 174.0));
   }

   #[test]
   fn justify_space_between_pushes_free_space_between_items() {
      let items = [spec(ItemSize::Fixed(50.0)), spec(ItemSize::Fixed(50.0))];
      let sizes = sizes(&[(10.0, 10.0), (10.0, 10.0)]);
      let out = resolve(
         Axis::Row,
         &items,
         &sizes,
         vec2(300.0, 0.0),
         0.0,
         Justify::SpaceBetween,
         CrossAlign::Start,
      );

      assert!(close(out.rects[0].min.x, 0.0));
      assert!(close(out.rects[1].min.x, 250.0));
   }

   #[test]
   fn justify_center_and_end_use_the_leftover() {
      let items = [spec(ItemSize::Fixed(50.0)), spec(ItemSize::Fixed(50.0))];
      let sizes = sizes(&[(10.0, 10.0), (10.0, 10.0)]);

      let centered = resolve(
         Axis::Row,
         &items,
         &sizes,
         vec2(300.0, 0.0),
         0.0,
         Justify::Center,
         CrossAlign::Start,
      );
      assert!(close(centered.rects[0].min.x, 100.0));
      assert!(close(centered.rects[1].min.x, 150.0));

      let ended = resolve(
         Axis::Row,
         &items,
         &sizes,
         vec2(300.0, 0.0),
         0.0,
         Justify::End,
         CrossAlign::Start,
      );
      assert!(close(ended.rects[0].min.x, 200.0));
      assert!(close(ended.rects[1].min.x, 250.0));
   }

   #[test]
   fn align_center_centers_the_shorter_item_on_the_cross_axis() {
      let items = [spec(ItemSize::Auto), spec(ItemSize::Auto)];
      let sizes = sizes(&[(100.0, 20.0), (100.0, 40.0)]);

      let centered = resolve(
         Axis::Row,
         &items,
         &sizes,
         vec2(400.0, 0.0),
         0.0,
         Justify::Start,
         CrossAlign::Center,
      );
      assert!(close(centered.rects[0].min.y, 10.0));
      assert!(close(centered.rects[1].min.y, 0.0));
      assert!(close(centered.size.y, 40.0));

      let started = resolve(
         Axis::Row,
         &items,
         &sizes,
         vec2(400.0, 0.0),
         0.0,
         Justify::Start,
         CrossAlign::Start,
      );
      assert!(close(started.rects[0].min.y, 0.0));
      assert!(close(started.rects[1].min.y, 0.0));

      let stretched = resolve(
         Axis::Row,
         &items,
         &sizes,
         vec2(400.0, 0.0),
         0.0,
         Justify::Start,
         CrossAlign::Stretch,
      );
      assert!(close(stretched.rects[0].height(), 40.0));
      assert!(close(stretched.rects[1].height(), 40.0));
   }

   #[test]
   fn overflowing_items_never_produce_negative_sizes() {
      let items = [
         spec(ItemSize::Fixed(200.0)),
         spec(ItemSize::Fixed(200.0)),
         spec(ItemSize::Fixed(200.0)),
      ];
      let fixed_sizes = sizes(&[(10.0, 10.0), (10.0, 10.0), (10.0, 10.0)]);
      let out = resolve(
         Axis::Row,
         &items,
         &fixed_sizes,
         vec2(400.0, 0.0),
         10.0,
         Justify::Start,
         CrossAlign::Start,
      );

      assert!(close(out.rects[0].width(), 200.0));
      assert!(close(out.rects[1].width(), 200.0));
      assert!(close(out.rects[2].width(), 200.0));
      assert!(close(out.rects[0].min.x, 0.0));
      assert!(close(out.rects[1].min.x, 210.0));
      assert!(close(out.rects[2].min.x, 420.0));
      assert!(close(out.size.x, 400.0));

      let items = [
         spec(ItemSize::Fixed(200.0)),
         spec(ItemSize::Fixed(200.0)),
         spec(ItemSize::Grow(1.0)),
      ];
      let grow_sizes = sizes(&[(10.0, 10.0), (10.0, 10.0), (10.0, 10.0)]);
      let out = resolve(
         Axis::Row,
         &items,
         &grow_sizes,
         vec2(300.0, 0.0),
         0.0,
         Justify::Start,
         CrossAlign::Start,
      );

      assert!(close(out.rects[2].width(), 0.0));
   }
}
