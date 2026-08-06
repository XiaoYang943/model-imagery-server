pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}
impl Vec2 {
    pub fn new(x: f64, y: f64) -> Vec2 {
        return Vec2 { x: x, y: y };
    }
    pub fn mul(&self, other: &Vec2) -> Vec2 {
        return Vec2::new(self.x * other.x, self.y * other.y);
    }
    pub fn scale(&mut self, scale: f64) {
        self.x *= scale;
        self.y *= scale;
    }
    pub fn scale_and_add(&mut self, scale: &Vec2, other: &Vec2) {
        self.x = self.x * scale.x + other.x;
        self.y = self.y * scale.y + other.y;
    }
}
