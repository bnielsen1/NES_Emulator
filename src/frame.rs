pub struct Frame {
    pub data: Vec<u8>,
    pub transparency: Vec<bool>
}

impl Frame {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 240;

    pub fn new() -> Self {
        Frame {
            data: vec![0; Frame::WIDTH * Frame::HEIGHT * 3], // dimensions of screen * 3 colors per pixel
            transparency: vec![true; Frame::WIDTH * Frame::HEIGHT]    
        }
    }

    pub fn check_and_set(&mut self, trans: bool, priority: bool, x: usize, y: usize, color: (u8, u8, u8)) {
        let actual_coord = (y * (Frame::WIDTH * 3)) + (x * 3);
        if actual_coord + 2 < self.data.len() && (priority || self.transparency[actual_coord/3]){
            self.transparency[actual_coord/3] = trans;
            self.data[actual_coord] = color.0;
            self.data[actual_coord + 1] = color.1;
            self.data[actual_coord + 2] = color.2;
        }
    }

    pub fn set_pixel(&mut self, trans: bool, x: usize, y: usize, color: (u8, u8, u8)) {
        let actual_coord = (y * (Frame::WIDTH * 3)) + (x * 3);
        if actual_coord + 2 < self.data.len() {
            self.transparency[actual_coord/3] = trans;
            self.data[actual_coord] = color.0;
            self.data[actual_coord + 1] = color.1;
            self.data[actual_coord + 2] = color.2;
        }
    }
}