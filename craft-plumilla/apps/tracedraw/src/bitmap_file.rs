//! The file of one picture (#1834): Save Bitmap As writes a picture as
//! PNG, JPEG or TIFF, and Edit Bitmap hands it to another program and
//! reads it back.
//!
//! A drawing keeps a picture's pixels as a PNG, RGB for the screen, and
//! for a print picture (a `.cdr` CMYK image, or a picture in the CMYK
//! colour mode) also its mode and the ink channels it prints from
//! (`Shape::picture_inks`, #1659). A CMYK picture saved as JPEG or TIFF
//! is written in CMYK from those inks (or its pixels through the colour
//! management when it keeps none), with the CMYK writers of the
//! JPEG and TIFF export (#1763); PNG holds no CMYK and is written as the
//! drawing keeps it. A CMYK TIFF read back gives the picture its inks
//! again.

use tracedraw_core::document::ShapeKind;
#[cfg(test)]
use tracedraw_core::picture_inks::PictureInks;
use tracedraw_core::{Document, Shape};

/// The kinds of file a picture is written as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapFile {
    Png,
    Jpeg,
    Tiff,
}

impl BitmapFile {
    /// In the order the save dialog lists them.
    pub const ALL: [BitmapFile; 3] = [BitmapFile::Png, BitmapFile::Jpeg, BitmapFile::Tiff];

    /// The file name extension written.
    pub fn ext(self) -> &'static str {
        match self {
            BitmapFile::Png => "png",
            BitmapFile::Jpeg => "jpg",
            BitmapFile::Tiff => "tif",
        }
    }

    /// The extensions the save dialog's filter takes.
    pub fn exts(self) -> &'static [&'static str] {
        match self {
            BitmapFile::Png => &["png"],
            BitmapFile::Jpeg => &["jpg", "jpeg"],
            BitmapFile::Tiff => &["tif", "tiff"],
        }
    }

    /// The name of the save dialog's filter (a format name).
    pub fn filter_name(self) -> &'static str {
        match self {
            BitmapFile::Png => "PNG",
            BitmapFile::Jpeg => "JPEG",
            BitmapFile::Tiff => "TIFF",
        }
    }

    /// The kind a file name's extension names, if one of these.
    pub fn from_name(name: &str) -> Option<BitmapFile> {
        let ext = std::path::Path::new(name)
            .extension()?
            .to_str()?
            .to_ascii_lowercase();
        BitmapFile::ALL
            .into_iter()
            .find(|k| k.exts().contains(&ext.as_str()))
    }
}

/// Whether `s` is a picture in the CMYK colour mode or keeping ink
/// channels that fit its pixels.
pub fn is_cmyk(s: &Shape) -> bool {
    let ShapeKind::Bitmap { png, .. } = &s.kind else {
        return false;
    };
    s.cmyk_bitmap
        || s.picture_inks
            .as_deref()
            .is_some_and(|i| i.fits(png.as_slice()))
}

/// The kind of file a picture is saved as unless the user picks another:
/// the kind of the file it is linked to or was imported from (its
/// name), else TIFF for a CMYK picture, which keeps its inks without
/// loss, else PNG.
pub fn own_kind(s: &Shape) -> BitmapFile {
    let link = s
        .data
        .iter()
        .find(|(k, _)| k == "bitmap.link")
        .map(|(_, v)| v.as_str());
    link.and_then(BitmapFile::from_name)
        .or_else(|| s.name.as_deref().and_then(BitmapFile::from_name))
        .unwrap_or(if is_cmyk(s) {
            BitmapFile::Tiff
        } else {
            BitmapFile::Png
        })
}

/// The picture's resolution in dots per inch, from its pixel width and
/// its width on the page.
fn dpi(s: &Shape) -> f64 {
    let ShapeKind::Bitmap { rect, width_px, .. } = &s.kind else {
        return 0.0;
    };
    let c = s.transform.as_coeffs();
    let mm = rect.width().abs() * c[0].hypot(c[1]);
    if mm > 0.0 && mm.is_finite() {
        f64::from(*width_px) * 25.4 / mm
    } else {
        0.0
    }
}

/// JPEG quality of a saved picture.
const QUALITY: u8 = 92;

/// The file of picture `s` of `doc` as `kind`; `icc` is the CMYK
/// profile a CMYK file carries.
pub fn write(
    doc: &Document,
    s: &Shape,
    kind: BitmapFile,
    icc: Option<&[u8]>,
) -> Result<Vec<u8>, String> {
    let ShapeKind::Bitmap { png, .. } = &s.kind else {
        return Err("not a bitmap".into());
    };
    let dpi = dpi(s);
    if kind == BitmapFile::Png {
        return Ok(png.to_vec());
    }
    if is_cmyk(s) {
        let source_profile = s
            .picture_inks
            .as_deref()
            .filter(|inks| inks.fits(png))
            .and_then(|inks| inks.profile.as_ref())
            .map(|profile| profile.data.as_slice());
        let icc = icc.or(source_profile);
        let spots: Vec<(u32, [f32; 4])> = doc.inks.iter().map(|i| (i.id, i.cmyk)).collect();
        let cmyk = |alpha: bool| -> Result<(u32, u32, Vec<u8>), String> {
            // Transform full source inks, then flatten alpha for an opaque output.
            let (w, h, mut samples) = crate::picture_plates::picture_cmyk(s, &spots, true)
                .ok_or("the picture cannot be read")?;
            if let (Some(source), Some(target)) = (source_profile, icc) {
                tracedraw_io::picture::convert_cmyk_samples(&mut samples, 5, source, target)
                    .map_err(|error| error.to_string())?;
            }
            if alpha {
                return Ok((w, h, samples));
            }
            let opaque = samples
                .chunks_exact(5)
                .flat_map(|p| {
                    let alpha = u16::from(p[4]);
                    [p[0], p[1], p[2], p[3]]
                        .map(|value| ((u16::from(value) * alpha + 127) / 255) as u8)
                })
                .collect();
            Ok((w, h, opaque))
        };
        let out = match kind {
            BitmapFile::Jpeg => {
                // No transparency in a JPEG: the inks over white paper.
                let (w, h, ink) = cmyk(false)?;
                tracedraw_io::cmyk_bitmap::cmyk_jpeg(w, h, &ink, QUALITY, dpi, icc)
            }
            _ => {
                // A TIFF keeps the transparency as an alpha channel
                // beside the inks (#1925); an opaque picture stays four
                // channels.
                let (w, h, inka) = cmyk(true)?;
                if inka.chunks_exact(5).all(|p| p[4] == 255) {
                    let ink: Vec<u8> = inka
                        .chunks_exact(5)
                        .flat_map(|p| [p[0], p[1], p[2], p[3]])
                        .collect();
                    tracedraw_io::cmyk_bitmap::cmyk_tiff(w, h, &ink, dpi, icc)
                } else {
                    tracedraw_io::cmyk_bitmap::cmyka_tiff(w, h, &inka, dpi, icc)
                }
            }
        };
        return out.ok_or_else(|| "CMYK encoding failed".to_string());
    }
    let img = crate::bitmap_fx::decode(png).ok_or("the picture cannot be read")?;
    let gray = crate::bitmap_fx::png_is_gray(png);
    let mut out = std::io::Cursor::new(Vec::new());
    match kind {
        BitmapFile::Jpeg => {
            // No transparency in a JPEG: the picture over white paper.
            let flat = image::RgbImage::from_fn(img.width(), img.height(), |x, y| {
                let p = img.get_pixel(x, y).0;
                let a = u16::from(p[3]);
                image::Rgb(
                    [p[0], p[1], p[2]]
                        .map(|v| ((u16::from(v) * a + 255 * (255 - a) + 127) / 255) as u8),
                )
            });
            let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, QUALITY);
            if gray {
                image::DynamicImage::ImageRgb8(flat)
                    .to_luma8()
                    .write_with_encoder(enc)
            } else {
                flat.write_with_encoder(enc)
            }
            .map_err(|e| e.to_string())?;
            Ok(crate::export::record_dpi(out.into_inner(), "jpg", dpi))
        }
        _ => {
            let opaque = img.pixels().all(|p| p[3] == 255);
            let dynamic = image::DynamicImage::ImageRgba8(img);
            let dynamic = match (gray, opaque) {
                (true, true) => image::DynamicImage::ImageLuma8(dynamic.to_luma8()),
                (true, false) => image::DynamicImage::ImageLumaA8(dynamic.to_luma_alpha8()),
                (false, true) => image::DynamicImage::ImageRgb8(dynamic.to_rgb8()),
                (false, false) => dynamic,
            };
            dynamic
                .write_to(&mut out, image::ImageFormat::Tiff)
                .map_err(|e| e.to_string())?;
            Ok(out.into_inner())
        }
    }
}

pub use tracedraw_io::picture::ReadPicture;

/// Read a picture file: a CMYK TIFF (8 bits per ink, with or without
/// alpha) keeps its inks, and its pixels show them through the colour
/// management; every other file is read as pixels.
pub fn read(bytes: &[u8]) -> Result<ReadPicture, String> {
    tracedraw_io::picture::read(bytes).map_err(|error| error.to_string())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::app::App;
    use tracedraw_core::geometry::Rect;
    use tracedraw_core::{Command, ShapeId};

    #[test]
    fn import_and_save_keep_each_picture_profile_with_one_undo_step() {
        let source_profile = tracedraw_core::icc::test_profiles::lut16_cmyk_profile();
        let values: Vec<u8> = (0..8)
            .flat_map(|i| [17 + i, 33, 49, 65, if i < 4 { 127 } else { 255 }])
            .collect();
        let source =
            tracedraw_io::cmyk_bitmap::cmyka_tiff(4, 2, &values, 300.0, Some(&source_profile))
                .unwrap();
        let folder = dir("profile-import");
        let path = folder.join("original.tif");
        std::fs::write(&path, &source).unwrap();
        let mut app = App::headless();
        // Match the working profile to the source: the default import policy
        // converts to the working profile. This test checks preservation when
        // no colour conversion is needed, including the original CMYKA samples.
        app.engine
            .run(&Command::SetCmykProfile {
                profile: Some(tracedraw_core::document::DocumentProfile::new(
                    "CMYK fixture",
                    source_profile.clone(),
                )),
            })
            .unwrap();
        let working = app.doc().cmyk_profile.clone();
        app.import_bitmap(&path);
        assert!(
            app.doc().cmyk_profile == working,
            "import keeps the existing working profile"
        );
        let id = app.selection[0];
        let shape = app.doc().find_shape(id).unwrap();
        assert!(shape.cmyk_bitmap);
        assert_eq!(
            shape
                .picture_inks
                .as_ref()
                .unwrap()
                .profile
                .as_ref()
                .unwrap()
                .data
                .as_slice(),
            source_profile
        );
        let output = folder.join("saved.tif");
        app.save_bitmap_to(id, &output, BitmapFile::Tiff).unwrap();
        let bytes = std::fs::read(output).unwrap();
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(&bytes)).unwrap();
        assert_eq!(
            decoder.get_tag_u8_vec(tiff::tags::Tag::IccProfile).unwrap(),
            source_profile
        );
        let tiff::decoder::DecodingResult::U8(actual) = decoder.read_image().unwrap() else {
            panic!("CMYKA8");
        };
        assert_eq!(actual, values);
        app.undo();
        assert!(
            app.doc().find_shape(id).is_none(),
            "one undo removes pixels, inks, profile and link together"
        );
        app.redo();
        assert!(app
            .doc()
            .find_shape(id)
            .unwrap()
            .picture_inks
            .as_ref()
            .unwrap()
            .profile
            .is_some());
        app.try_open_path(path.clone()).unwrap();
        assert_eq!(
            app.doc().cmyk_profile.as_ref().unwrap().data.as_slice(),
            source_profile
        );
        assert!(
            app.file.is_none(),
            "Save must not overwrite an imported TIFF with native JSON"
        );
        assert_eq!(std::fs::read(path).unwrap(), source);
        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn cropping_original_ink_planes_keeps_their_profile() {
        let profile = tracedraw_core::icc::test_profiles::lut16_cmyk_profile();
        let samples: Vec<u8> = (0..8).flat_map(|i| [i * 16, 33, 49, 65, 127]).collect();
        let source =
            tracedraw_io::cmyk_bitmap::cmyka_tiff(4, 2, &samples, 300.0, Some(&profile)).unwrap();
        let doc = tracedraw_io::raster::document_from_bytes(&source, "crop").unwrap();
        let mut shape = doc.pages[0].layers[0].shapes[0].clone();
        let ShapeKind::Bitmap { rect, .. } = &shape.kind else {
            panic!("bitmap");
        };
        let area = Rect::new(rect.x0, rect.y0, rect.x0 + rect.width() / 2.0, rect.y1);
        let (kind, inks) = crate::crop_tool::crop_picture(&shape, area).unwrap();
        shape.kind = kind;
        shape.picture_inks = inks;
        let bytes = tracedraw_io::picture::original_cmyk_tiff(&shape, 300.0).unwrap();
        let mut decoder = tiff::decoder::Decoder::new(std::io::Cursor::new(&bytes)).unwrap();
        assert_eq!(decoder.dimensions().unwrap(), (2, 2));
        assert_eq!(
            decoder.get_tag_u8_vec(tiff::tags::Tag::IccProfile).unwrap(),
            profile
        );
        let tiff::decoder::DecodingResult::U8(actual) = decoder.read_image().unwrap() else {
            panic!("CMYKA8");
        };
        let expected: Vec<u8> = samples
            .chunks_exact(20)
            .flat_map(|row| row[..10].iter().copied())
            .collect();
        assert_eq!(actual, expected);
    }

    /// Four by two pixels: cyan ink on the left half, black on the
    /// right, as a `.cdr` CMYK image is read (pixels for the screen and
    /// its ink channels).
    fn cmyk_picture(app: &mut App) -> ShapeId {
        let rgb = |x: u32| if x < 2 { [0, 160, 230] } else { [30, 30, 30] };
        let img = image::RgbaImage::from_fn(4, 2, |x, _| {
            let [r, g, b] = rgb(x);
            image::Rgba([r, g, b, 255])
        });
        let png = crate::bitmap_fx::encode(&img).expect("png");
        // Planes C, M, Y, K one under the other.
        let planes = image::GrayImage::from_fn(4, 8, |x, y| {
            let ink = y / 2;
            image::Luma([match (ink, x < 2) {
                (0, true) | (3, false) => 255,
                _ => 0,
            }])
        });
        let mut planes_png = std::io::Cursor::new(Vec::new());
        planes
            .write_to(&mut planes_png, image::ImageFormat::Png)
            .expect("planes");
        let inks = PictureInks::cmyk(&png, planes_png.into_inner().into());
        let id = app
            .new_shape(ShapeKind::Bitmap {
                rect: Rect::new(10.0, 10.0, 10.0 + 4.0 * 25.4 / 300.0, 20.0),
                width_px: 4,
                height_px: 2,
                png: png.into(),
                fx: None,
            })
            .expect("bitmap");
        app.run(Command::SetPictureInks {
            shape: id,
            cmyk: true,
            inks: Some(Box::new(inks)),
        });
        id
    }

    fn rgb_picture(app: &mut App) -> ShapeId {
        let img = image::RgbaImage::from_fn(5, 3, |x, _| image::Rgba([200, 40 * x as u8, 10, 255]));
        app.new_shape(ShapeKind::Bitmap {
            rect: Rect::new(10.0, 10.0, 15.0, 13.0),
            width_px: 5,
            height_px: 3,
            png: crate::bitmap_fx::encode(&img).expect("png").into(),
            fx: None,
        })
        .expect("bitmap")
    }

    fn dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("tracedraw-1834-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("dir");
        d
    }

    fn planes(s: &Shape) -> Vec<u8> {
        let inks = s.picture_inks.as_deref().expect("inks");
        let ShapeKind::Bitmap { png, .. } = &s.kind else {
            panic!("not a bitmap");
        };
        assert!(inks.fits(png.as_slice()), "the inks fit the pixels");
        image::load_from_memory(&inks.png)
            .expect("planes")
            .to_luma8()
            .into_raw()
    }

    /// Save Bitmap As (#1834): a CMYK picture is offered as TIFF and
    /// keeps its ink channels as they are in a CMYK TIFF; as JPEG it is a
    /// four-component CMYK file; PNG stays the pixels as kept.
    #[test]
    fn save_bitmap_as_keeps_the_inks_of_a_cmyk_picture() {
        let mut app = App::headless();
        let id = cmyk_picture(&mut app);
        let s = app.doc().find_shape(id).expect("picture").clone();
        assert!(is_cmyk(&s));
        assert_eq!(own_kind(&s), BitmapFile::Tiff);
        let d = dir("save");

        let tif = d.join("picture.tif");
        app.save_bitmap_to(id, &tif, BitmapFile::Png).expect("tiff");
        let bytes = std::fs::read(&tif).expect("read");
        let mut dec = tiff::decoder::Decoder::new(std::io::Cursor::new(&bytes)).expect("tiff");
        assert_eq!(dec.colortype().expect("type"), tiff::ColorType::CMYK(8));
        let back = read(&bytes).expect("read back");
        assert_eq!(back.planes.is_some(), true);
        let (_, inks) = back.into_png().expect("png");
        let got = image::load_from_memory(&inks.expect("inks").png)
            .expect("planes")
            .to_luma8()
            .into_raw();
        assert_eq!(got, planes(&s), "the inks as the drawing keeps them");
        assert_eq!(
            crate::image_dpi::resolution(&bytes).map(|(x, _)| x.round()),
            Some(300.0)
        );

        let jpg = d.join("picture.jpeg");
        app.save_bitmap_to(id, &jpg, BitmapFile::Png).expect("jpeg");
        let bytes = std::fs::read(&jpg).expect("read");
        let sof = bytes
            .windows(2)
            .position(|w| w == [0xFF, 0xC0])
            .expect("SOF0");
        assert_eq!(bytes[sof + 9], 4, "four components (CMYK)");
        let img = image::load_from_memory(&bytes).expect("decodes").to_rgb8();
        assert_eq!(img.dimensions(), (4, 2));
        // Cyan ink alone shows as no red, full green and blue.
        let p = img.get_pixel(0, 0).0;
        assert!(p[0] < 40 && p[1] > 215 && p[2] > 215, "cyan: {p:?}");

        let png = d.join("picture.png");
        app.save_bitmap_to(id, &png, BitmapFile::Tiff).expect("png");
        let ShapeKind::Bitmap { png: kept, .. } = &s.kind else {
            panic!("bitmap");
        };
        assert_eq!(std::fs::read(&png).expect("read"), kept.to_vec());

        // No extension: the kind asked for.
        let plain = d.join("picture");
        app.save_bitmap_to(id, &plain, BitmapFile::Tiff)
            .expect("plain");
        assert!(std::fs::read(&plain).expect("read").starts_with(b"II*\0"));
        let _ = std::fs::remove_dir_all(&d);
    }

    /// #1925: a CMYK picture with transparency (here its right half is
    /// clear) keeps it in Save Bitmap As TIFF, as an alpha channel
    /// beside the full inks, and in Edit Bitmap: the file read back
    /// leaves the picture transparent where it was.
    #[test]
    fn a_cmyk_picture_keeps_its_transparency_in_tiff_and_edit_bitmap() {
        let mut app = App::headless();
        let id = cmyk_picture(&mut app);
        // The same picture with its right half (the black ink) clear.
        let s = app.doc().find_shape(id).expect("picture").clone();
        let ShapeKind::Bitmap { png, .. } = &s.kind else {
            panic!("bitmap");
        };
        let mut px = crate::bitmap_fx::decode(png).expect("pixels");
        for (x, _, p) in px.enumerate_pixels_mut() {
            if x >= 2 {
                p.0[3] = 0;
            }
        }
        let clear = crate::bitmap_fx::encode(&px).expect("png");
        let inks = PictureInks::cmyk(&clear, s.picture_inks.as_deref().expect("inks").png.clone());
        let id = app
            .new_shape(ShapeKind::Bitmap {
                rect: Rect::new(30.0, 10.0, 30.0 + 4.0 * 25.4 / 300.0, 20.0),
                width_px: 4,
                height_px: 2,
                png: clear.into(),
                fx: None,
            })
            .expect("bitmap");
        app.run(Command::SetPictureInks {
            shape: id,
            cmyk: true,
            inks: Some(Box::new(inks)),
        });
        let before = app.doc().find_shape(id).expect("picture").clone();
        let d = dir("alpha");

        let tif = d.join("clear.tif");
        app.save_bitmap_to(id, &tif, BitmapFile::Png).expect("tiff");
        let bytes = std::fs::read(&tif).expect("read");
        let mut dec = tiff::decoder::Decoder::new(std::io::Cursor::new(&bytes)).expect("tiff");
        assert_eq!(dec.colortype().expect("type"), tiff::ColorType::CMYKA(8));
        let back = read(&bytes).expect("read back");
        let alpha: Vec<u8> = back.rgba.pixels().map(|p| p.0[3]).collect();
        assert_eq!(alpha, [255, 255, 0, 0, 255, 255, 0, 0], "clear half kept");
        let (_, inks) = back.into_png().expect("png");
        let got = image::load_from_memory(&inks.expect("inks").png)
            .expect("planes")
            .to_luma8()
            .into_raw();
        assert_eq!(got, planes(&before), "the full inks, not faded");

        // Edit Bitmap: hand the file over, the other program saves it as
        // it is, the picture is read back still clear on the right.
        let path = app.write_bitmap_for_editing(id, &d).expect("written");
        app.run(Command::SetObjectData {
            shape: id,
            data: vec![("bitmap.link".into(), path.display().to_string())],
        });
        let resaved = std::fs::read(&path).expect("read");
        std::fs::write(&path, resaved).expect("write");
        assert!(app.reload_linked_bitmap(id));
        let s = app.doc().find_shape(id).expect("picture");
        assert!(s.cmyk_bitmap);
        let ShapeKind::Bitmap { png, .. } = &s.kind else {
            panic!("bitmap");
        };
        let px = crate::bitmap_fx::decode(png).expect("pixels");
        let alpha: Vec<u8> = px.pixels().map(|p| p.0[3]).collect();
        assert_eq!(alpha, [255, 255, 0, 0, 255, 255, 0, 0], "still clear");
        assert_eq!(planes(s), planes(&before));

        // An opaque CMYK picture stays a four channel file.
        let tif = d.join("opaque.tif");
        let opaque = cmyk_picture(&mut app);
        app.save_bitmap_to(opaque, &tif, BitmapFile::Png)
            .expect("tiff");
        let bytes = std::fs::read(&tif).expect("read");
        let mut dec = tiff::decoder::Decoder::new(std::io::Cursor::new(&bytes)).expect("tiff");
        assert_eq!(dec.colortype().expect("type"), tiff::ColorType::CMYK(8));
        let _ = std::fs::remove_dir_all(&d);
    }

    /// An RGB picture is offered as the kind of file it came from, else
    /// PNG, and is written as JPEG or TIFF when asked.
    #[test]
    fn save_bitmap_as_writes_an_rgb_picture_as_jpeg_or_tiff() {
        let mut app = App::headless();
        let id = rgb_picture(&mut app);
        let s = app.doc().find_shape(id).expect("picture").clone();
        assert!(!is_cmyk(&s));
        assert_eq!(own_kind(&s), BitmapFile::Png);
        let mut named = s.clone();
        named.name = Some("Photo.JPG".into());
        assert_eq!(own_kind(&named), BitmapFile::Jpeg);
        named
            .data
            .push(("bitmap.link".into(), "/pictures/scan.tiff".into()));
        assert_eq!(own_kind(&named), BitmapFile::Tiff, "the linked file first");

        let d = dir("rgb");
        for (name, format) in [
            ("p.jpg", image::ImageFormat::Jpeg),
            ("p.tif", image::ImageFormat::Tiff),
        ] {
            let path = d.join(name);
            app.save_bitmap_to(id, &path, BitmapFile::Png).expect(name);
            let bytes = std::fs::read(&path).expect("read");
            assert_eq!(image::guess_format(&bytes).expect("format"), format);
            let img = image::load_from_memory(&bytes).expect("decodes").to_rgb8();
            assert_eq!(img.dimensions(), (5, 3));
            let p = img.get_pixel(2, 1).0;
            assert!(
                (p[0] as i32 - 200).abs() < 8 && (p[1] as i32 - 80).abs() < 8,
                "{name}: {p:?}"
            );
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Edit Bitmap (#1834): a CMYK picture goes to the other program as
    /// a CMYK TIFF with its inks, and the file it saves back gives the
    /// picture its new inks (and the pixels that show them); undo puts
    /// the old ones back. An RGB picture still goes as a PNG.
    #[test]
    fn edit_bitmap_hands_over_and_reads_back_the_inks() {
        let mut app = App::headless();
        let id = cmyk_picture(&mut app);
        let before = app.doc().find_shape(id).expect("picture").clone();
        let d = dir("edit");
        let path = app.write_bitmap_for_editing(id, &d).expect("written");
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("tif"));
        let back = read(&std::fs::read(&path).expect("read")).expect("tiff");
        let (_, inks) = back.into_png().expect("png");
        assert_eq!(
            image::load_from_memory(&inks.expect("inks").png)
                .expect("planes")
                .to_luma8()
                .into_raw(),
            planes(&before)
        );
        app.run(Command::SetObjectData {
            shape: id,
            data: vec![("bitmap.link".into(), path.display().to_string())],
        });
        // The other program saves magenta ink everywhere.
        let magenta: Vec<u8> = (0..8).flat_map(|_| [0u8, 255, 0, 0]).collect();
        let tif = tracedraw_io::cmyk_bitmap::cmyk_tiff(4, 2, &magenta, 300.0, None).expect("tif");
        std::fs::write(&path, tif).expect("write");
        assert!(app.reload_linked_bitmap(id));
        let s = app.doc().find_shape(id).expect("picture").clone();
        assert!(s.cmyk_bitmap);
        let p = planes(&s);
        assert!(p[..8].iter().all(|v| *v == 0), "no cyan");
        assert!(p[8..16].iter().all(|v| *v == 255), "magenta");
        assert!(p[16..].iter().all(|v| *v == 0), "no yellow or black");
        let ShapeKind::Bitmap { png, .. } = &s.kind else {
            panic!("bitmap");
        };
        let px = crate::bitmap_fx::decode(png).expect("pixels");
        let c = px.get_pixel(1, 1).0;
        assert!(
            c[0] > 180 && c[1] < 120 && c[2] > 100,
            "shows magenta: {c:?}"
        );
        app.engine.undo().expect("undo");
        let s = app.doc().find_shape(id).expect("picture");
        assert_eq!(planes(s), planes(&before));

        let rgb = rgb_picture(&mut app);
        let path = app.write_bitmap_for_editing(rgb, &d).expect("written");
        assert_eq!(path.extension().and_then(|e| e.to_str()), Some("png"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
