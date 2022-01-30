pub struct Region {
    pub left: i32,
    pub right: i32,
    pub top: i32,
    pub bottom: i32,
}

type Resolution = (i32, i32);
const BASE_RESOLUTION: Resolution = (2048, 1152);
pub const MAIN_MENU_DUEL: Region = Region {
    left: 140,
    right: 390,
    top: 225,
    bottom: 300,
};

impl Region {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
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
}
