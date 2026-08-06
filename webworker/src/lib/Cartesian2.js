export default class Cartesian2 {
    constructor(x, y) {
        this.x = x;
        this.y = y
    }
    scale(scale) {
        this.x *= scale;
        this.y *= scale;
    }
    scale_and_add(scale, other) {
        this.x = this.x * scale.x + other.x;
        this.y = this.y * scale.y + other.y;
    }
}