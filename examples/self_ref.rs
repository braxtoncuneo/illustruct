

use illustruct::{
    kind::{
        Kind,
        Primitive,
        PrimValue,
        composite::{
            Field,
            CompositeMode,
        }, reference::ReferenceMode,
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
        char_dims:  Vec2::new(  5.0,  8.0),
        text_pads:  Vec2::new(  2.0,  2.0),
        label_pads: Vec2::new(  5.0,  2.0),
        union_xpad: 3.0,
        fill_inset: 1.5,
        prong_width: 6.0,
        prong_xpad:  3.0,
        chamfer_size : 6.0
    };

    let data = Kind::alias("data",&float);
    let link = Kind::comp("link",CompositeMode::Product,Vec::new());
    let link_ptr = Kind::refr(ReferenceMode::Ptr, &link);
    link.add_field("data", &data);
    link.add_field("next", &link_ptr);


    let mut ribbon = MemRibbon::new(0x0);
    ribbon.span(
        "span",
        vec![
            Field::anon(&link),
            Field::anon(&link),
            Field::anon(&link),
            Field::anon(&link),
        ]
    );

    for i in 0..4 {
        let adr = 8*i;
        ribbon.write_at(adr, PrimValue::Size((adr+8) as u32));
    }

    for i in 0..4 {
        let adr = 8*i;
        ribbon.write_at(adr+4, PrimValue::F32(i as f32));
    }

    ribbon.save_svg("links.svg", spec, true,true);



}
