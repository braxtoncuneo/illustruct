use illustruct::{
    kind::{
        Kind,
        primitive::Primitive,
        composite::{self, Field, Composite},
    },
    block_draw::{BlockDrawSpec, util::Vec2},
    mem_ribbon::MemRibbon,
};

#[allow(unused_variables)]
fn main() {
    use Primitive::*;

    let uint8_t  = Kind::from(U8);
    let uint16_t = Kind::from(U16);
    let uint32_t = Kind::from(U32);
    let uint64_t = Kind::from(U64);

    let spec = &BlockDrawSpec {
        char_dims:  Vec2::new(10.0, 16.0),
        text_pads:  Vec2::new( 4.0,  4.0),
        label_pads: Vec2::new(10.0,  4.0),
        union_xpad: 6.0,
        fill_inset: 3.0,
        prong_width: 12.0,
        prong_xpad: 6.0,
        chamfer_size: 12.0
    };

    let align_rib = MemRibbon::new(0).span(
        "RAM_start",
        vec![
            Field::new("byte_A", &uint8_t),
            Field::new("byte_B", &uint8_t),
            Field::new("two_bytes", &uint16_t),
            Field::new("four_bytes", &uint32_t),
            Field::new("eight_bytes", &uint64_t),
        ],
    );

    let unalign_rib = MemRibbon::new(0).span(
        "RAM_start",
        vec![
            Field::new("byte_A", &uint8_t),
            Field::new("byte_B", &uint8_t),
            Field::new("byte_C", &uint8_t),
            Field::new("two_bytes", &uint16_t),
            Field::new("four_bytes", &uint32_t),
            Field::new("eight_bytes", &uint64_t),
        ],
    );

    align_rib  .save_svg("align_illust.svg",spec,false,true);
    unalign_rib.save_svg("unalign_illust.svg",spec,false,true);

    let compact_struct = Kind::from(Composite::new(
        "CompactStruct",
        composite::Mode::Product,
        vec![
            Field::new(" first_byte",&uint8_t),
            Field::new("second_byte",&uint8_t),
            Field::new("  two_bytes",&uint16_t),
        ],
    ));

    let padded_struct = Kind::from(Composite::new(
        "PaddedStruct",
        composite::Mode::Product,
        vec![
            Field::new(" first_byte",&uint8_t),
            Field::new("  two_bytes",&uint16_t),
            Field::new("second_byte",&uint8_t),
        ],
    ));

    let comp_rib = compact_struct.into_ribbon();
    comp_rib.save_svg("compact_struct.svg", spec, false, true);

    padded_struct.into_ribbon()
        .save_svg("padded_struct.svg", spec, false, true);
}
