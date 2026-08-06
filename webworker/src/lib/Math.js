import DeveloperError from "./DeveloperError"
import defined from "./defined.js";
import defaultValue from "./defaultValue.js";
const CesiumMath = {};
CesiumMath.PI = Math.PI;
CesiumMath.RADIANS_PER_DEGREE = Math.PI / 180.0;
CesiumMath.TWO_PI = 2.0 * Math.PI;
CesiumMath.EPSILON1 = 0.1;
CesiumMath.EPSILON14 = 0.00000000000001;
CesiumMath.EPSILON12 = 0.000000000001;
CesiumMath.PI_OVER_TWO = Math.PI / 2.0;
CesiumMath.DEGREES_PER_RADIAN = 180.0 / Math.PI;
CesiumMath.toRadians = function (degrees) {
    //>>includeStart('debug', pragmas.debug);
    if (!defined(degrees)) {
        throw new DeveloperError("degrees is required.");
    }
    //>>includeEnd('debug');
    return degrees * CesiumMath.RADIANS_PER_DEGREE;
};

/**
 * Produces an angle in the range -Pi <= angle <= Pi which is equivalent to the provided angle.
 *
 * @param {number} angle in radians
 * @returns {number} The angle in the range [<code>-CesiumMath.PI</code>, <code>CesiumMath.PI</code>].
 */
CesiumMath.negativePiToPi = function (angle) {
    //>>includeStart('debug', pragmas.debug);
    if (!defined(angle)) {
        throw new DeveloperError("angle is required.");
    }
    //>>includeEnd('debug');
    if (angle >= -CesiumMath.PI && angle <= CesiumMath.PI) {
        // Early exit if the input is already inside the range. This avoids
        // unnecessary math which could introduce floating point error.
        return angle;
    }
    return CesiumMath.zeroToTwoPi(angle + CesiumMath.PI) - CesiumMath.PI;
};

/**
 * Produces an angle in the range 0 <= angle <= 2Pi which is equivalent to the provided angle.
 *
 * @param {number} angle in radians
 * @returns {number} The angle in the range [0, <code>CesiumMath.TWO_PI</code>].
 */
CesiumMath.zeroToTwoPi = function (angle) {
    //>>includeStart('debug', pragmas.debug);
    if (!defined(angle)) {
        throw new DeveloperError("angle is required.");
    }
    //>>includeEnd('debug');
    if (angle >= 0 && angle <= CesiumMath.TWO_PI) {
        // Early exit if the input is already inside the range. This avoids
        // unnecessary math which could introduce floating point error.
        return angle;
    }
    const mod = CesiumMath.mod(angle, CesiumMath.TWO_PI);
    if (
        Math.abs(mod) < CesiumMath.EPSILON14 &&
        Math.abs(angle) > CesiumMath.EPSILON14
    ) {
        return CesiumMath.TWO_PI;
    }
    return mod;
};
CesiumMath.mod = function (m, n) {
    //>>includeStart('debug', pragmas.debug);
    if (!defined(m)) {
        throw new DeveloperError("m is required.");
    }
    if (!defined(n)) {
        throw new DeveloperError("n is required.");
    }
    if (n === 0.0) {
        throw new DeveloperError("divisor cannot be 0.");
    }
    //>>includeEnd('debug');
    if (CesiumMath.sign(m) === CesiumMath.sign(n) && Math.abs(m) < Math.abs(n)) {
        // Early exit if the input does not need to be modded. This avoids
        // unnecessary math which could introduce floating point error.
        return m;
    }

    return ((m % n) + n) % n;
};
CesiumMath.sign = defaultValue(Math.sign, function sign(value) {
    value = +value; // coerce to number
    if (value === 0 || value !== value) {
        // zero or NaN
        return value;
    }
    return value > 0 ? 1 : -1;
});

export default CesiumMath

/**
 * Constraint a value to lie between two values.
 *
 * @param {number} value The value to clamp.
 * @param {number} min The minimum value.
 * @param {number} max The maximum value.
 * @returns {number} The clamped value such that min <= result <= max.
 */
CesiumMath.clamp = function (value, min, max) {
    //>>includeStart('debug', pragmas.debug);
    Check.typeOf.number("value", value);
    Check.typeOf.number("min", min);
    Check.typeOf.number("max", max);
    //>>includeEnd('debug');

    return value < min ? min : value > max ? max : value;
};

/**
 * Determines if two values are equal using an absolute or relative tolerance test. This is useful
 * to avoid problems due to roundoff error when comparing floating-point values directly. The values are
 * first compared using an absolute tolerance test. If that fails, a relative tolerance test is performed.
 * Use this test if you are unsure of the magnitudes of left and right.
 *
 * @param {number} left The first value to compare.
 * @param {number} right The other value to compare.
 * @param {number} [relativeEpsilon=0] The maximum inclusive delta between <code>left</code> and <code>right</code> for the relative tolerance test.
 * @param {number} [absoluteEpsilon=relativeEpsilon] The maximum inclusive delta between <code>left</code> and <code>right</code> for the absolute tolerance test.
 * @returns {boolean} <code>true</code> if the values are equal within the epsilon; otherwise, <code>false</code>.
 *
 * @example
 * const a = Cesium.Math.equalsEpsilon(0.0, 0.01, Cesium.Math.EPSILON2); // true
 * const b = Cesium.Math.equalsEpsilon(0.0, 0.1, Cesium.Math.EPSILON2);  // false
 * const c = Cesium.Math.equalsEpsilon(3699175.1634344, 3699175.2, Cesium.Math.EPSILON7); // true
 * const d = Cesium.Math.equalsEpsilon(3699175.1634344, 3699175.2, Cesium.Math.EPSILON9); // false
 */
CesiumMath.equalsEpsilon = function (
    left,
    right,
    relativeEpsilon,
    absoluteEpsilon
) {
    //>>includeStart('debug', pragmas.debug);
    if (!defined(left)) {
        throw new DeveloperError("left is required.");
    }
    if (!defined(right)) {
        throw new DeveloperError("right is required.");
    }
    //>>includeEnd('debug');

    relativeEpsilon = defaultValue(relativeEpsilon, 0.0);
    absoluteEpsilon = defaultValue(absoluteEpsilon, relativeEpsilon);
    const absDiff = Math.abs(left - right);
    return (
        absDiff <= absoluteEpsilon ||
        absDiff <= relativeEpsilon * Math.max(Math.abs(left), Math.abs(right))
    );
};