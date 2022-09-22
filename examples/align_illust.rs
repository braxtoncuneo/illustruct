use illustruct::{
    kind::{
        Kind,
        primitive::Primitive,
        composite::Composite,
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
            uint8_t.field_named("byte_A"),
            uint8_t.field_named("byte_B"),
            uint16_t.field_named("two_bytes"),
            uint32_t.field_named("four_bytes"),
            uint64_t.field_named("eight_bytes"),
        ],
    );

    let unalign_rib = MemRibbon::new(0).span(
        "RAM_start",
        vec![
            uint8_t.field_named("byte_A"),
            uint8_t.field_named("byte_B"),
            uint8_t.field_named("byte_C"),
            uint16_t.field_named("two_bytes"),
            uint32_t.field_named("four_bytes"),
            uint64_t.field_named("eight_bytes"),
        ],
    );

    align_rib  .save_svg("align_illust.svg", spec, false, true);
    unalign_rib.save_svg("unalign_illust.svg", spec, false, true);

    let compact_struct = Kind::from(Composite::product(
        "CompactStruct",
        vec![
            uint8_t.field_named(" first_byte"),
            uint8_t.field_named("second_byte"),
            uint16_t.field_named("  two_bytes"),
        ],
    ));

    let padded_struct = Kind::from(Composite::product(
        "PaddedStruct",
        vec![
            uint8_t.field_named(" first_byte"),
            uint16_t.field_named("  two_bytes"),
            uint8_t.field_named("second_byte"),
        ],
    ));

    let comp_rib = compact_struct.to_ribbon();
    comp_rib.save_svg("compact_struct.svg", spec, false, true);

    padded_struct.to_ribbon()
        .save_svg("padded_struct.svg", spec, false, true);
}
