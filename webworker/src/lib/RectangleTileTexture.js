import Cartesian2 from './Cartesian2'
import DeveloperError from "./DeveloperError"
import defaultValue from "./defaultValue.js";
import Rectangle from './Rectangle'
import GeographicTilingScheme from './GeographicTilingScheme'
import WebMercatorTilingScheme from './WebMercatorTilingScheme'
import { Tile, Tiles, getCoordsOfLayer } from "./tile.js"
import { imageCache } from './ImageCache';
export default class RectangleTileTexture {
    /**
     * 
     * @param {Array<string>} urls 
     * @param {number} tileSize 默认256
     * @param {TilingScheme} tilingScheme 
     * @param {boolean} isBlend 是否直接混合，而不是在着色器内混合，混合的工作放到CPU完成
     */
    constructor(urls, tileSize, epsg, levelOffset, isBlend) {
        this.urls = urls
        this.isBlend = defaultValue(isBlend, true)
        this.tileSize = defaultValue(tileSize, 256)
        this.levelOffset = defaultValue(levelOffset, 0)
        this.epsg = defaultValue(epsg, 4326)
        if (epsg == 4326) {
            this.tilingScheme = new GeographicTilingScheme()
        } else if (epsg == 3857) {
            this.tilingScheme = new WebMercatorTilingScheme()
        } else {
            throw new DeveloperError("不支持投影方式,epsg=", epsg)
        }
        this.inWorker = typeof document == "undefined"
    }
    get imageryLayerNum() {
        return this.urls.length
    }
    get imageCount() {
        return this.tiles.data.length * this.imageryLayerNum
    }
    createTexture() {
        const imageCount = this.imageCount
        const tileSize = this.tileSize
        const width = tileSize
        const height = imageCount * this.tileSize
        let canvas
        if (this.inWorker) {
            canvas = new OffscreenCanvas(width, height)
        } else {
            canvas = document.createElement("canvas")
            canvas.width = width
            canvas.height = height
        }
        const ctx = canvas.getContext("2d")
        let imageInfoList = new ImageInfoList(
            this.imageryLayerNum,
        )
        for (let i = 0; i < this.tiles.data.length; i++) {
            const tile = this.tiles.data[i]
            const translation = tile.translation
            const scale = tile.scale
            for (let j = 0; j < this.imageryLayerNum; j++) {
                const imageInfo = tile.images[j]
                let index = i * this.imageryLayerNum + j;
                ctx.drawImage(imageInfo.img, 0, (index) * tileSize, tileSize, tileSize)
                imageInfo.imageAtlasIndex = index;
                //update translation and scale and clip
                imageInfo.translation = new Cartesian2(
                    translation.x,
                    translation.y / imageCount + (imageCount - 1 - index) / imageCount
                );
                imageInfo.scale = new Cartesian2(
                    scale.x,
                    scale.y / imageCount
                )
                imageInfo.clip = [...tile.clip]
                imageInfo.tile = tile
                imageInfoList.push(imageInfo)
            }
        }
        let res = {
            imageInfoList,
            imageryLayerNum: this.imageryLayerNum,
            tileNum: this.tiles.data.length,
            size: width * height * 4,//在cache中计算大小了
        }
        if (this.inWorker) {
            let imageData = canvas.getImageData(0, 0, width, height)
            res.imageData = imageData
        } else {
            res.canvas = canvas;
        }
        return res;
    }
    createTextureBlend() {
        let x_num = this.tiles.x_num;
        let y_num = this.tiles.y_num;
        let tileSize = this.tileSize;
        let left_top = new Cartesian2(0, 0)
        let right_bottom = new Cartesian2(1, 1)
        /**
         * 算法对应server/src/merge_tiles.rs中的clip函数
         * tiles是个瓦片网格，在数组中以列为主存储，假如tiles的大小是2x2，tiles列表中的瓦片坐标依次是
         * [(0,0),(0,1),(1,0),(1,1)],
         * [左上,左下,右上,右下]
         * 
         * 下面代码基于这样的假设：多边形的左上点在瓦片网格的左上瓦片，右下点在瓦片网格的右下瓦片
         * 
         * 思路：以多边形为裁剪框，裁剪出的部分瓦片网格就是我们需要的。
         * 
         * 多边形左上点uv坐标是(0,0)，换算为图片坐标上是(0,0)*scale + translation，再将0-1映射到0-256就是裁剪框左上点在瓦片网格中的左上点坐标。
         * 
         * 多边形的右下点uv坐标是(1,1)，换算为图片坐标上是(1,1)*scale + translation，得到右下点在右下瓦片上的图片坐标，将0-1映射到0-256再加上偏移值得到裁剪框右下点在瓦片网格中的右下点坐标。
         * 
         * 暂时没发现有bug
         */
        let first_tile = this.tiles.data[0];
        let last_tile = this.tiles.data[this.tiles.data.length - 1];
        left_top.scale_and_add(first_tile.scale, first_tile.translation)
        left_top.scale(tileSize)
        right_bottom.scale_and_add(last_tile.scale, last_tile.translation)
        let offset = new Cartesian2((x_num - 1) * tileSize, (y_num - 1) * tileSize)
        right_bottom.scale_and_add(
            new Cartesian2(tileSize, tileSize),
            offset,
        );
        //算法对应server/src/merge_tiles.rs中的merge_tile函数
        const canvas = new OffscreenCanvas(x_num * tileSize, y_num * tileSize)
        const ctx = canvas.getContext("2d")
        for (let i = 0; i < this.tiles.data.length; i++) {
            const tile = this.tiles.data[i]

            let y = i % y_num;
            let x = (i - y) / y_num;
            let offset_x = x * tileSize;
            let offset_y = y * tileSize;

            for (let j = 0; j < this.imageryLayerNum; j++) {
                const imageInfo = tile.images[j]
                let index = i * this.imageryLayerNum + j;
                ctx.drawImage(imageInfo.img, offset_x, offset_y, tileSize, tileSize)
            }
        }
        let imageData = ctx.getImageData(left_top.x, left_top.y, right_bottom.x - left_top.x, right_bottom.y - left_top.y);
        const size = imageData.width * imageData.height * 4
        if (this.inWorker) {
            return { imageData, size }
        } else {
            let clipedCanvas = document.createElement("canvas");
            clipedCanvas.width = imageData.width;
            clipedCanvas.height = imageData.height;
            let ctx1 = clipedCanvas.getContext("2d");
            ctx1.putImageData(imageData, 0, 0);
            return {
                canvas: clipedCanvas,
                size
            }
        }
    }
    requestImage(rectangle, level) {
        const promiseList = []
        const tiles = getCoordsOfLayer(rectangle, this.tilingScheme, level, undefined, this.imageryLayerNum, this.isBlend)
        this.tiles = tiles
        for (let i = 0; i < this.urls.length; i++) {
            const url = this.urls[i]
            for (const tile of tiles.data) {
                let requestUrl = makeRequestUrl(url, tile.x, tile.y, tile.level + this.levelOffset);
                promiseList.push(
                    imageCache.fetchImage(requestUrl)
                        .then(imageBitMap => {
                            tile.images[i] = { img: imageBitMap, zIndex: i, originZIndex: i }
                        })
                )
            }
        }
        return Promise.all(promiseList).then(() => {
            if (this.isBlend) {
                return this.createTextureBlend()
            } else {
                return this.createTexture()
            }
        })
    }
}
function makeRequestUrl(url, x, y, level) {
    const params = { x, y, z: level }
    let s = url;
    for (let key in params) {
        s = s.replace(new RegExp("\\{" + key + "\\}", "g"), params[key]);
    }
    return s;
}


export class ImageInfoList {
    constructor(imageryLayerNum) {
        this.data = []
        this.imageryLayerNum = imageryLayerNum;
    }
    get length() {
        return this.data.length
    }
    push(imageInfo) {
        this.data.push(imageInfo)
    }
    translationAndScaleList(Cartesian4) {
        if (Cartesian4) {
            return this.data.map(info => new Cartesian4(info.translation.x, info.translation.y, info.scale.x, info.scale.y))
        } else {
            return this.data.map(info => [info.translation.x, info.translation.y, info.scale.x, info.scale.y])
        }
    }
    clipList(Cartesian4) {
        if (Cartesian4) {
            return this.data.map(info => new Cartesian4(...info.clip))
        } else {
            return this.data.map(info => info.clip)
        }
    }
    /**
     * 举个例子:这是一个3x3瓦片集合,其中每个瓦片上有三张图片,也就是有三个图层,
     * 其中的每一块是一个组,比如(0,1,2),0是groupStartIndex,2的index是2,意思是相对于groupStartIndex的索引值是2
     * 0*    09*    18*-|--->groupStartIndex==18
     * 1*    10*    19*-|--->group
     * 2*    11*    20*-|--->groupIndex==19
     * 
     * 3*    12*    21*
     * 4*    13*    22*
     * 5*    14*    23*
     * 
     * 6*    15*    24*
     * 7*    16*    25*
     * 8*    17*    26*
     * 
     * @param {Array<{zIndex:number,originZIndex:number}>} newLayerList 
     */
    reorderLayer(newLayerList) {
        let zIndexMap = {}
        for (let layer of newLayerList) {
            zIndexMap[layer.originZIndex] = layer.zIndex;
        }
        let newList = new Array(this.data.length)
        for (let tileIndex = 0; tileIndex < this.data.length; tileIndex++) {
            let imageInfo = this.data[tileIndex]
            const zIndex = zIndexMap[imageInfo.originZIndex];
            imageInfo.zIndex = zIndex;

            let groupIndex = tileIndex % this.imageryLayerNum;
            let groupStartIndex = tileIndex - groupIndex;
            let newListIndex = groupStartIndex + zIndex;
            newList[newListIndex] = imageInfo;
        }
        this.data = newList;
    }
}