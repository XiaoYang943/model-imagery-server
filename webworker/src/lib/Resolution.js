import Cartographic from "./Cartographic"
import EllipsoidGeodesic from "./EllipsoidGeodesic"
import DeveloperError from "./DeveloperError.js";
/**
 * 获取某一层在赤道处的地图分辨率
 * @param {number} level 
 */
function getLevelResolution(level, tileSize) {
    const xNumAtZeroLevel = 2
    const yNumAtZeroLevel = 1
    //6378137.0, 6378137.0, 6356752.3142451793
    const earthRadius = 6378137
    return (2 * Math.PI * earthRadius) / (tileSize * (xNumAtZeroLevel << level))
}
const MAX_LEVEL = 30
/**
 * 计算分辨率
 * quality的意思是矩形的宽度等于几个256，用来确定多边形的分辨率。分辨率=矩形宽度(米)/(quality*256像素)
 */
export default class Resolution {
    constructor(tileSize = 256, quality = 0.8) {
        this.tileSize = tileSize
        this.quality = quality
        this.geodesic = new EllipsoidGeodesic();
        const xNumAtZeroLevel = 2
        const earthRadius = 6378137
        this.metersPerPixelAtZeroLevel = (2 * Math.PI * earthRadius) / (this.tileSize * xNumAtZeroLevel)
    }
    get tileSizeWithQuality() {
        return Math.ceil(this.tileSize * this.quality)
    }
    computeLevel(tileRectangle, epsg, maxLevel) {
        const midLatitude = (tileRectangle.south + tileRectangle.north) / 2
        const startPoint = new Cartographic(tileRectangle.west, midLatitude)
        const endPoint = new Cartographic(tileRectangle.east, midLatitude)
        this.geodesic.setEndPoints(startPoint, endPoint);
        let level = (this.metersPerPixelAtZeroLevel * Math.cos(midLatitude)) / (this.geodesic.surfaceDistance / this.tileSizeWithQuality)
        level = Math.log(level) / Math.log(2)
        level = Math.round(level) | 0
        if (level > maxLevel) {
            level == maxLevel
        }
        return level
    }
    computeLevel2(tileRectangle, epsg, maxLevel) {
        let tempScheme = null;
        let initRadius = Math.PI;

        if (epsg === 3857) {
            initRadius = Math.PI * 2;
        }
        if (epsg === 4326) {
            initRadius = Math.PI;
        }

        let boundingBox = tileRectangle
        let west_north_point = [boundingBox.west, boundingBox.north];
        let east_south_point = [boundingBox.east, boundingBox.south];

        let gap_lon = boundingBox.east - boundingBox.west;
        let level = 0;

        let isSkip = false;
        while (true && !isSkip) {
            let currentLevelGap = initRadius / Math.pow(2, level);
            let nextLevelGap = initRadius / Math.pow(2, level + 1);

            if (gap_lon <= currentLevelGap && gap_lon >= nextLevelGap) {
                level = level + 1;
                isSkip = true;
            } else {
                level++;
            }
        }
        if (level > maxLevel) {
            level = maxLevel;
        }
        return level
    }
}