#![no_std]
#![no_main]

use agb::{
    display::{
        tiled::{TileFormat, TileSet, TileSetting},
        Priority,
    },
    include_gfx,
};

include_gfx!("examples/water_tiles.toml");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (gfx, mut vram) = gba.display.video.tiled0();
    let vblank = agb::interrupt::VBlank::get();

    let tileset = TileSet::new(water_tiles::water_tiles.tiles, TileFormat::FourBpp);
    let tileset_ref = vram.add_tileset(tileset);

    vram.set_background_palettes(water_tiles::water_tiles.palettes);

    let mut bg = gfx.background(Priority::P0);

    for y in 0..20u16 {
        for x in 0..30u16 {
            bg.set_tile(
                &mut vram,
                (x, y).into(),
                tileset_ref,
                TileSetting::new(0, false, false, 0),
            );
        }
    }

    bg.commit();
    bg.show();

    let mut i = 0;
    loop {
        i = (i + 1) % 8;

        vram.replace_tile(tileset_ref, 0, tileset_ref, i);

        vblank.wait_for_vblank();
    }
}