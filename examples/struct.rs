use illustruct::{
    kind::{
        Kind,
        primitive::Primitive,
        composite::Composite,
    },
    block_draw::{
        BlockDrawSpec,
        util::Vec2,
    },
};


#[allow(unused_variables)]
fn main() {
    use Primitive::*;

    let uint8_t   = Kind::from(U8);
    let uint16_t  = Kind::from(U16);
    let float     = Kind::from(F32);

    let spec = &BlockDrawSpec {
        char_dims:  Vec2::new(  5.0,  8.0),
        text_pads:  Vec2::new(  2.0,  2.0),
        label_pads: Vec2::new(  5.0,  2.0),
        union_xpad: 3.0,
        fill_inset: 1.5,
        prong_width: 6.0,
        prong_xpad:  3.0,
        chamfer_size : 6.0
    };

    Kind::from(Composite::product(
        "RPG_Character",
        vec![
            uint8_t.field_named("level"),
            uint8_t.field_named("brain"),
            uint8_t.field_named("brawn"),
            uint8_t.field_named("style"),
            uint16_t.field_named("experience"),
            uint16_t.field_named("hit_points"),
        ],
    ))
        .to_ribbon()
        .save_svg("rpg.svg", spec, false, true);

    let point = Kind::from(Composite::product(
        "Point",
        vec![
            float.field_named("x"),
            float.field_named("y"),
        ],
    ));

    let point_rib = point.to_ribbon();
    point_rib.save_svg("point.svg", spec, false, true);


    let tri = Kind::from(Composite::product(
        "Triangle",
        vec![
            point.field_named("a"),
            point.field_named("b"),
            point.field_named("c"),
        ],
    ));

    let tri_rib = tri.to_ribbon();
    tri_rib.save_svg("triangle.svg", spec, false, true);
}
