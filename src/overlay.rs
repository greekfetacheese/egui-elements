use egui::{Color32, Context, Id, LayerId, Order, Rect};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, Default)]
pub struct OverlayManager(Arc<RwLock<OverlayCounter>>);

impl OverlayManager {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(OverlayCounter::new())))
    }

    pub fn tint_0(&self) -> Color32 {
        Color32::from_black_alpha(40)
    }

    pub fn tint_1(&self) -> Color32 {
        Color32::from_black_alpha(60)
    }

    pub fn tint_2(&self) -> Color32 {
        Color32::from_black_alpha(80)
    }

    pub fn tint_3(&self) -> Color32 {
        Color32::from_black_alpha(100)
    }

    pub fn counter(&self) -> u8 {
        self.0.read().unwrap().counter()
    }

    pub fn order(&self) -> Order {
        self.0.read().unwrap().order()
    }

    pub fn paint_background(&self) {
        self.0.write().unwrap().paint_background()
    }

    pub fn paint_middle(&self) {
        self.0.write().unwrap().paint_middle()
    }

    pub fn paint_foreground(&self) {
        self.0.write().unwrap().paint_foreground()
    }

    pub fn paint_tooltip(&self) {
        self.0.write().unwrap().paint_tooltip()
    }

    pub fn paint_debug(&self) {
        self.0.write().unwrap().paint_debug()
    }

    /// Call this when you open a window
    pub fn window_opened(&self) {
        self.0.write().unwrap().window_opened();
    }

    /// Call this when you close a window
    pub fn window_closed(&self) {
        self.0.write().unwrap().window_closed();
    }

    pub fn recommended_order(&self) -> Order {
        self.0.read().unwrap().recommended_order()
    }

    pub fn calculate_alpha(&self) -> u8 {
        self.0.read().unwrap().calculate_alpha()
    }

    /// Returns the tint color based on the counter
    pub fn overlay_tint(&self) -> Color32 {
        self.0.read().unwrap().overlay_tint()
    }

    /// Paints a full-screen darkening overlay up to Foreground layer if needed
    ///
    /// If `recommend_order` is true, it will choose an order based on the counter
    pub fn paint_overlay(&self, ctx: &Context, recommend_order: bool) {
        self.0.read().unwrap().paint_overlay(ctx, recommend_order);
    }

    /// Paints an overlay at a specific screen position
    pub fn paint_overlay_at(&self, ctx: &Context, rect: Rect, order: Order, id: Id, tint: Color32) {
        self.0
            .read()
            .unwrap()
            .paint_overlay_at(ctx, rect, order, id, tint);
    }
}

#[derive(Clone, Debug)]
struct OverlayCounter {
    counter: u8,
    order: Order,
}

impl Default for OverlayCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayCounter {
    pub fn new() -> Self {
        Self {
            counter: 0,
            order: Order::Background,
        }
    }

    pub fn counter(&self) -> u8 {
        self.counter
    }

    pub fn order(&self) -> Order {
        self.order
    }

    fn paint_background(&mut self) {
        self.order = Order::Background;
    }

    fn paint_middle(&mut self) {
        self.order = Order::Middle;
    }

    fn paint_foreground(&mut self) {
        self.order = Order::Foreground;
    }

    fn paint_tooltip(&mut self) {
        self.order = Order::Tooltip;
    }

    fn paint_debug(&mut self) {
        self.order = Order::Debug;
    }

    fn window_opened(&mut self) {
        self.counter += 1;
    }

    fn window_closed(&mut self) {
        if self.counter > 0 {
            self.counter -= 1;
        }
    }

    fn calculate_alpha(&self) -> u8 {
        let counter = self.counter;

        if counter == 0 {
            return 0;
        }

        if counter > 3 {
            return 220;
        }

        let mut a = 80;
        for _ in 1..counter {
            a += 40;
        }

        a
    }

    fn overlay_tint(&self) -> Color32 {
        let counter = self.counter();

        if counter == 1 {
            return Color32::from_black_alpha(80);
        }

        let alpha = self.calculate_alpha();
        Color32::from_black_alpha(alpha)
    }

    fn recommended_order(&self) -> Order {
        if self.counter() == 1 {
            Order::Background
        } else if self.counter() == 2 {
            Order::Middle
        } else {
            Order::Foreground
        }
    }

    fn paint_overlay(&self, ctx: &Context, recommend_order: bool) {
        let counter = self.counter();
        if counter == 0 {
            return;
        }

        let order = if recommend_order {
            self.recommended_order()
        } else {
            self.order()
        };

        let layer_id = LayerId::new(order, Id::new("darkening_overlay"));

        let painter = ctx.layer_painter(layer_id);
        painter.rect_filled(ctx.content_rect(), 0.0, self.overlay_tint());
    }

    pub fn paint_overlay_at(&self, ctx: &Context, rect: Rect, order: Order, id: Id, tint: Color32) {
        let layer_id = LayerId::new(order, id);

        let painter = ctx.layer_painter(layer_id);
        painter.rect_filled(rect, 0.0, tint);
    }
}
