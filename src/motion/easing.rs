//! Timing curves for [`Motion`](super::Motion).

/// The curve an animation follows over its duration.
///
/// The cubic family is what most web design systems use; [`Easing::Custom`] takes any `fn(f32) -> f32`, so
/// `egui::emath::easing::exponential_out` or a project's own curve works without a new variant.
#[derive(Clone, Copy, Debug)]
pub enum Easing {
   /// Constant speed.
   Linear,
   /// Slow start, fast end.
   EaseIn,
   /// Fast start, slow end. The default: it reads as "the UI responded" instead of "something moved".
   EaseOut,
   /// Slow at both ends.
   EaseInOut,
   /// Any `fn(f32) -> f32` mapping `0..=1` to `0..=1`.
   Custom(fn(f32) -> f32),
}

impl PartialEq for Easing {
   /// Curves are equal when they are the same variant — except [`Easing::Custom`], where the function pointers
   /// are compared with [`std::ptr::fn_addr_eq`] (a plain `==` on `fn` pointers is a lint: addresses are not
   /// guaranteed unique across codegen units).
   fn eq(&self, other: &Self) -> bool {
      match (self, other) {
         (Self::Custom(a), Self::Custom(b)) => std::ptr::fn_addr_eq(*a, *b),
         (Self::Custom(_), _) | (_, Self::Custom(_)) => false,
         (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
      }
   }
}

impl Easing {
   /// Map a normalized `0..=1` progress onto the curve. Input is clamped.
   pub fn apply(self, t: f32) -> f32 {
      let t = t.clamp(0.0, 1.0);
      match self {
         Self::Linear => t,
         Self::EaseIn => egui::emath::easing::cubic_in(t),
         Self::EaseOut => egui::emath::easing::cubic_out(t),
         Self::EaseInOut => egui::emath::easing::cubic_in_out(t),
         Self::Custom(f) => f(t),
      }
   }
}

impl Default for Easing {
   fn default() -> Self {
      Self::EaseOut
   }
}
