

use illustruct::{
    kind::{
        Kind,
        Primitive,
        PrimValue,
        composite::Field,
    },
    block_draw::{
        BlockDrawSpec,
        util::Vec2,
    },
    mem_ribbon::MemRibbon,
};


#[allow(unused_variables)]
fn main() {
    use Primitive::*;

    let uint8_t  = Kind::from(U8);
    let byte     = Kind::alias("byte", &uint8_t);

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

    let mut ram_rib = MemRibbon::new(0);
    ram_rib.span(
        "RAM_start",
        vec![
            Field::new("0",&byte),
            Field::new("1",&byte),
            Field::new("2",&byte),
            Field::new("3",&byte),
            Field::new("4",&byte),
            Field::new("5",&byte),
            Field::new("6",&byte),
            Field::new("7",&byte),
        ]
    );
    ram_rib.ellipse(0);
    ram_rib.span(
        "RAM_end",
        vec![
            Field::new("N-4",&byte),
            Field::new("N-3",&byte),
            Field::new("N-2",&byte),
            Field::new("N-1",&byte),
        ]
    );

    for (i,ch) in "SomeCoolData".chars().enumerate() {
        ram_rib.write_at(i, PrimValue::Char(ch as u8))
    }

    ram_rib.save_svg("RAM_illust.svg",spec,true,true);

}
