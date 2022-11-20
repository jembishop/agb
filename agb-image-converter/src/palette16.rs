use crate::colour::Colour;
use std::collections::HashSet;

const MAX_COLOURS: usize = 256;
const MAX_COLOURS_PER_PALETTE: usize = 16;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Palette16 {
    colours: Vec<Colour>,
}

impl Palette16 {
    pub fn new() -> Self {
        Palette16 {
            colours: Vec::with_capacity(MAX_COLOURS_PER_PALETTE),
        }
    }

    pub fn add_colour(&mut self, colour: Colour) -> bool {
        if self.colours.contains(&colour) {
            return false;
        }

        if self.colours.len() == MAX_COLOURS_PER_PALETTE {
            panic!("Can have at most 16 colours in a single palette");
        }
        self.colours.push(colour);
        true
    }

    pub fn try_add_colour(&mut self, colour: Colour) -> bool {
        if self.colours.contains(&colour) {
            return true;
        }

        if self.colours.len() == MAX_COLOURS_PER_PALETTE {
            return false;
        }

        self.colours.push(colour);
        true
    }

    pub fn colour_index(&self, colour: Colour, transparent_colour: Option<Colour>) -> u8 {
        let colour_to_search = match (transparent_colour, colour.is_transparent()) {
            (Some(transparent_colour), true) => transparent_colour,
            _ => colour,
        };

        self.colours
            .iter()
            .position(|c| *c == colour_to_search)
            .unwrap_or_else(|| {
                panic!(
                    "Can't get a colour index without it existing, looking for {:?}, got {:?}",
                    colour, self.colours
                )
            }) as u8
    }

    pub fn colours(&self) -> impl Iterator<Item = &Colour> {
        self.colours.iter()
    }

    pub fn len(&self) -> usize {
        self.colours.len()
    }

    fn union_length(&self, other: &Palette16) -> usize {
        self.colours
            .iter()
            .chain(&other.colours)
            .collect::<HashSet<_>>()
            .len()
    }

    fn is_satisfied_by(&self, other: &Palette16) -> bool {
        self.colours
            .iter()
            .collect::<HashSet<_>>()
            .is_subset(&other.colours.iter().collect::<HashSet<_>>())
    }
}

impl IntoIterator for Palette16 {
    type Item = Colour;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.colours.into_iter()
    }
}

pub(crate) struct Palette16Optimiser {
    palettes: Vec<Palette16>,
    colours: Vec<Colour>,
    transparent_colour: Option<Colour>,
}

#[derive(Debug)]
pub(crate) struct Palette16OptimisationResults {
    pub optimised_palettes: Vec<Palette16>,
    pub assignments: Vec<usize>,
    pub transparent_colour: Option<Colour>,
}

impl Palette16Optimiser {
    pub fn new(transparent_colour: Option<Colour>) -> Self {
        Palette16Optimiser {
            palettes: vec![],
            colours: Vec::new(),
            transparent_colour,
        }
    }

    pub fn add_palette(&mut self, palette: Palette16) {
        self.palettes.push(palette.clone());

        for colour in palette.colours {
            if self.colours.contains(&colour) {
                continue;
            }

            self.colours.push(colour);
        }

        if self.colours.len() > MAX_COLOURS {
            panic!("Cannot have over 256 colours");
        }
    }

    pub fn optimise_palettes(&self) -> Palette16OptimisationResults {
        let mut assignments = vec![0; self.palettes.len()];
        let mut optimised_palettes = vec![];

        let mut unsatisfied_palettes = self
            .palettes
            .iter()
            .cloned()
            .collect::<HashSet<Palette16>>();

        while !unsatisfied_palettes.is_empty() {
            let palette = self.find_maximal_palette_for(&unsatisfied_palettes);

            for test_palette in unsatisfied_palettes.clone() {
                if test_palette.is_satisfied_by(&palette) {
                    unsatisfied_palettes.remove(&test_palette);
                }
            }

            for (i, overall_palette) in self.palettes.iter().enumerate() {
                if overall_palette.is_satisfied_by(&palette) {
                    assignments[i] = optimised_palettes.len();
                }
            }

            optimised_palettes.push(palette);

            if optimised_palettes.len() == MAX_COLOURS / MAX_COLOURS_PER_PALETTE {
                panic!("Failed to find covering palettes");
            }
        }

        Palette16OptimisationResults {
            optimised_palettes,
            assignments,
            transparent_colour: self.transparent_colour,
        }
    }

    fn find_maximal_palette_for(&self, unsatisfied_palettes: &HashSet<Palette16>) -> Palette16 {
        let mut palette = Palette16::new();

        palette.add_colour(
            self.transparent_colour
                .unwrap_or_else(|| Colour::from_rgb(255, 0, 255, 0)),
        );

        loop {
            let mut colour_usage = vec![0; MAX_COLOURS];
            let mut a_colour_is_used = false;

            for current_palette in unsatisfied_palettes {
                if palette.union_length(current_palette) > MAX_COLOURS_PER_PALETTE {
                    continue;
                }

                for colour in &current_palette.colours {
                    if palette.colours.contains(colour) {
                        continue;
                    }

                    if let Some(colour_index) = self.colours.iter().position(|c| c == colour) {
                        colour_usage[colour_index] += 1;
                        a_colour_is_used = true;
                    }
                }
            }

            if !a_colour_is_used {
                return palette;
            }

            let best_index = colour_usage
                .iter()
                .enumerate()
                .max_by(|(_, usage1), (_, usage2)| usage1.cmp(usage2))
                .unwrap()
                .0;

            let best_colour = self.colours[best_index];

            palette.add_colour(best_colour);
            if palette.colours.len() == MAX_COLOURS_PER_PALETTE {
                return palette;
            }
        }
    }
}
