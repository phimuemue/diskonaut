use ::tui::layout::Rect;

use crate::state::files::Folder;
use crate::state::tiles::files_in_folder::FileType;
use crate::state::tiles::{files_in_folder, FileMetadata, Tile, TreeMap, HV, HV::*};

pub struct Board {
    pub tiles: Vec<Tile>,
    pub unrenderable_tile_coordinates: Option<(u16, u16)>,
    pub selected_index: Option<usize>, // None means nothing is selected
    pub previous_indices_and_zoom_level: Vec<(Option<usize>, usize)>, // Stack of previous stats
    pub zoom_level: usize,
    area: Rect,
    files: Vec<FileMetadata>,
}

impl Board {
    pub fn new(folder: &Folder) -> Self {
        Board {
            tiles: vec![],
            unrenderable_tile_coordinates: None,
            files: files_in_folder(folder, 0),
            selected_index: None,
            previous_indices_and_zoom_level: vec![],
            zoom_level: 0,
            area: Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            },
        }
    }
    pub fn change_files(&mut self, folder: &Folder) {
        self.files = files_in_folder(folder, self.zoom_level);
        self.fill();
    }
    pub fn change_area(&mut self, area: &Rect) {
        if self.area != *area {
            self.area = *area;
            self.fill();
        }
    }
    fn fill(&mut self) {
        let mut tree_map = TreeMap::new(&self.area);
        tree_map.populate_tiles(self.files.iter().collect());
        self.tiles = tree_map.tiles;
        self.unrenderable_tile_coordinates = tree_map.unrenderable_tile_coordinates;
    }
    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index
    }
    pub fn set_selected_index(&mut self, next_index: &usize) {
        self.selected_index = Some(*next_index);
    }
    pub fn has_selected_index(&self) -> bool {
        self.selected_index.is_some()
    }
    pub fn reset_selected_index(&mut self) {
        self.selected_index = None;
    }
    pub fn currently_selected(&self) -> Option<&Tile> {
        match &self.selected_index {
            Some(selected_index) => self.tiles.get(*selected_index),
            None => None,
        }
    }
    pub fn pop_previous_index_and_zoom_level(&mut self) -> Option<(Option<usize>, usize)> {
        self.previous_indices_and_zoom_level.pop()
    }
    pub fn move_to_largest_folder(&mut self) {
        let next_index = self
            .tiles
            .iter()
            .enumerate()
            .filter(|(_, tile)| tile.file_type == FileType::Folder)
            .map(|(index, _)| index)
            .next();

        if let Some(index) = next_index {
            self.set_selected_index(&index);
        }
    }
    fn move_selected(&mut self, hv: HV, is_directly_beneath: impl Fn(&Tile, &Tile, HV) -> bool) {
        match self.currently_selected() {
            Some(currently_selected) => {
                let next_index = self
                    .tiles
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| {
                        is_directly_beneath(c, &currently_selected, hv)
                            && c.overlaps_with(&currently_selected, hv)
                    })
                    // get the index of the tile with the most overlap with currently selected
                    .max_by_key(|(_, c)| c.get_overlap_with(&currently_selected, hv))
                    .map(|(index, _)| index);
                match next_index {
                    Some(i) => self.set_selected_index(&i),
                    None => self.reset_selected_index(), // move off the edge of the screen resets selection
                }
            }
            None => self.set_selected_index(&0),
        }
    }
    pub fn move_selected_right(&mut self) {
        self.move_selected(Horizontal, Tile::is_directly_after)
    }
    pub fn move_selected_left(&mut self) {
        self.move_selected(Horizontal, Tile::is_directly_before)
    }
    pub fn move_selected_down(&mut self) {
        self.move_selected(Vertical, Tile::is_directly_after)
    }
    pub fn move_selected_up(&mut self) {
        self.move_selected(Vertical, Tile::is_directly_before)
    }
    pub fn zoom_in(&mut self, folder: &Folder) {
        if self.zoom_level < self.files.len() {
            self.zoom_level += 1;
            self.files = files_in_folder(folder, self.zoom_level);
            self.fill();
        }
    }
    pub fn zoom_out(&mut self, folder: &Folder) {
        if self.zoom_level > 0 {
            self.zoom_level -= 1;
            self.files = files_in_folder(folder, self.zoom_level);
            self.fill();
        }
    }
    pub fn reset_zoom(&mut self, folder: &Folder) {
        self.zoom_level = 0;
        self.files = files_in_folder(folder, self.zoom_level);
        self.fill();
    }
    pub fn reset_zoom_index(&mut self) {
        self.zoom_level = 0;
    }
    pub fn set_zoom_index(&mut self, index: usize) {
        self.zoom_level = index;
    }
    pub fn record_current_index_and_zoom_level(&mut self) {
        self.previous_indices_and_zoom_level
            .push((self.get_selected_index(), self.zoom_level));
    }
}
