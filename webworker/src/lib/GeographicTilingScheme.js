import Cartesian2 from "./Cartesian2.js";
import defaultValue from "./defaultValue.js";
import defined from "./defined.js";
import CesiumMath from "./Math.js";
import Rectangle from "./Rectangle.js";

function GeographicTilingScheme(options) {
    options = defaultValue(options, defaultValue.EMPTY_OBJECT);
    this._rectangle = defaultValue(options.rectangle, Rectangle.MAX_VALUE);
    this._numberOfLevelZeroTilesX = defaultValue(
        options.numberOfLevelZeroTilesX,
        2
    );
    this._numberOfLevelZeroTilesY = defaultValue(
        options.numberOfLevelZeroTilesY,
        1
    );
}
GeographicTilingScheme.prototype.getNumberOfXTilesAtLevel = function (level) {
    return this._numberOfLevelZeroTilesX << level;
};

GeographicTilingScheme.prototype.getNumberOfYTilesAtLevel = function (level) {
    return this._numberOfLevelZeroTilesY << level;
};

GeographicTilingScheme.prototype.rectangleToNativeRectangle = function (
    rectangle,
    result
) {
    const west = CesiumMath.toDegrees(rectangle.west);
    const south = CesiumMath.toDegrees(rectangle.south);
    const east = CesiumMath.toDegrees(rectangle.east);
    const north = CesiumMath.toDegrees(rectangle.north);
    if (!defined(result)) {
        return new Rectangle(west, south, east, north);
    }
    result.west = west;
    result.south = south;
    result.east = east;
    result.north = north;
    return result;
};
GeographicTilingScheme.prototype.tileXYToNativeRectangle = function (
    x,
    y,
    level,
    result
) {
    const rectangleRadians = this.tileXYToRectangle(x, y, level, result);
    rectangleRadians.west = CesiumMath.toDegrees(rectangleRadians.west);
    rectangleRadians.south = CesiumMath.toDegrees(rectangleRadians.south);
    rectangleRadians.east = CesiumMath.toDegrees(rectangleRadians.east);
    rectangleRadians.north = CesiumMath.toDegrees(rectangleRadians.north);
    return rectangleRadians;
};

GeographicTilingScheme.prototype.tileXYToRectangle = function (
    x,
    y,
    level,
    result
) {
    const rectangle = this._rectangle;

    const xTiles = this.getNumberOfXTilesAtLevel(level);
    const yTiles = this.getNumberOfYTilesAtLevel(level);

    const xTileWidth = rectangle.width / xTiles;
    const west = x * xTileWidth + rectangle.west;
    const east = (x + 1) * xTileWidth + rectangle.west;

    const yTileHeight = rectangle.height / yTiles;
    const north = rectangle.north - y * yTileHeight;
    const south = rectangle.north - (y + 1) * yTileHeight;

    if (!defined(result)) {
        result = new Rectangle(west, south, east, north);
    }

    result.west = west;
    result.south = south;
    result.east = east;
    result.north = north;
    return result;
};
GeographicTilingScheme.prototype.positionToTileXY = function (
    position,
    level,
    result
) {
    const rectangle = this._rectangle;
    if (!Rectangle.contains(rectangle, position)) {
        // outside the bounds of the tiling scheme
        return undefined;
    }

    const xTiles = this.getNumberOfXTilesAtLevel(level);
    const yTiles = this.getNumberOfYTilesAtLevel(level);

    const xTileWidth = rectangle.width / xTiles;
    const yTileHeight = rectangle.height / yTiles;

    let longitude = position.longitude;
    if (rectangle.east < rectangle.west) {
        longitude += CesiumMath.TWO_PI;
    }

    let xTileCoordinate = ((longitude - rectangle.west) / xTileWidth) | 0;
    if (xTileCoordinate >= xTiles) {
        xTileCoordinate = xTiles - 1;
    }

    let yTileCoordinate =
        ((rectangle.north - position.latitude) / yTileHeight) | 0;
    if (yTileCoordinate >= yTiles) {
        yTileCoordinate = yTiles - 1;
    }

    if (!defined(result)) {
        return new Cartesian2(xTileCoordinate, yTileCoordinate);
    }

    result.x = xTileCoordinate;
    result.y = yTileCoordinate;
    return result;
};
export default GeographicTilingScheme;
