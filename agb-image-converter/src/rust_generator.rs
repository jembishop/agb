use crate::palette16::Palette16OptimisationResults;
use crate::Colour;
use crate::{add_image_256_to_tile_data, add_image_to_tile_data, collapse_to_4bpp, TileSize};
use crate::{image_loader::Image, ByteString};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use std::collections::{HashMap, HashSet};
use std::iter;

pub(crate) fn generate_palette_code(
    results: &Palette16OptimisationResults,
    crate_prefix: &str,
    palette_mapping: HashMap<Colour, String>,
) -> TokenStream {
    let crate_prefix = format_ident!("{}", crate_prefix);

    let current_colours: HashSet<Colour> = results
        .optimised_palettes
        .iter()
        .flat_map(|palette| palette.colours().map(|x| *x))
        .collect();

    let mut missing_colours: Vec<_> = palette_mapping
        .keys()
        .filter(|x| !current_colours.contains(x))
        .map(|x| *x)
        .into_iter()
        .collect();

    let padded_palettes: Vec<Vec<Colour>> = results
        .optimised_palettes
        .iter()
        .map(|palette| {
            let num_colours_to_map = core::cmp::min(16 - palette.len(), missing_colours.len());
            palette
                .clone()
                .into_iter()
                // If we have colours we want to map that are not in
                // the palette, then insert them into available spaces
                .chain(missing_colours.drain(..num_colours_to_map))
                .chain(iter::repeat(Colour {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                }))
                .take(16)
                .collect()
        })
        .collect();

    if missing_colours.len() != 0 {
        panic!("Not enough space left in palette for mapped colours!")
    }

    let palettes = padded_palettes.iter().map(|palette| {
        let colours = palette
            .clone()
            .into_iter()
            .map(|colour| colour.to_rgb15() as u16);

        quote! {
            #crate_prefix::display::palette16::Palette16::new([
                #(#colours),*
            ])
        }
    });

    // Generate indices for mapped colours, ignoring duplicates
    let mut found_colours = HashSet::new();
    let mapped_colours =
        padded_palettes
            .iter()
            .flatten()
            .enumerate()
            .filter_map(|(idx, colour)| {
                palette_mapping.get(colour).and_then(|name| {
                    found_colours.insert(name).then(|| {
                        let ident = format_ident!("{}", name.clone());
                        quote!(pub const #ident: usize = #idx;)
                    })
                })
            });

    quote! {
        pub const PALETTES: &[#crate_prefix::display::palette16::Palette16] = &[#(#palettes),*];
        pub mod mapped_colours {
            #(#mapped_colours)*
        }
    }
}

pub(crate) fn generate_code(
    output_variable_name: &str,
    results: &Palette16OptimisationResults,
    image: &Image,
    image_filename: &str,
    tile_size: TileSize,
    crate_prefix: String,
    assignment_offset: Option<usize>,
) -> TokenStream {
    let crate_prefix = format_ident!("{}", crate_prefix);
    let output_variable_name = format_ident!("{}", output_variable_name);

    let (tile_data, assignments) = if let Some(assignment_offset) = assignment_offset {
        let mut tile_data = Vec::new();

        add_image_to_tile_data(&mut tile_data, image, tile_size, results, assignment_offset);

        let tile_data = collapse_to_4bpp(&tile_data);

        let num_tiles = image.width * image.height / tile_size.to_size().pow(2);

        let assignments = results
            .assignments
            .iter()
            .skip(assignment_offset)
            .take(num_tiles)
            .map(|&x| x as u8)
            .collect();

        (tile_data, assignments)
    } else {
        let mut tile_data = Vec::new();

        add_image_256_to_tile_data(&mut tile_data, image, tile_size, results);

        (tile_data, vec![])
    };

    let data = ByteString(&tile_data);

    quote! {
        #[allow(non_upper_case_globals)]
        pub const #output_variable_name: #crate_prefix::display::tile_data::TileData = {
            const _: &[u8] = include_bytes!(#image_filename);

            const TILE_DATA: &[u8] = {
                pub struct AlignedAs<Align, Bytes: ?Sized> {
                    pub _align: [Align; 0],
                    pub bytes: Bytes,
                }

                const ALIGNED: &AlignedAs<u16, [u8]> = &AlignedAs {
                    _align: [],
                    bytes: *#data,
                };

                &ALIGNED.bytes
            };

            const PALETTE_ASSIGNMENT: &[u8] = &[
                #(#assignments),*
            ];

            #crate_prefix::display::tile_data::TileData::new(TILE_DATA, PALETTE_ASSIGNMENT)
        };
    }
}
