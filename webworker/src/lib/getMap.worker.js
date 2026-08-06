import registerPromiseWorker from 'promise-worker/register'
import Rectangle from './Rectangle.js';
import RectangleTileTexture from '../lib/RectangleTileTexture.js'
import { imageCache } from './ImageCache.js'
import Resolution from './Resolution.js';
// self.imageCache = new Map();
const TILE_SIZE = 256
function GetMap(tmpUrl, bbox, epsg, maxLevel, levelOffset) {
    let boundingBox = new Rectangle.fromRadians(Number(bbox[0]), Number(bbox[1]), Number(bbox[2]), Number(bbox[3]));
    const resolution = new Resolution(TILE_SIZE, 0.8)
    const level = resolution.computeLevel(boundingBox, epsg, maxLevel)
    let urls = tmpUrl.split(/,(?=\s*http)/).map(x=>x.trim());
    const task = new RectangleTileTexture(urls, TILE_SIZE, epsg, levelOffset, true)
    return task.requestImage(boundingBox, level)
}
registerPromiseWorker(async function (requestUrl) {
    let url = new URL(requestUrl)
    let bbox = url.searchParams.get("bbox")
    let bboxList = bbox.split(",")
    let tmpUrl = url.searchParams.get("url")
    tmpUrl = decodeURIComponent(tmpUrl);
    let epsg = Number(url.searchParams.get("epsg"))
    let maxLevel = Number(url.searchParams.get("maxLevel"))
    let levelOffset = Number(url.searchParams.get("levelOffset"))

    let key = makeKey(`${url}_${bboxList[0]},${bboxList[1]},${bboxList[2]},${bboxList[3]}_${maxLevel}_${levelOffset}`);
    let cached = imageCache.get(key);
    if (cached) {
        return cached
    } else {
        let res = await GetMap(tmpUrl, bboxList, epsg, maxLevel, levelOffset);
        imageCache.add(key, res);
        return res
    }
});

function makeKey(keyStr) {
    var hash = 0, i, chr;
    if (keyStr.length === 0) return hash;
    for (i = 0; i < keyStr.length; i++) {
        chr = keyStr.charCodeAt(i);
        hash = ((hash << 5) - hash) + chr;
        hash |= 0; // Convert to 32bit integer
    }
    return hash;
}