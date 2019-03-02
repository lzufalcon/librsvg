extern crate cairo;
extern crate rsvg_internals;

use cairo::ImageSurface;

use rsvg_internals::{IRect, RenderingError};
use rsvg_internals::surface_utils::{
    iterators::Pixels,
    shared_surface::{SharedImageSurface, SurfaceType},
    ImageSurfaceDataExt,
    Pixel,
};

pub enum BufferDiff {
    DifferentSizes,
    Diff(Diff),
}

pub struct Diff {
    num_pixels_changed: usize,
    max_diff: u8,
    surface: SharedImageSurface,
}

#[inline]
fn pixel_max(p: &Pixel) -> u8 {
    p.r.max(p.g).max(p.b).max(p.a)
}

#[inline]
fn emphasize(p: &Pixel) -> Pixel {
    let mut r = p.r as u32;
    let mut g = p.g as u32;
    let mut b = p.b as u32;
    let mut a = p.a as u32;

    // emphasize
    r = r * 4;
    g = g * 4;
    b = b * 4;
    a = a * 4;

    // make sure they are visible
    if r > 0 {
        r += 128;
    }

    if g > 0 {
        g += 128;
    }

    if b > 0 {
        b += 128;
    }

    if a > 0 {
        a += 128;
    }

    let r = r.min(255) as u8;
    let g = g.min(255) as u8;
    let b = b.min(255) as u8;
    let a = a.min(255) as u8;

    Pixel { r, g, b, a }
}

pub fn compare_surfaces(
    surf_a: &SharedImageSurface,
    surf_b: &SharedImageSurface,
) -> Result<BufferDiff, RenderingError> {
    let a_width = surf_a.width();
    let a_height = surf_a.height();

    let b_width = surf_b.width();
    let b_height = surf_b.height();

    if a_width != b_width || a_height != b_height {
        return Ok(BufferDiff::DifferentSizes);
    }

    let mut surf_diff = ImageSurface::create(cairo::Format::ARgb32, a_width, a_height)?;
    let diff_stride = surf_diff.get_stride() as usize;

    let bounds = IRect {
        x0: 0,
        y0: 0,
        x1: a_width,
        y1: a_height,
    };

    let mut num_pixels_changed = 0;
    let mut max_diff = 0;

    {
        let mut diff_data = surf_diff.get_data().unwrap();

        for ((xa, ya, pixel_a), (_, _, pixel_b)) in
            Pixels::new(surf_a, bounds).zip(Pixels::new(surf_b, bounds))
        {
            if pixel_a != pixel_b {
                num_pixels_changed += 1;

                let pixel_diff = pixel_a.diff(&pixel_b);

                let pixel_max_diff = pixel_max(&pixel_diff);

                max_diff = max_diff.max(pixel_max_diff);

                let mut pixel_diff = emphasize(&pixel_diff);

                if pixel_diff.r == 0 && pixel_diff.g == 0 && pixel_diff.b == 0 {
                    // alpha only difference; convert alpha to gray

                    pixel_diff.r = pixel_diff.a;
                    pixel_diff.g = pixel_diff.a;
                    pixel_diff.b = pixel_diff.a;
                }

                diff_data.set_pixel(diff_stride, pixel_diff, xa, ya);
            } else {
                let black = Pixel {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                };
                diff_data.set_pixel(diff_stride, black, xa, ya);
            }
        }
    }

    let surface = SharedImageSurface::new(surf_diff, SurfaceType::SRgb)?;

    Ok(BufferDiff::Diff(Diff {
        num_pixels_changed,
        max_diff,
        surface,
    }))
}
