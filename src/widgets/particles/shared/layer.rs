//! Layer selection for the particle effects.

use egui::{Context, Id, LayerId, Order, Rect};

/// Layer a particle effect is painted on.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleLayer {
   /// Behind everything (`Order::Background`).
   Background,
   /// In front of windows (`Order::Foreground`).
   Foreground,
   /// On top of everything including debug/tooltips (`Order::Debug`).
   Top,
}

impl ParticleLayer {
   /// Human-readable name, matching the settings window labels.
   pub fn to_str(&self) -> &str {
      match self {
         ParticleLayer::Background => "Background",
         ParticleLayer::Foreground => "Foreground",
         ParticleLayer::Top => "Top",
      }
   }
}

/// Resolves the concrete layer and area an effect paints into.
///
/// `order` is the already-resolved [`ParticleLayer`] order; `custom_layer` wins
/// over it and `custom_area` wins over `ctx.content_rect()`. Returns `None` for
/// a degenerate (≤ 0) area so the caller can bail out before touching any state.
pub(crate) fn resolve_target(
   ctx: &Context,
   id: Id,
   order: Order,
   custom_layer: Option<LayerId>,
   custom_area: Option<Rect>,
) -> Option<(LayerId, Rect)> {
   let area = custom_area.unwrap_or_else(|| ctx.content_rect());
   if area.width() <= 0.0 || area.height() <= 0.0 {
      return None;
   }

   let layer_id = custom_layer.unwrap_or_else(|| LayerId::new(order, id));

   Some((layer_id, area))
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn particle_layer_to_str() {
      assert_eq!(ParticleLayer::Background.to_str(), "Background");
      assert_eq!(ParticleLayer::Foreground.to_str(), "Foreground");
      assert_eq!(ParticleLayer::Top.to_str(), "Top");
   }
}
