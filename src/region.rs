pub struct Region {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

const BASE_RESOLUTION: (i32, i32) = (1920, 1080);
const MAIN_MENU: Region = Region {
    left: 130,
    right: 358,
    top: 218,
    bottom: 518,
};

impl Region {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}
