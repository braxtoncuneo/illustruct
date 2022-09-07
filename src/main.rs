#![feature(trait_alias, extend_one)]

mod kind;
mod block_draw;
mod graph;
mod mem_ribbon;
mod access;

use block_draw::util::Vec2;
use kind::{Kind, Primitive, composite::{Field, CompositeMode}};

use svg::Document;

use crate::block_draw::{BlockDrawSpec, util::Translate};

/*
struct ToneMap <N: const usize> (RGB[N]);

&impl ToneMap {

    fn map_tone(&self,tone_id:usiz&e) -> Option<RGB>{

    }

    fn to_color_str(&self,tone: Tone) -> String {
        let RGB(r,g,b) = self.map_tone(tone);
        format!("#{:02X?}{:02X?}{:02X?}",r,g,b)
    }

}
*/

#[allow(unused_variables)]
fn main() {
    use Primitive::*;

    let uint8_t  = Kind::from(U8);
    let uint16_t = Kind::from(U16);
    let uint32_t = Kind::from(U32);

    fn fields<'a>(names: &'_ str, kind: &'a Kind<'a>) -> Vec<Field<'a>> {
        names.split_whitespace().map(|name| Field::new(name, kind)).collect()
    }

    let point = Kind::comp(
        "Point".into(),
        CompositeMode::Product,
        fields("x gorp z", &uint16_t),
    );

    let triangle = Kind::comp(
        "Line".into(),
        CompositeMode::Product,
        fields("a b c", &point),
    );

    let tetra = Kind::comp(
        "LineUnion".into(),
        CompositeMode::Sum,
        fields("p r", &triangle),
    );

    let spec = &BlockDrawSpec {
        char_dims:  Vec2::new(10.0, 16.0),
        text_pads:  Vec2::new( 4.0,  4.0),
        label_pads: Vec2::new( 6.0,  4.0),
        union_xpad: 6.0,
        fill_inset: 3.0,
        prong_width: 12.0,
        prong_xpad: 6.0,
        chamfer_size : 12.0
    };

    let diag = spec.make_plan(&tetra, Vec2::default(), None, false)
        .into_svg()
        .set("transform", Translate(16.0, 16.0));

    let document = Document::new()
        .set("viewBox", (0, 0, 1100, 800))
        .add(diag);

    svg::save("image.svg", &document).unwrap();
}
