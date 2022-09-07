#![feature(trait_alias,extend_one)]

mod kind;
mod block_draw;
mod graph;
mod mem_ribbon;
mod access;


use block_draw::Vec2;
use kind::{Kind,Field,Primitive, Composite};

use svg::Document;


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




fn main()
{

use Primitive::*;

let uint8_t  = Kind::prim(U8);
let uint16_t = Kind::prim(U16);
let uint32_t = Kind::prim(U32);


let point = Kind::comp(Composite {
    name: "Point".to_string(),
    mode: kind::CompositeMode::Product,
    fields: vec![
        Field{name: Some("x".to_string()),kind:&uint16_t},
        Field{name: Some("gorp".to_string()),kind:&uint16_t},
        Field{name: Some("z".to_string()),kind:&uint16_t},
    ]
});


let triangle = Kind::comp(Composite {
    name: "Line".to_string(),
    mode: kind::CompositeMode::Product,
    fields: vec![
        Field{name: Some("a".to_string()),kind:&point},
        Field{name: Some("b".to_string()),kind:&point},
        Field{name: Some("c".to_string()),kind:&point},
    ]
});

let tetra = Kind::comp(Composite {
    name: "LineUnion".to_string(),
    mode: kind::CompositeMode::Sum,
    fields: vec![
        Field{name: Some("p".to_string()),kind:&triangle},
        Field{name: Some("r".to_string()),kind:&triangle},
    ]
});




let spec = &block_draw::BlockDrawSpec {
    char_dims    : block_draw::Vec2{ x: 10.0, y: 16.0},
    text_pads    : block_draw::Vec2{ x:  4.0, y:  4.0},
    label_pads   : block_draw::Vec2{ x:  6.0, y:  4.0},
    union_xpad   : 6.0,
    fill_inset   : 3.0,
    prong_width  : 12.0,
    prong_xpad   : 6.0,
    chamfer_size : 12.0
};

let diag = spec.make_plan(
        &tetra,
        Vec2::new(0f32,0f32),
        None,
        false,
    )
    .into_svg()
    .set("transform","translate(16,16)");


let document = Document::new()
    .set("viewBox", (0, 0, 1100, 800))
    .add(diag);

svg::save("image.svg", &document).unwrap();
}
