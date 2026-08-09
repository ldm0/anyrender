#![cfg(all(feature = "filters", not(feature = "multithreading")))]

use std::sync::Arc;

use anyrender::{Filter, ImageRenderer, PaintScene, filters::FilterEffect};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use kurbo::{Affine, Rect};
use peniko::{Color, Fill, Mix};

const WIDTH: u32 = 100;
const HEIGHT: u32 = 100;

#[test]
fn filtered_layer_can_contain_a_nested_clip() {
    let mut renderer = VelloCpuImageRenderer::new(WIDTH, HEIGHT);
    let mut pixels = Vec::new();

    renderer.render_to_vec(
        |scene| {
            scene.push_layer(
                Mix::Normal,
                1.0,
                Affine::IDENTITY,
                &Rect::new(10.0, 10.0, 90.0, 90.0),
                Some(Arc::new(Filter::single(FilterEffect::blur(5.0)))),
                None,
            );
            scene.push_clip_layer(Affine::IDENTITY, &Rect::new(30.0, 30.0, 70.0, 70.0));
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                Color::from_rgb8(255, 0, 0),
                None,
                &Rect::new(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT)),
            );
            scene.pop_layer();
            scene.pop_layer();
        },
        &mut pixels,
    );

    let outside = pixel(&pixels, 5, 50);
    let blurred_edge = pixel(&pixels, 27, 50);
    let center = pixel(&pixels, 50, 50);

    assert_eq!(outside, [0, 0, 0, 0], "the outer layer clip must hold");
    assert!(
        blurred_edge[0] > 0 && blurred_edge[3] > 0,
        "the filter must extend the nested clip's red input: {blurred_edge:?}"
    );
    assert!(
        center[0] > 240 && center[1] == 0 && center[2] == 0 && center[3] > 240,
        "the filtered layer must preserve its opaque red center: {center:?}"
    );
    assert!(
        blurred_edge[3] < center[3],
        "the edge must be partially blurred instead of merely filled: edge={blurred_edge:?}, center={center:?}"
    );
}

fn pixel(buffer: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * WIDTH + x) * 4) as usize;
    buffer[offset..offset + 4].try_into().unwrap()
}
