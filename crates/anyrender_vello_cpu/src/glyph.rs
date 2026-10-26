//! Prepared glyph measurement for CPU painting and its bounded fallbacks.

use anyrender::{Glyph, NormalizedCoord};
use glifo_vello_cpu::{FontEmbolden, GlyphPrepCache, GlyphRunBuilder};
use kurbo::{Affine, Diagonal2, Rect, Stroke, Vec2};
use peniko::FontData;

/// The representation selected by the CPU backend for a glyph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphRepresentation {
    /// A COLR paint graph, including its layers and clip box.
    Colr,
    /// A supported bitmap strike.
    Bitmap,
    /// A monochrome outline, including synthetic weight.
    Outline,
}

/// Conservative geometric ink for one positioned glyph, in scene coordinates.
///
/// Bounds include font variation, hinting, synthetic weight, an optional stroke,
/// and both transforms. Paint transparency, device sampling, filters and caller
/// clipping are excluded. Color images follow the CPU backend's current filled
/// image behavior when a stroke is requested.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GlyphInkBounds {
    /// The glyph has no geometric ink, such as a space.
    Empty,
    /// Preparation found drawable geometry.
    Bounds {
        /// The selected representation.
        representation: GlyphRepresentation,
        /// Conservative bounds after the run and glyph transforms.
        bounds: Rect,
    },
    /// Geometry could not be measured; callers should retain a conservative fallback.
    Unavailable,
}

/// A reusable glyph measurer that owns the backend's preparation caches.
///
/// Measuring does not draw or allocate an atlas. Cache maintenance is performed
/// automatically; the font-engine builders and caches remain private.
#[derive(Debug, Default)]
pub struct GlyphMeasurer {
    cache: GlyphPrepCache,
}

impl GlyphMeasurer {
    /// Measure one result per glyph with the same parameters as CPU glyph drawing.
    /// Synthetic weight and stroke width are local pixels before either transform.
    /// `stroke` adds conservative outline stroke ink to the fill bounds.
    #[allow(
        clippy::too_many_arguments,
        reason = "Mirrors PaintScene::draw_glyphs parameters."
    )]
    pub fn measure(
        &mut self,
        font: &FontData,
        font_size: f32,
        hint: bool,
        normalized_coords: &[NormalizedCoord],
        embolden: Vec2,
        stroke: Option<&Stroke>,
        transform: Affine,
        glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = Glyph> + Clone,
    ) -> Vec<GlyphInkBounds> {
        self.cache.maintain();
        GlyphRunBuilder::new(font.clone(), transform, Affine::IDENTITY, ())
            .font_size(font_size)
            .hint(hint)
            .normalized_coords(normalized_coords)
            .font_embolden(FontEmbolden::new(Diagonal2::new(embolden.x, embolden.y)))
            .glyph_transform(glyph_transform.unwrap_or_default())
            .ink_bounds(
                glyphs.map(|glyph| glifo_vello_cpu::Glyph {
                    id: glyph.id,
                    x: glyph.x,
                    y: glyph.y,
                }),
                &mut self.cache,
                stroke,
            )
            .into_iter()
            .map(|bounds| match bounds {
                glifo_vello_cpu::GlyphInkBounds::Empty => GlyphInkBounds::Empty,
                glifo_vello_cpu::GlyphInkBounds::Unavailable => GlyphInkBounds::Unavailable,
                glifo_vello_cpu::GlyphInkBounds::Bounds {
                    representation,
                    bounds,
                } => GlyphInkBounds::Bounds {
                    representation: match representation {
                        glifo_vello_cpu::GlyphRepresentation::Colr => GlyphRepresentation::Colr,
                        glifo_vello_cpu::GlyphRepresentation::Bitmap => GlyphRepresentation::Bitmap,
                        glifo_vello_cpu::GlyphRepresentation::Outline => {
                            GlyphRepresentation::Outline
                        }
                    },
                    bounds,
                },
            })
            .collect()
    }

    /// Release the stored glyph preparations.
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}
