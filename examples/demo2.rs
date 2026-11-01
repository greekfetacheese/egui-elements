use eframe::egui::*;
use egui_elements::widgets::Button;
use egui_elements::{Theme, ThemeKind};

struct DemoApp {}

impl DemoApp {
   fn new(cc: &eframe::CreationContext<'_>) -> Self {
      let mut theme = Theme::new(ThemeKind::TokyoNight);
      theme.install(&cc.egui_ctx);

      Self {}
   }
}

impl eframe::App for DemoApp {
   fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
      let theme = Theme::current(ui.ctx());
      let bg = theme.colors.bg;

      CentralPanel::default()
         .frame(Frame::new().fill(bg).inner_margin(Margin::same(10)))
         .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(16.0, 8.0);
            ui.spacing_mut().button_padding = vec2(6.0, 8.0);

            // Bad
            ui.label("Bad");
            ui.horizontal(|ui| {
               ui.label("Normal text");

               let btn_text = "Click me";
               let button = Button::new(btn_text);

               ui.add(button);
            });

            // Good
            let ui_size = vec2(100.0, 30.0);

            ui.label("Good");
            ui.allocate_ui(ui_size, |ui| {
               ui.horizontal_centered(|ui| {
                  ui.label("Normal text");

                  let btn_text = "Click me";
                  let button = Button::new(btn_text);

                  ui.add(button);
               });
            });

            // Without the allocate_ui this breaks
            ui.label("This breaks the ui causing massive Y space");
            ui.horizontal_centered(|ui| {
               ui.label("Normal text");

               let btn_text = "Click me";
               let button = Button::new(btn_text);

               ui.add(button);
            });
         });
   }
}

fn main() -> eframe::Result {
   let options = eframe::NativeOptions {
      viewport: ViewportBuilder::default()
         .with_title("egui-elements demo")
         .with_inner_size([450.0, 300.0])
         .with_resizable(true),
      ..Default::default()
   };

   eframe::run_native(
      "egui-elements demo",
      options,
      Box::new(|cc| {
         egui_extras::install_image_loaders(&cc.egui_ctx);
         Ok(Box::new(DemoApp::new(cc)))
      }),
   )
}
