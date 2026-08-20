use crate::theme::Theme;
use crate::utils::TINT_1;
use crate::widgets::{
    Button,
    secure_text_edit::{SecureTextEdit, SecureTextEditOutput},
};
use egui::*;

#[cfg(all(feature = "qr-scanner", target_os = "linux"))]
use super::QRScanner;

use secure_types::SecureString;

const VISIBLE_BLACK: ImageSource<'_> = include_image!("../../assets/visible-black.png");
const VISIBLE_WHITE: ImageSource<'_> = include_image!("../../assets/visible-white.png");
const INVISIBLE_BLACK: ImageSource<'_> = include_image!("../../assets/invisible-black.png");
const INVISIBLE_WHITE: ImageSource<'_> = include_image!("../../assets/invisible-white.png");
const QR_CODE_BLACK: ImageSource<'_> = include_image!("../../assets/qr-code-black.png");
const QR_CODE_WHITE: ImageSource<'_> = include_image!("../../assets/qr-code-white.png");

/// A secure input field that can be used to edit a text containing sensitive information.
#[derive(Clone)]
pub struct SecureInputField {
    open: bool,
    pub(crate) text: SecureString,
    text_hidden: bool,
    id: &'static str,
    icon_size: Vec2,
    min_size: Vec2,
    inner_margin: Option<Margin>,
    #[cfg(all(feature = "qr-scanner", target_os = "linux"))]
    qr_scanner: QRScanner,
    qr_enabled: bool,
}

impl SecureInputField {
    /// Create a new secure input field.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the input field, this will also be used as the name of the input field
    /// * `text_hidden` - Whether the text is masked
    /// * `open` - Whether the input field is open
    ///
    /// # Panics
    ///
    /// If the `SecureString` allocation fails.
    pub fn new(id: &'static str, text_hidden: bool, open: bool) -> Self {
        Self {
            open,
            text: SecureString::new_with_capacity(32).unwrap(),
            text_hidden,
            id,
            icon_size: vec2(20.0, 20.0),
            min_size: vec2(300.0, 20.0),
            inner_margin: None,
            #[cfg(all(feature = "qr-scanner", target_os = "linux"))]
            qr_scanner: QRScanner::new(),
            qr_enabled: true,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Erase the text from memory
    pub fn erase(&mut self) {
        self.text.erase();
    }

    pub fn icon_size(mut self, size: Vec2) -> Self {
        self.icon_size = size;
        self
    }

    pub fn min_size(mut self, size: Vec2) -> Self {
        self.min_size = size;
        self
    }

    pub fn inner_margin(mut self, margin: Margin) -> Self {
        self.inner_margin = Some(margin);
        self
    }

    pub fn qr_enabled(mut self, enabled: bool) -> Self {
        self.qr_enabled = enabled;
        self
    }

    pub fn is_text_hidden(&self) -> bool {
        self.text_hidden
    }

    /// Return a clone of the text
    pub fn text(&self) -> SecureString {
        self.text.clone()
    }

    /// Set the id of this input field
    pub fn set_id(&mut self, id: &'static str) {
        self.id = id;
    }

    /// Set whether this input field is hidden
    pub fn set_text_hidden(&mut self, text_hidden: bool) {
        self.text_hidden = text_hidden;
    }

    /// Set the text of this input field
    pub fn set_text(&mut self, text: SecureString) {
        self.text = text;
    }

    /// Set the minimum size of this input field
    pub fn set_min_size(&mut self, size: Vec2) {
        self.min_size = size;
    }

    /// Set the icon size of this input field
    pub fn set_icon_size(&mut self, size: Vec2) {
        self.icon_size = size;
    }

    /// Set the inner margin of this input field
    pub fn set_inner_margin(&mut self, margin: Margin) {
        self.inner_margin = Some(margin);
    }

    /// Enable the QR scanner
    pub fn enable_qr_scanner(&mut self) {
        self.qr_enabled = true;
    }

    /// Disable the QR scanner
    pub fn disable_qr_scanner(&mut self) {
        self.qr_enabled = false;
    }

    /// Show this input field
    ///
    /// # Returns
    /// `SecureTextEditOutput`
    pub fn show(&mut self, ui: &mut Ui) -> Option<SecureTextEditOutput> {
        if !self.open {
            return None;
        }

        let ui_size = self.min_size;
        let theme = Theme::current(ui.ctx());

        let mut hidden = self.is_text_hidden();
        let field_name = self.id.to_string();
        let img_size = self.icon_size;

        ui.label(RichText::new(field_name).size(theme.typography.large));

        let response = self.text.secure_mut(|text_str| {
            let margin = self.inner_margin.unwrap_or_else(|| {
                Margin::same(theme.inner_margin)
            });

            let row_height = (theme.typography.normal + margin.sum().y).max(ui_size.y);
            let row_size = vec2(ui_size.x, row_height);

            let text_edit = SecureTextEdit::singleline(text_str)
                .min_size(ui_size)
                .margin(margin)
                .password(hidden)
                .font(FontId::proportional(theme.typography.normal));

            let response =
                ui.allocate_ui_with_layout(row_size, Layout::left_to_right(Align::Center), |ui| {
                    let output = text_edit.show(ui);

                    let img_source = if hidden {
                        match theme.dark {
                            true => INVISIBLE_WHITE,
                            false => INVISIBLE_BLACK,
                        }
                    } else {
                        match theme.dark {
                            true => VISIBLE_WHITE,
                            false => VISIBLE_BLACK,
                        }
                    };

                    let img = if theme.image_tint_recommended {
                        Image::new(img_source)
                            .tint(TINT_1)
                            .fit_to_exact_size(img_size)
                    } else {
                        Image::new(img_source).fit_to_exact_size(img_size)
                    };

                    let button = Button::image(img);
                    if ui.add(button).clicked() {
                        hidden = !hidden;
                    }

                    #[cfg(all(feature = "qr-scanner", target_os = "linux"))]
                    {
                        if self.qr_enabled {
                            let img_source = match theme.dark {
                                true => QR_CODE_WHITE,
                                false => QR_CODE_BLACK,
                            };

                            let img = if theme.image_tint_recommended {
                                Image::new(img_source)
                                    .tint(TINT_1)
                                    .fit_to_exact_size(img_size)
                            } else {
                                Image::new(img_source).fit_to_exact_size(img_size)
                            };

                            let button = Button::image(img);
                            if ui.add(button).clicked() {
                                self.qr_scanner.open(ui.ctx().clone());
                            }
                        }
                    }
                    output
                });
            response
        });

        #[cfg(all(feature = "qr-scanner", target_os = "linux"))]
        {
            if self.qr_enabled {
                self.qr_scanner.show(ui.ctx());
                let res = self.qr_scanner.get_result();
                if let Some(res) = res {
                    self.qr_scanner.reset();
                    self.set_text(res);
                }
            }
        }

        self.set_text_hidden(hidden);

        Some(response.inner)
    }
}
