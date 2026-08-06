import { expect, describe, it, beforeEach } from 'vitest'
import { calculateTranslationAndScale, getCoordsOfLayer } from '../tile'
import Rectangle from '../Rectangle.js'
import GeographicTilingScheme from '../GeographicTilingScheme.js'
import WebMercatorTilingScheme from '../WebMercatorTilingScheme.js'
// import * as Cesium from 'cesium'
describe("tiles", () => {
    beforeEach(async (context) => {
        const boundingRectangle = {
            "north": 36.02706241294971,
            "south": 36.014049750548764,
            "east": 119.41071546303122,
            "west": 119.38205085662787
        }
        const tileRectangle = Rectangle.fromDegrees(
            boundingRectangle.west,
            boundingRectangle.south,
            boundingRectangle.east,
            boundingRectangle.north
        )
        context.tileRectangle = tileRectangle
    })
    it('getCoordsOfLayer/Tiles', ({ tileRectangle }) => {
        const tilingScheme = new GeographicTilingScheme()
        let imageryLevel = 13
        const imageryLayerRectangle = undefined
        const imageryLayerNum = 3
        imageryLevel = 15
        let tiles = getCoordsOfLayer(tileRectangle, tilingScheme, imageryLevel, imageryLayerRectangle, imageryLayerNum)
        expect(tiles.x_num).toBe(7)
        expect(tiles.y_num).toBe(3)
        expect(tiles.layer_num).toBe(3)
        expect(tiles.level).toBe(imageryLevel)
        const tile = tiles.data[0]
        expect(tile.images.length).toBe(3)
        expect(tile.level).toBe(imageryLevel)

        expect(tiles.minx + tiles.x_num - 1).toBe(tiles.maxx)
        expect(tiles.miny + tiles.y_num - 1).toBe(tiles.maxy)

        //主列
        for (let x = 0; x < tiles.x_num; x++) {
            for (let y = 0; y < tiles.y_num; y++) {
                let tile = tiles.data[x * tiles.y_num + y]
                expect(tile.x).toBe(tiles.minx + x)
                expect(tile.y).toBe(tiles.miny + y)
            }
        }
    })
    it('getCoordsOfLayer/GeographicTilingScheme', ({ tileRectangle }) => {
        const tilingScheme = new GeographicTilingScheme()
        let imageryLevel = 13
        const imageryLayerRectangle = undefined
        const imageryLayerNum = 3
        let tiles = getCoordsOfLayer(tileRectangle, tilingScheme, imageryLevel, imageryLayerRectangle, imageryLayerNum)
        expect(tiles.minx).toBe(13625)
        expect(tiles.miny).toBe(2456)
        expect(tiles.maxx).toBe(13626)
        expect(tiles.maxy).toBe(2456)
        expect(tiles.data.length).toBe(2)

        imageryLevel = 14
        tiles = getCoordsOfLayer(tileRectangle, tilingScheme, imageryLevel, imageryLayerRectangle, imageryLayerNum)
        expect(tiles.minx).toBe(27250)
        expect(tiles.miny).toBe(4912)
        expect(tiles.maxx).toBe(27253)
        expect(tiles.maxy).toBe(4913)
        expect(tiles.data.length).toBe(8)

        imageryLevel = 15
        tiles = getCoordsOfLayer(tileRectangle, tilingScheme, imageryLevel, imageryLayerRectangle, imageryLayerNum)
        expect(tiles.minx).toBe(54500)
        expect(tiles.miny).toBe(9825)
        expect(tiles.maxx).toBe(54506)
        expect(tiles.maxy).toBe(9827)
        expect(tiles.data.length).toBe(7 * 3)
    })

    it('getCoordsOfLayer/WebMercatorTilingScheme', ({ tileRectangle }) => {
        const tilingScheme = new WebMercatorTilingScheme()
        let imageryLevel = 13
        const imageryLayerRectangle = undefined
        const imageryLayerNum = 3
        let tiles = getCoordsOfLayer(tileRectangle, tilingScheme, imageryLevel, imageryLayerRectangle, imageryLayerNum)
        expect(tiles.minx).toBe(6812)
        expect(tiles.miny).toBe(3216)
        expect(tiles.maxx).toBe(6813)
        expect(tiles.maxy).toBe(3216)
        expect(tiles.data.length).toBe(2)

        imageryLevel = 14
        tiles = getCoordsOfLayer(tileRectangle, tilingScheme, imageryLevel, imageryLayerRectangle, imageryLayerNum)
        expect(tiles.minx).toBe(13625)
        expect(tiles.miny).toBe(6432)
        expect(tiles.maxx).toBe(13626)
        expect(tiles.maxy).toBe(6432)
        expect(tiles.data.length).toBe(2)

        imageryLevel = 15
        tiles = getCoordsOfLayer(tileRectangle, tilingScheme, imageryLevel, imageryLayerRectangle, imageryLayerNum)
        expect(tiles.minx).toBe(27250)
        expect(tiles.miny).toBe(12864)
        expect(tiles.maxx).toBe(27253)
        expect(tiles.maxy).toBe(12865)
        expect(tiles.data.length).toBe(4 * 2)

        imageryLevel = 16
        tiles = getCoordsOfLayer(tileRectangle, tilingScheme, imageryLevel, imageryLayerRectangle, imageryLayerNum)
        expect(tiles.minx).toBe(54500)
        expect(tiles.miny).toBe(25728)
        expect(tiles.maxx).toBe(54506)
        expect(tiles.maxy).toBe(25731)
        expect(tiles.data.length).toBe(7 * 4)

        imageryLevel = 17
        tiles = getCoordsOfLayer(tileRectangle, tilingScheme, imageryLevel, imageryLayerRectangle, imageryLayerNum)
        expect(tiles.minx).toBe(109001)
        expect(tiles.miny).toBe(51457)
        expect(tiles.maxx).toBe(109012)
        expect(tiles.maxy).toBe(51463)
        expect(tiles.data.length).toBe(12 * 7)
    })
})
