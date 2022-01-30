pub struct Region {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

<<<<<<< HEAD
type Resolution = (i32, i32);
const BASE_RESOLUTION: Resolution = (2048, 1152);
pub const MAIN_MENU_DUEL: Region = Region {
    left: 140,
    right: 390,
    top: 225,
    bottom: 300,
};
pub const CARD_NAME_DECK_EDIT: Region = Region {
    left: 59,
    right: 424,
    top: 168,
    bottom: 201,
=======
const BASE_RESOLUTION: (i32, i32) = (1920, 1080);
const MAIN_MENU: Region = Region {
    left: 130,
    right: 358,
    top: 218,
    bottom: 518,
>>>>>>> d887a974c5ece7fea1b0b0e12a973446158e7cd3
};

impl Region {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
<<<<<<< HEAD
    pub fn calc_by_resolution(&self, resolution: Resolution) -> Self {
        let x_base = resolution.1 / BASE_RESOLUTION.0;
        let y_base = resolution.1 / BASE_RESOLUTION.1;
        Region {
            left: self.left * x_base,
            right: self.right * x_base,
            top: self.top * y_base,
            bottom: self.bottom * y_base,
        }
    }
=======
>>>>>>> d887a974c5ece7fea1b0b0e12a973446158e7cd3
}
