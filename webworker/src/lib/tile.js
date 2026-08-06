
import Cartesian2 from './Cartesian2'
import DeveloperError from "./DeveloperError"
import defaultValue from "./defaultValue.js";
import Rectangle from './Rectangle'

const imageryBoundsScratch = new Rectangle();
const tileImageryBoundsScratch = new Rectangle();
const clippedRectangleScratch = new Rectangle();
export class Tile {
    constructor(x, y, level, translation, scale, clip, rectangle, imageryLayerNum) {
        this.x = x
        this.y = y
        this.level = level
        this.translation = translation
        this.scale = scale
        this.clip = clip
        this.images = new Array(imageryLayerNum);
        this.rectangle = rectangle
    }
}
function findValue(tiles) {
    let minx = 999999999;
    let miny = 999999999;
    let maxx = 0;
    let maxy = 0;
    for (let tile of tiles) {
        if (tile.x > maxx) {
            maxx = tile.x;
        }
        if (tile.y > maxy) {
            maxy = tile.y;
        }
        if (tile.x < minx) {
            minx = tile.x;
        }
        if (tile.y < miny) {
            miny = tile.y;
        }
    }
    return { minx, miny, maxx, maxy };
}
export class Tiles {
    constructor(level, layer_num) {
        this.level = level
        this.layer_num = layer_num
        this.data = []
    }
    push(tile) { this.data.push(tile) }
    len() { return this.data.length }
    pushFinish() {
        let { minx, miny, maxx, maxy } = findValue(this.data);
        this.minx = minx;
        this.miny = miny;
        this.maxx = maxx;
        this.maxy = maxy;
        this.x_num = maxx - minx + 1;
        this.y_num = maxy - miny + 1;
    }
}
export function getCoordsOfLayer(tileRectangle, tilingScheme, imageryLevel, imageryLayerRectangle, imageryLayerNum, isBlend) {
    let imageryBounds = imageryLayerRectangle ? imageryLayerRectangle : Rectangle.MAX_VALUE;
    // let imageryBounds
    // if (imageryLayerRectangle) {
    //     imageryBounds = Rectangle.intersection(
    //         imageryProvider.rectangle,
    //         imageryLayer._rectangle,
    //         imageryBoundsScratch
    //     );
    // } else {
    //     imageryBounds = Rectangle.MAX_VALUE
    // }
    let rectangle = Rectangle.intersection(
        tileRectangle,
        imageryBounds,
        tileImageryBoundsScratch
    );
    const northwestTileCoordinates = tilingScheme.positionToTileXY(
        Rectangle.northwest(rectangle),
        imageryLevel
    );
    const southeastTileCoordinates = tilingScheme.positionToTileXY(
        Rectangle.southeast(rectangle),
        imageryLevel
    );
    let tiles = new Tiles(imageryLevel, imageryLayerNum)
    for (let i = northwestTileCoordinates.x; i <= southeastTileCoordinates.x; i++) {
        for (let j = northwestTileCoordinates.y; j <= southeastTileCoordinates.y; j++) {
            const tile = new Tile(i, j, imageryLevel);
            const imageryRectangle = tilingScheme.tileXYToRectangle(tile.x, tile.y, tile.level);
            const ts = calculateTranslationAndScale(imageryRectangle, tileRectangle, !isBlend)
            const translation = new Cartesian2(ts[0], ts[1])
            const scale = new Cartesian2(ts[2], ts[3])
            const clip = calculateClip(imageryRectangle, tileRectangle, !isBlend)
            tiles.push(new Tile(
                i, j, imageryLevel, translation, scale, clip, imageryRectangle, imageryLayerNum
            ))
        }
    }
    tiles.pushFinish()
    return tiles
}

export function calculateTranslationAndScale(imageryRectangle, tileRectangle, flipY = true) {
    const terrainWidth = tileRectangle.width;
    const terrainHeight = tileRectangle.height;
    const scaleX = terrainWidth / imageryRectangle.width;
    const scaleY = terrainHeight / imageryRectangle.height;
    let y = flipY ? (tileRectangle.south - imageryRectangle.south) : (imageryRectangle.north - tileRectangle.north)
    const translationX = (scaleX * (tileRectangle.west - imageryRectangle.west)) / terrainWidth
    const translationY = (scaleY * y) / terrainHeight
    return [translationX, translationY, scaleX, scaleY]
}

export function calculateClip(imageryRectangle, tileRectangle, flipY = true) {
    const terrainWidth = tileRectangle.width;
    const terrainHeight = tileRectangle.height;
    const intersection = Rectangle.intersection(
        tileRectangle,
        imageryRectangle,
        clippedRectangleScratch
    )
    const width = intersection.width / terrainWidth
    const height = intersection.height / terrainHeight
    const minX = (intersection.west - tileRectangle.west) / terrainWidth
    let y2 = flipY ? (intersection.south - tileRectangle.south) : (tileRectangle.north - intersection.north)
    const minY = y2 / terrainHeight
    return [minX, minY, minX + width, minY + height]
}