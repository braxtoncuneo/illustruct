

use illustruct::{
    kind::{
        Kind,
        Primitive,
        PrimValue,
        composite::{
            Field,
            CompositeMode,
        },
    },
    block_draw::{
        BlockDrawSpec,
        util::Vec2,
    },
    mem_ribbon::MemRibbon,
    access::Access,
};


#[allow(unused_variables)]
fn main() {
    use Primitive::*;

    let uint8_t   = Kind::from(U8);
    let uint16_t  = Kind::from(U16);
    let float     = Kind::from(F32);

    let spec = &BlockDrawSpec {
        char_dims:  Vec2::new(10.0, 16.0),
        text_pads:  Vec2::new( 4.0,  4.0),
        label_pads: Vec2::new( 10.0,  4.0),
        union_xpad: 6.0,
        fill_inset: 3.0,
        prong_width: 12.0,
        prong_xpad: 6.0,
        chamfer_size : 12.0
    };

    Kind::comp(
        "RPG_Character",
        CompositeMode::Product,
        vec![
            Field::new("level",&uint8_t),
            Field::new("brain",&uint8_t),
            Field::new("brawn",&uint8_t),
            Field::new("style",&uint8_t),
            Field::new("experience",&uint16_t),
            Field::new("hit_points",&uint16_t),
        ],
    ).into_ribbon()
    .save_svg("rpg.svg", spec, false, true);

    let point = Kind::comp(
        "Point",
        CompositeMode::Product,
        vec![
            Field::new("x",&float),
            Field::new("y",&float),
        ],
    );
    point.into_ribbon()
    .save_svg("point.svg", spec, false, true);


    Kind::comp(
        "Triangle",
        CompositeMode::Product,
        vec![
            Field::new("a",&point),
            Field::new("b",&point),
            Field::new("c",&point),
        ],
    ).into_ribbon()
    .save_svg("triangle.svg", spec, false, true);

}
