
import CesiumMath from './Math'
import Cartographic from "./Cartographic"
import DeveloperError from "./DeveloperError"
import defined from "./defined.js";
import defaultValue from "./defaultValue.js";
export default function Rectangle(west, south, east, north) {
    this.west = defaultValue(west, 0.0);
    this.south = defaultValue(south, 0.0);
    this.east = defaultValue(east, 0.0);
    this.north = defaultValue(north, 0.0);
}

Object.defineProperties(Rectangle.prototype, {
    width: {
        get: function () {
            return Rectangle.computeWidth(this);
        },
    },
    height: {
        get: function () {
            return Rectangle.computeHeight(this);
        },
    },
});
Rectangle.packedLength = 4;
Rectangle.computeWidth = function (rectangle) {
    let east = rectangle.east;
    const west = rectangle.west;
    if (east < west) {
        east += CesiumMath.TWO_PI;
    }
    return east - west;
};
Rectangle.computeHeight = function (rectangle) {
    return rectangle.north - rectangle.south;
};
Rectangle.fromDegrees = function (west, south, east, north, result) {
    west = CesiumMath.toRadians(defaultValue(west, 0.0));
    south = CesiumMath.toRadians(defaultValue(south, 0.0));
    east = CesiumMath.toRadians(defaultValue(east, 0.0));
    north = CesiumMath.toRadians(defaultValue(north, 0.0));

    if (!defined(result)) {
        return new Rectangle(west, south, east, north);
    }

    result.west = west;
    result.south = south;
    result.east = east;
    result.north = north;

    return result;
};
Rectangle.fromRadians = function (west, south, east, north, result) {
    if (!defined(result)) {
        return new Rectangle(west, south, east, north);
    }

    result.west = defaultValue(west, 0.0);
    result.south = defaultValue(south, 0.0);
    result.east = defaultValue(east, 0.0);
    result.north = defaultValue(north, 0.0);

    return result;
};
Rectangle.intersection = function (rectangle, otherRectangle, result) {
    let rectangleEast = rectangle.east;
    let rectangleWest = rectangle.west;

    let otherRectangleEast = otherRectangle.east;
    let otherRectangleWest = otherRectangle.west;

    if (rectangleEast < rectangleWest && otherRectangleEast > 0.0) {
        rectangleEast += CesiumMath.TWO_PI;
    } else if (otherRectangleEast < otherRectangleWest && rectangleEast > 0.0) {
        otherRectangleEast += CesiumMath.TWO_PI;
    }

    if (rectangleEast < rectangleWest && otherRectangleWest < 0.0) {
        otherRectangleWest += CesiumMath.TWO_PI;
    } else if (otherRectangleEast < otherRectangleWest && rectangleWest < 0.0) {
        rectangleWest += CesiumMath.TWO_PI;
    }

    const west = CesiumMath.negativePiToPi(
        Math.max(rectangleWest, otherRectangleWest)
    );
    const east = CesiumMath.negativePiToPi(
        Math.min(rectangleEast, otherRectangleEast)
    );

    if (
        (rectangle.west < rectangle.east ||
            otherRectangle.west < otherRectangle.east) &&
        east <= west
    ) {
        return undefined;
    }

    const south = Math.max(rectangle.south, otherRectangle.south);
    const north = Math.min(rectangle.north, otherRectangle.north);

    if (south >= north) {
        return undefined;
    }

    if (!defined(result)) {
        return new Rectangle(west, south, east, north);
    }
    result.west = west;
    result.south = south;
    result.east = east;
    result.north = north;
    return result;
};
Rectangle.MAX_VALUE = Object.freeze(
    new Rectangle(
        -Math.PI,
        -CesiumMath.PI_OVER_TWO,
        Math.PI,
        CesiumMath.PI_OVER_TWO
    )
);
Rectangle.southwest = function (rectangle, result) {
    if (!defined(result)) {
        return new Cartographic(rectangle.west, rectangle.south);
    }
    result.longitude = rectangle.west;
    result.latitude = rectangle.south;
    result.height = 0.0;
    return result;
};
Rectangle.northwest = function (rectangle, result) {
    if (!defined(result)) {
        return new Cartographic(rectangle.west, rectangle.north);
    }
    result.longitude = rectangle.west;
    result.latitude = rectangle.north;
    result.height = 0.0;
    return result;
};
Rectangle.northeast = function (rectangle, result) {
    if (!defined(result)) {
        return new Cartographic(rectangle.east, rectangle.north);
    }
    result.longitude = rectangle.east;
    result.latitude = rectangle.north;
    result.height = 0.0;
    return result;
};
Rectangle.southeast = function (rectangle, result) {
    if (!defined(result)) {
        return new Cartographic(rectangle.east, rectangle.south);
    }
    result.longitude = rectangle.east;
    result.latitude = rectangle.south;
    result.height = 0.0;
    return result;
};
Rectangle.contains = function (rectangle, cartographic) {
    let longitude = cartographic.longitude;
    const latitude = cartographic.latitude;

    const west = rectangle.west;
    let east = rectangle.east;

    if (east < west) {
        east += CesiumMath.TWO_PI;
        if (longitude < 0.0) {
            longitude += CesiumMath.TWO_PI;
        }
    }
    return (
        (longitude > west ||
            CesiumMath.equalsEpsilon(longitude, west, CesiumMath.EPSILON14)) &&
        (longitude < east ||
            CesiumMath.equalsEpsilon(longitude, east, CesiumMath.EPSILON14)) &&
        latitude >= rectangle.south &&
        latitude <= rectangle.north
    );
};