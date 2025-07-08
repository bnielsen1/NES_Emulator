pub struct Frame {
    pub data: Vec<u8>
}

impl Frame {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 240;

    pub fn new() -> Self {
        Frame {
            data: vec![0; Frame::WIDTH * Frame::HEIGHT * 3] // dimensions of screen * 3 colors per pixel    
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        let actual_coord = (y * (Frame::WIDTH * 3)) + (x * 3);
        if actual_coord + 2 < self.data.len() {
            self.data[actual_coord] = color.0;
            self.data[actual_coord + 1] = color.1;
            self.data[actual_coord + 2] = color.2;
        }
    }
}