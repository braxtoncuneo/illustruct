use illustruct::{
    kind::{
        Kind,
        primitive::{Primitive, PrimValue},
        composite::{Field, Composite},
        reference::{self, Reference},
        Alias,
    },
    block_draw::{BlockDrawSpec, util::Vec2},
    mem_ribbon::MemRibbon,
};

#[allow(unused_variables)]
fn main() {
    let uint8_t  = Kind::from(Primitive::U8);
    let uint16_t = Kind::from(Primitive::U16);
    let float    = Kind::from(Primitive::F32);

    let spec = BlockDrawSpec {
        char_dims:  Vec2::new(5.0, 8.0),
        text_pads:  Vec2::new(2.0, 2.0),
        label_pads: Vec2::new(5.0, 2.0),
        union_xpad: 3.0,
        fill_inset: 1.5,
        prong_width: 6.0,
        prong_xpad: 3.0,
        chamfer_size: 6.0
    };

    let data = Kind::from(Alias::new("data", &float));
    let link = Kind::from(Composite::product("link", Vec::new()));
    let link_ptr = Kind::from(Reference::new(reference::Mode::Ptr, &link));
    link.add_field("data", &data);
    link.add_field("next", &link_ptr);

    let mut ribbon = MemRibbon::new(0x0)
        .span("span", std::iter::repeat(&link)
            .map(Field::anon)
            .take(4)
            .collect(),
        );

    for i in 0..4 {
        let adr = 8 * i;
        ribbon.write_at(adr, PrimValue::Size((adr + 8) as _));
    }

    for i in 0..4 {
        let adr = 8 * i;
        ribbon.write_at(adr + 4, PrimValue::F32(i as _));
    }

    ribbon.save_svg("links.svg", &spec, true,true);
}
