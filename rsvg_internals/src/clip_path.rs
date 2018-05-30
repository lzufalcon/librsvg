use libc;
use std::cell::Cell;

use cairo::{self, MatrixTrait};

use attributes::Attribute;
use coord_units::CoordUnits;
use drawing_ctx::{self, RsvgDrawingCtx};
use handle::RsvgHandle;
use node::{boxed_node_new, NodeResult, NodeTrait, NodeType, RsvgNode};
use parsers::parse;
use property_bag::PropertyBag;

coord_units!(ClipPathUnits, CoordUnits::UserSpaceOnUse);

pub struct NodeClipPath {
    units: Cell<ClipPathUnits>,
}

impl NodeClipPath {
    fn new() -> NodeClipPath {
        NodeClipPath {
            units: Cell::new(ClipPathUnits::default()),
        }
    }

    pub fn get_units(&self) -> ClipPathUnits {
        self.units.get()
    }

    pub fn to_cairo_context(&self, node: &RsvgNode, draw_ctx: *mut RsvgDrawingCtx) {
        let cascaded = node.get_cascaded_values();

        let clip_units = self.units.get();

        let orig_bbox = drawing_ctx::get_bbox(draw_ctx).clone();

        let cr = drawing_ctx::get_cairo_context(draw_ctx);

        //        let cr_save_transform = cr.get_matrix();

        let child_matrix = if clip_units == ClipPathUnits(CoordUnits::ObjectBoundingBox) {
            let rect = orig_bbox.rect.unwrap();

            let mut bbtransform =
                cairo::Matrix::new(rect.width, 0.0, 0.0, rect.height, rect.x, rect.y);
            cairo::Matrix::multiply(&bbtransform, &cr.get_matrix())
        } else {
            cr.get_matrix()
        };

        let cr = drawing_ctx::get_cairo_context(draw_ctx);
        cr.set_matrix(child_matrix);

        node.draw_children(&cascaded, draw_ctx, false, true);

        // FIXME: this is an EPIC HACK to keep the clipping context from
        // accumulating bounding boxes.  We'll remove this later, when we
        // are able to extract bounding boxes from outside the
        // general drawing loop.
        drawing_ctx::set_bbox(draw_ctx, &orig_bbox);

        let cr = drawing_ctx::get_cairo_context(draw_ctx);
        cr.clip();

        //        if clip_units == ClipPathUnits(CoordUnits::ObjectBoundingBox) {
        //            cr.set_matrix(cr_save_transform);
        //        }
    }
}

impl NodeTrait for NodeClipPath {
    fn set_atts(&self, _: &RsvgNode, _: *const RsvgHandle, pbag: &PropertyBag) -> NodeResult {
        for (_key, attr, value) in pbag.iter() {
            match attr {
                Attribute::ClipPathUnits => {
                    self.units.set(parse("clipPathUnits", value, (), None)?)
                }

                _ => (),
            }
        }

        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn rsvg_node_clip_path_new(
    _: *const libc::c_char,
    raw_parent: *const RsvgNode,
) -> *const RsvgNode {
    boxed_node_new(
        NodeType::ClipPath,
        raw_parent,
        Box::new(NodeClipPath::new()),
    )
}
