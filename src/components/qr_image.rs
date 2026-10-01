use egui::{Context, Image, ImageSource, Vec2, load::Bytes};
use image::{ImageBuffer, ImageFormat, Luma};
use qrcodegen_no_heap::*;
use std::sync::Arc;

#[cfg(feature = "secure-types")]
use secure_types::Zeroize;

/// Failure while encoding or rendering a [`QrImage`].
#[derive(Debug, Clone)]
pub enum QrError {
   /// The payload could not be encoded as a QR symbol.
   EncodingFailed(String),
   /// PNG encoding of the raster failed.
   ImageError(String),
   /// Any other failure (including [`QrImage::empty_with_error`]).
   Other(String),
}

impl QrError {
   /// Human-readable description of the error.
   pub fn to_string(&self) -> String {
      match self {
         QrError::EncodingFailed(message) => format!("Encoding failed: {}", message),
         QrError::ImageError(message) => format!("Image failed: {}", message),
         QrError::Other(message) => message.to_string(),
      }
   }
}

impl std::fmt::Display for QrError {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
         QrError::EncodingFailed(message) => write!(f, "Encoding failed: {}", message),
         QrError::ImageError(message) => write!(f, "Image failed: {}", message),
         QrError::Other(message) => write!(f, "{}", message),
      }
   }
}

impl std::error::Error for QrError {}

/// Zeroizes the given image data when the `secure-types` feature is enabled.
///
/// Returns `true` only when the data was actually zeroized.
#[cfg(feature = "secure-types")]
fn zeroize_image_data(data: &mut [u8]) -> bool {
   data.zeroize();
   true
}

#[cfg(not(feature = "secure-types"))]
fn zeroize_image_data(_data: &mut [u8]) -> bool {
   false
}

/// Default raster budget, in pixels per side.
const DEFAULT_TARGET_PX: u32 = 512;

/// Error-correction level of a QR symbol.
///
/// A higher level survives more damage but needs more modules for the same
/// payload, and more modules in the same space means smaller modules on screen —
/// which is what makes a symbol hard to scan. Ask for the lowest level that still
/// covers how the symbol is used: `High` for something printed or handled,
/// `Medium` for a symbol displayed on a clean screen.
///
/// Encoding upgrades the level for free when the payload still fits the same
/// symbol size, so asking for `Low` never costs modules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QrEcc {
   /// Tolerates about 7 % erroneous codewords.
   Low,
   /// Tolerates about 15 % erroneous codewords.
   Medium,
   /// Tolerates about 25 % erroneous codewords.
   Quartile,
   /// Tolerates about 30 % erroneous codewords; the default.
   #[default]
   High,
}

impl QrEcc {
   const fn to_codegen(self) -> QrCodeEcc {
      match self {
         Self::Low => QrCodeEcc::Low,
         Self::Medium => QrCodeEcc::Medium,
         Self::Quartile => QrCodeEcc::Quartile,
         Self::High => QrCodeEcc::High,
      }
   }
}

/// How a [`QrImage`] is encoded and rasterised.
///
/// The raster is always built from whole modules — `(modules + 2 * quiet_zone) *
/// module_px` pixels — so drawing it at [`QrImage::image_size`] needs no
/// resampling. Resampling a QR raster is what smears module edges and makes a
/// dense symbol unscannable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QrEncoding {
   /// Error-correction level.
   pub ecc: QrEcc,
   /// Raster budget per side, in pixels. The module size is this budget divided
   /// by the module count and rounded down (at least 1 px), so the raster never
   /// exceeds the budget — unless the budget is smaller than the module count, in
   /// which case every module is 1 px and the raster is [`QrImage::min_display_px`]
   /// instead.
   ///
   /// Size this to the space the symbol is drawn in, not to a fixed default: a
   /// symbol displayed at its natural size is crisp, and a bigger display size
   /// is the only thing that gives a dense payload bigger modules.
   pub target_px: u32,
   /// Light border around the symbol, in modules. The QR specification asks for 4
   /// and that is the default, because the surface a symbol is drawn on is not
   /// under our control: on a dark surface a borderless symbol sits straight
   /// against dark pixels, and a scanner thresholds the symbol's edge against
   /// them. Measured on screen: a 61-module symbol whose edge touched a dark card
   /// was not found at all, while the same symbol with this border was found.
   ///
   /// The border is part of the raster, so it is white wherever the symbol is
   /// drawn. It costs a little module size at a given [`Self::target_px`]; 0 gives
   /// that size back when the symbol is drawn on a light surface.
   pub quiet_zone: u32,
}

impl Default for QrEncoding {
   fn default() -> Self {
      Self {
         ecc: QrEcc::High,
         target_px: DEFAULT_TARGET_PX,
         quiet_zone: DEFAULT_QUIET_ZONE,
      }
   }
}

/// Light border the QR specification asks for, in modules.
const DEFAULT_QUIET_ZONE: u32 = 4;

/// A QR Code image.
///
/// The encoded image data lives in an `Arc<[u8]>` and uses the [`Bytes::Shared`] type
/// as the image source, which means no additional allocations are needed.
///
/// On `clear()` the image data is zeroized if the `secure-types` feature is enabled.
///
/// # Usage
///
/// ```no_run
/// use egui::*;
/// use egui_elements::components::QrImage;
///
/// struct MyUi {
///  open: bool,
///  qr: QrImage,
/// }
///
/// impl MyUi {
///
/// fn show(&mut self, ui: &mut Ui) {
/// if !self.open {
///  return;
/// }
///
/// let btn = Button::new("Show QR Code");
///
/// if ui.add(btn).clicked() {
///  let qr = QrImage::new("Hello, world!", "bytes://test".to_string());
///  self.qr = qr;
/// }
///
///  let image = self.qr.image();
///  ui.add(image);
///
/// // For the image to be cleared succesfully we need to call `clear()` on the `QrImage`
/// // when the image is out of scope (egui frame)
///
///  self.close_ui(ui);
/// }
///
/// fn close_ui(&mut self, ui: &mut Ui) {
///  let btn = Button::new("Close");
///  if ui.add(btn).clicked() {
///  // Close the UI so in the next frame the image doesn't show up
///  self.open = false;
///  self.qr.clear(ui.ctx());
///  }
/// }
/// }
/// ```
pub struct QrImage {
   /// Encoded QR image bytes (PNG), shared so egui can reference them cheaply.
   image_data: Arc<[u8]>,
   /// Cache key used by egui to identify and later forget the loaded image.
   uri: String,
   /// Symbol size in modules per side, quiet zone excluded. 0 without an image.
   modules: u32,
   /// Light border around the symbol, in modules.
   quiet_zone: u32,
   /// Module size of the raster in pixels. 0 without an image.
   module_px: u32,
   /// Stores a failure from encoding/rendering instead of ever erroring out.
   error: Option<QrError>,
}

impl QrImage {
   /// Create a new [QrImage] from the given data and uri.
   ///
   /// Uses [`QrEncoding::default`]: error correction `High` into a raster of about
   /// [`DEFAULT_TARGET_PX`] pixels per side. When the symbol is drawn at a known
   /// size, prefer [`QrImage::with_encoding`] and give the raster that size, so
   /// the raster is drawn 1:1 instead of being resampled.
   ///
   /// If the image encoding fails the error will be stored in the [QrImage].
   pub fn new(data: &str, uri: String) -> Self {
      Self::with_encoding(data, uri, QrEncoding::default())
   }

   /// Create a new [QrImage] with an explicit [`QrEncoding`].
   ///
   /// The raster is a whole number of modules per side, so
   /// [`QrImage::image_size`] is exactly what to draw it at:
   ///
   /// ```no_run
   /// # use egui::*;
   /// # use egui_elements::components::{QrEncoding, QrImage};
   /// # let (data, uri) = ("0x492e7Bc37cCBE1Afb1738ceE2aa880Fe7184C0d", String::new());
   /// let qr = QrImage::with_encoding(
   ///    data,
   ///    uri,
   ///    QrEncoding {
   ///       target_px: 250,
   ///       ..Default::default()
   ///    },
   /// );
   ///
   /// // Draw it at its natural size (see `image_size_pt` for HiDPI displays).
   /// let image = qr.image().fit_to_exact_size(qr.image_size());
   /// ```
   ///
   /// If the image encoding fails the error will be stored in the [QrImage].
   pub fn with_encoding(data: &str, uri: String, encoding: QrEncoding) -> Self {
      let res = data_to_qr(data, encoding);
      let (image_data, modules, module_px, error) = match res {
         Ok((image_data, modules, module_px)) => (image_data, modules, module_px, None),
         Err(e) => (Vec::new(), 0, 0, Some(e)),
      };

      Self {
         image_data: image_data.into(),
         uri,
         modules,
         quiet_zone: encoding.quiet_zone,
         module_px,
         error,
      }
   }

   /// Create an empty [QrImage] with an error.
   pub fn empty_with_error(error: String) -> Self {
      Self {
         image_data: Vec::new().into(),
         uri: String::new(),
         modules: 0,
         quiet_zone: 0,
         module_px: 0,
         error: Some(QrError::Other(error)),
      }
   }

   /// `true` if encoding or rendering failed.
   pub fn has_error(&self) -> bool {
      self.error.is_some()
   }

   /// The stored error, if any.
   pub fn error(&self) -> Option<&QrError> {
      self.error.as_ref()
   }

   /// Returns true if the image is cleared
   pub fn is_cleared(&self) -> bool {
      self.image_data.is_empty()
   }

   /// Returns an [Image] that can be used in egui.
   ///
   /// It uses [Bytes::Shared] as the source so only one allocation exists.
   pub fn image(&self) -> Image<'static> {
      let data = self.image_data.clone();
      let image = Image::new(ImageSource::Bytes {
         uri: self.uri.clone().into(),
         bytes: Bytes::Shared(data),
      });
      image
   }

   /// Size of the raster in pixels, or zero without an image.
   ///
   /// This is a whole number of modules per side (symbol plus its quiet zone), so
   /// drawing the image at exactly this size keeps every module edge crisp. Any
   /// other size resamples the raster and blurs the edges.
   pub fn image_size(&self) -> Vec2 {
      Vec2::splat((self.raster_modules() * self.module_px) as f32)
   }

   /// [`QrImage::image_size`] in points for the given scale factor.
   ///
   /// Use this to draw the raster 1:1 on a HiDPI display:
   ///
   /// ```no_run
   /// # use egui::*;
   /// # use egui_elements::components::QrImage;
   /// # let (qr, ui): (QrImage, &mut Ui) = unimplemented!();
   /// let size = qr.image_size_pt(ui.ctx().pixels_per_point());
   /// ui.add(qr.image().fit_to_exact_size(size));
   /// ```
   pub fn image_size_pt(&self, pixels_per_point: f32) -> Vec2 {
      self.image_size() / pixels_per_point.max(0.01)
   }

   /// Symbol size in modules per side, quiet zone excluded, or zero without an
   /// image.
   ///
   /// The module count is what decides how big the symbol has to be drawn: the
   /// same payload at a higher error-correction level, or a longer payload, needs
   /// more modules, so each module gets fewer pixels on screen.
   pub fn modules(&self) -> u32 {
      self.modules
   }

   /// Module size of the raster in pixels, or zero without an image.
   pub fn module_px(&self) -> u32 {
      self.module_px
   }

   /// Light border around the symbol, in modules.
   pub fn quiet_zone(&self) -> u32 {
      self.quiet_zone
   }

   /// Modules per side of the raster: the symbol plus its quiet zone.
   pub fn raster_modules(&self) -> u32 {
      if self.modules == 0 {
         0
      } else {
         self.modules + self.quiet_zone * 2
      }
   }

   /// Smallest display size, in pixels, that gives every module at least
   /// `module_px` pixels.
   ///
   /// Give the symbol this much room: below about 4 px per module a symbol on a
   /// raster display stops being reliably scannable — a scanner has to resolve the
   /// module grid out of resampled pixels, and the thinner the modules get the
   /// more the resampling matters.
   pub fn min_display_px(&self, module_px: u32) -> u32 {
      self.raster_modules() * module_px.max(1)
   }

   /// Clears the QR Code image.
   ///
   /// If the `secure-types` feature is enabled, the image data will be zeroized.
   /// otherwise it will just be replaced with 0s which doesn't guarantee zeroization.
   ///
   /// # Returns
   /// `true` If the image was zeroized successfully.
   pub fn clear(&mut self, ctx: &Context) -> bool {
      ctx.forget_image(&self.uri);

      self.error = None;
      self.modules = 0;
      self.quiet_zone = 0;
      self.module_px = 0;
      let success = match Arc::get_mut(&mut self.image_data) {
         Some(data) => {
            let did_zeroize = zeroize_image_data(data);
            self.image_data = Arc::new([0u8; 0]);
            did_zeroize
         }
         None => false,
      };

      success
   }
}

impl Drop for QrImage {
   fn drop(&mut self) {
      if let Some(data) = Arc::get_mut(&mut self.image_data) {
         zeroize_image_data(data);
      }
   }
}

/// Encodes `data` into a PNG raster and reports the symbol's module count and the
/// raster's module size.
pub(crate) fn data_to_qr(data: &str, encoding: QrEncoding) -> Result<(Vec<u8>, u32, u32), QrError> {
   let mut tempbuffer = vec![0u8; Version::MAX.buffer_len()];
   let mut outbuffer = vec![0u8; Version::MAX.buffer_len()];
   let qr = QrCode::encode_text(
      data,
      &mut tempbuffer,
      &mut outbuffer,
      encoding.ecc.to_codegen(),
      Version::MIN,
      Version::MAX,
      // No explicit mask: let the encoder pick the best one.
      None,
      // Upgrade error correction for free when the payload still fits the same
      // symbol size.
      true,
   )
   .map_err(|e| QrError::EncodingFailed(e.to_string()))?;

   let size = qr.size() as u32;
   // Whole modules only. A raster that is not an integer number of pixels per
   // module puts an interpolation artefact on every module edge, which is exactly
   // what a scanner cannot afford.
   let side_modules = size + encoding.quiet_zone * 2;
   let module_px = (encoding.target_px / side_modules).max(1);
   let img_size = side_modules * module_px;

   // Create white image buffer
   let mut img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(img_size, img_size);
   for pixel in img.pixels_mut() {
      *pixel = Luma([255u8]); // White background
   }

   let offset = encoding.quiet_zone * module_px;
   for y in 0..size {
      for x in 0..size {
         if qr.get_module(x as i32, y as i32) {
            let px = Luma([0u8]); // Black
            for dy in 0..module_px {
               for dx in 0..module_px {
                  *img.get_pixel_mut(
                     offset + x * module_px + dx,
                     offset + y * module_px + dy,
                  ) = px;
               }
            }
         }
      }
   }

   let mut png_bytes = Vec::new();
   img.write_to(
      &mut std::io::Cursor::new(&mut png_bytes),
      ImageFormat::Png,
   )
   .map_err(|e| QrError::ImageError(e.to_string()))?;

   #[cfg(feature = "secure-types")]
   {
      let mut img_buf = img.into_vec();
      img_buf.zeroize();
      tempbuffer.zeroize();
      outbuffer.zeroize();
   }

   Ok((png_bytes, size, module_px))
}

#[cfg(test)]
mod tests {
   use super::*;

   /// A SHA-512 digest the way swiss-knife serialises it for a QR symbol: 64 bytes
   /// as 128 hex characters.
   const DENSE_PAYLOAD: &str = concat!(
      "0123456789abcdef0123456789abcdef",
      "0123456789abcdef0123456789abcdef",
      "0123456789abcdef0123456789abcdef",
      "0123456789abcdef0123456789abcdef",
   );

   /// An EVM address: 42 characters.
   const ADDRESS: &str = "0x492e7Bc37cCBE1Afb1738ceE2aa880Fe7184C0d";

   fn raster(png: &[u8]) -> image::GrayImage {
      image::load_from_memory_with_format(png, ImageFormat::Png).unwrap().to_luma8()
   }

   #[test]
   fn test_qr_image() {
      let mut qr = QrImage::new("Hello, world!", "bytes://test".to_string());
      {
         let _image = qr.image();
      }

      assert!(!qr.is_cleared());
      assert!(!qr.has_error());
      assert!(qr.modules() > 0);
      assert_eq!(
         qr.image_size(),
         Vec2::splat((qr.raster_modules() * qr.module_px()) as f32)
      );

      qr.clear(&Context::default());
      assert!(qr.is_cleared());
      assert_eq!(qr.modules(), 0);
      assert_eq!(qr.raster_modules(), 0);
      assert_eq!(qr.module_px(), 0);
      assert_eq!(qr.image_size(), Vec2::ZERO);
   }

   /// The raster must be a whole number of modules per side and it must not exceed
   /// the requested budget, otherwise drawing it at `image_size()` would resample
   /// it and smear the module edges.
   #[test]
   fn raster_is_whole_modules_within_the_budget() {
      for target_px in [40, 120, 250, 512, 900] {
         // No quiet zone: this test is about the module budget arithmetic.
         let encoding = QrEncoding {
            target_px,
            quiet_zone: 0,
            ..Default::default()
         };
         let qr = QrImage::with_encoding(
            DENSE_PAYLOAD,
            "bytes://dense".to_string(),
            encoding,
         );
         assert!(!qr.has_error());

         let (modules, module_px) = (qr.modules(), qr.module_px());
         assert!(modules > 0, "target_px {target_px}");
         assert!(module_px >= 1, "target_px {target_px}");
         // Within the budget, unless one pixel per module is already narrower than
         // the module count — the floor is 1 px per module.
         assert!(
            qr.raster_modules() * module_px <= target_px || module_px == 1,
            "target_px {target_px}"
         );
         assert_eq!(
            qr.image_size(),
            Vec2::splat((qr.raster_modules() * module_px) as f32)
         );

         // The raster that gets loaded really has that many pixels.
         let (png, size, mp) = data_to_qr(DENSE_PAYLOAD, encoding).unwrap();
         assert_eq!((size, mp), (modules, module_px));
         let img = image::load_from_memory_with_format(&png, ImageFormat::Png).unwrap().to_luma8();
         assert_eq!(
            img.dimensions(),
            (modules * module_px, modules * module_px),
            "quiet_zone 0: the raster is the symbol alone"
         );
      }
   }

   /// Fewer modules for the same payload means bigger modules at the same display
   /// size, which is what makes a dense symbol scannable.
   #[test]
   fn lower_error_correction_uses_fewer_modules() {
      let with = |ecc| {
         QrImage::with_encoding(
            DENSE_PAYLOAD,
            format!("bytes://ecc-{ecc:?}"),
            QrEncoding {
               ecc,
               ..Default::default()
            },
         )
      };

      let low = with(QrEcc::Low);
      let high = with(QrEcc::High);

      assert!(low.modules() < high.modules());
      assert!(low.min_display_px(4) < high.min_display_px(4));
      assert_eq!(high.min_display_px(4), high.raster_modules() * 4);
   }

   /// The payload length is what drives the module count: a short address and a
   /// long hex digest are the same size on screen, so the digest's modules are
   /// much smaller — the difference between a code that scans and one that barely
   /// does.
   #[test]
   fn short_payloads_need_far_fewer_modules() {
      let address = QrImage::new(ADDRESS, "bytes://address".to_string());
      let digest = QrImage::new(DENSE_PAYLOAD, "bytes://digest".to_string());

      assert!(digest.modules() >= address.modules() * 3 / 2);
      // Same display size for both, so the digest gets fewer pixels per module.
      assert!(digest.module_px() <= address.module_px() || digest.modules() > address.modules());
      assert!(digest.min_display_px(4) > address.min_display_px(4));
   }

   /// The quiet zone is a light border the QR specification asks for; without one
   /// the symbol sits straight on the surface behind it, which is what a scanner
   /// thresholds against.
   #[test]
   fn quiet_zone_adds_a_light_border() {
      const QUIET_ZONE: u32 = 4;
      let encoding = QrEncoding {
         target_px: 250,
         quiet_zone: QUIET_ZONE,
         ..Default::default()
      };
      let bordered = QrImage::with_encoding(DENSE_PAYLOAD, "bytes://qz".to_string(), encoding);
      let plain = QrImage::with_encoding(
         DENSE_PAYLOAD,
         "bytes://plain".to_string(),
         QrEncoding {
            target_px: 250,
            ..Default::default()
         },
      );

      // Same symbol, more space needed around it.
      assert_eq!(bordered.modules(), plain.modules());
      assert!(bordered.module_px() <= plain.module_px());

      let side_modules = bordered.modules() + QUIET_ZONE * 2;
      let module_px = bordered.module_px();
      let (png, _, _) = data_to_qr(DENSE_PAYLOAD, encoding).unwrap();
      let img = raster(&png);
      assert_eq!(
         img.dimensions(),
         (side_modules * module_px, side_modules * module_px)
      );
      assert!(bordered.image_size().x <= 250.0);

      // Every pixel of the border is light.
      let side = side_modules * module_px;
      for i in 0..side {
         for b in 0..QUIET_ZONE * module_px {
            assert_eq!(img.get_pixel(i, b)[0], 255);
            assert_eq!(img.get_pixel(b, i)[0], 255);
         }
      }
   }

   #[test]
   fn image_size_pt_follows_the_scale_factor() {
      let qr = QrImage::new(ADDRESS, "bytes://pt".to_string());
      assert_eq!(qr.image_size_pt(1.0), qr.image_size());
      assert_eq!(qr.image_size_pt(2.0), qr.image_size() / 2.0);
   }
}
