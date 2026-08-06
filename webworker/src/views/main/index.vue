<script setup>
import * as Cesium from 'cesium';
import { GUI } from 'dat.gui';
import * as Utils from '../utils.js'
import axios from 'axios'
import RectangleTileTexture from '../../lib/RectangleTileTexture.js'
import { VueDraggable } from 'vue-draggable-plus'
import PromiseWorker from 'promise-worker';
import GetMapWorker from "../../lib/getMap.worker.js?worker"
import Resolution from '../../lib/Resolution'
function getLevelWithMaximumTexelSpacing(
    layer,
    texelSpacing,
    latitudeClosestToEquator
) {
    // PERFORMANCE_IDEA: factor out the stuff that doesn't change.
    const imageryProvider = layer._imageryProvider;
    const tilingScheme = imageryProvider.tilingScheme;
    const ellipsoid = tilingScheme.ellipsoid;
    const latitudeFactor = !(
        layer._imageryProvider.tilingScheme.projection instanceof
        Cesium.GeographicProjection
    )
        ? Math.cos(latitudeClosestToEquator)
        : 1.0;
    const tilingSchemeRectangle = tilingScheme.rectangle;
    const levelZeroMaximumTexelSpacing =
        (ellipsoid.maximumRadius * tilingSchemeRectangle.width * latitudeFactor) /
        (imageryProvider.tileWidth * tilingScheme.getNumberOfXTilesAtLevel(0));

    const twoToTheLevelPower = levelZeroMaximumTexelSpacing / texelSpacing;
    const level = Math.log(twoToTheLevelPower) / Math.log(2);
    const rounded = Math.round(level);
    return rounded | 0;
}
const list = ref([
    {
        name: 'osm_landuse_zhucheng',
        zIndex: 0,
        originZIndex: 0,
    },
    {
        name: 'hl_china',
        zIndex: 1,
        originZIndex: 1,
    },
    {
        name: 'osm_road_zhucheng',
        zIndex: 2,
        originZIndex: 2,
    },
])
let serverKey = "dce1ed03fdd1585943f5b69054868306"
let browserKey = "e7b39664564a7cf1cdd419f2bfa70073"
let serverType = "后端服务"
let rustUrlsTianDiTu = [
    "https://t2.tianditu.gov.cn/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk=75f0434f240669f4a2df6359275146d2",
    "https://t2.tianditu.gov.cn/cia_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=img&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={x}&TILEROW={y}&TILEMATRIX={z}&tk=75f0434f240669f4a2df6359275146d2",
]
let tianditu_vec_w = tk => [`https://t0.tianditu.gov.cn/vec_w/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=vec&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILEMATRIX={z}&TILEROW={y}&TILECOL={x}&tk=${tk}`]
let rustUrls = [
    "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_landuse_zhucheng/getMap?styleId=osm_landuse_zhucheng&x={x}&y={y}&l={z}",
    "http://intenal.geoway-atlas.com:31280/mapserver/vmap/hl_china/getMap?styleId=new&x={x}&y={y}&l={z}",
    "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_road_zhucheng/getMap?styleId=osm_road_zhucheng&x={x}&y={y}&l={z}",
]
let nodeUrls = [
    "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_landuse_zhucheng/export?styleId=osm_landuse_zhucheng&tilesize=512&withlabel=0&ratio=1",
    "http://intenal.geoway-atlas.com:31280/mapserver/vmap/hl_china/export?styleId=new&tilesize=512&withlabel=0&ratio=1",
    "http://intenal.geoway-atlas.com:31280/mapserver/vmap/osm_road_zhucheng/export?styleId=osm_road_zhucheng&tilesize=512&withlabel=0&ratio=1",
]
function getIndex(url, indexKey) {
    for (let item of list.value) {
        if (url.includes(item.name)) {
            return item[indexKey];
        }
    }
    throw "错误"
}
function update(urls, indexKey) {
    let newList = new Array(urls.length)
    for (let i = 0; i < urls.length; i++) {
        let url = urls[i]
        let originZIndex = getIndex(url, indexKey)
        newList[originZIndex] = url
    }
    return newList;
}
const worker = new PromiseWorker(new GetMapWorker());

let curImageInfoList
let rectanglePrimitive
let viewer
let tileset
let server = {
    "tianditu": 'http://localhost:8080/getMap',
    "rust": 'http://localhost:8080/getMap',
    "webworker": 'http://localhost:8080/getMap',
    "node": "http://localhost:7777/getMap",
}

onMounted(async () => {
    function getTransferUrl() {
        return server[params.后端服务类型]
    }
    function getUrls() {
        if (params.后端服务类型 == "node") {
            return nodeUrls
        } else if (params.后端服务类型 == "tianditu") {
            return tianditu_vec_w(serverType == "后端服务" ? serverKey : browserKey)
        } else {
            return rustUrls
        }
    }
    function getEpsg() {
        if (params.web墨卡托) {
            return 3857
        } else {
            return 4326
        }
    }
    function getLevelOffset() {
        if (params.后端服务类型 == "tianditu") {
            return 0;
        } else {
            return 1;
        }
    }
    class ImaeryProvider extends Cesium.ModelImageryProvider {
        constructor(options) {
            super(options)
        }
        makeRequestUrl(rectangle) {
            this.urls = getUrls()
            this.transferUrl = getTransferUrl()
            this.epsg = getEpsg()
            this.levelOffset = getLevelOffset()
            if (params.后端服务类型 == "node") {
                rectangle.west = Cesium.Math.toDegrees(rectangle.west)
                rectangle.south = Cesium.Math.toDegrees(rectangle.south)
                rectangle.east = Cesium.Math.toDegrees(rectangle.east)
                rectangle.north = Cesium.Math.toDegrees(rectangle.north)
            }
            return super.makeRequestUrl(rectangle)
        }
        requestImage(rectangle, request) {
            const url = this.makeRequestUrl(rectangle)
            console.log(this.count)
            if (params.后端服务类型 === "webworker") {
                return worker.postMessage(url).then(res => {
                    return res.imageData;
                })
            } else {
                return this.loadImage(url, request)
            }
        }
    }

    viewer = Utils.initViewer("cesium-map", { sceneModePicker: true });
    viewer.scene.debugShowFramesPerSecond = true;
    viewer.camera.setView({
        destination: new Cesium.Cartesian3(-2538371.7342178836, 4504102.511974393, 3734098.796782271),
        orientation: new Cesium.HeadingPitchRoll(6.282054590469908, -1.5639638694841835, 0)
    })
    const imageryLayer = viewer.scene.imageryLayers.addImageryProvider(new Cesium.TileCoordinatesImageryProvider({
        tilingScheme: new Cesium.WebMercatorTilingScheme(),
        tileHeight: 256,
        tileWidth: 256
    }))

    const boundingRectangle = {
        "north": 36.02706241294971,
        "south": 36.014049750548764,
        "east": 119.41071546303122,
        "west": 119.38205085662787
    }
    const tileRectangle = Cesium.Rectangle.fromDegrees(
        boundingRectangle.west,
        boundingRectangle.south,
        boundingRectangle.east,
        boundingRectangle.north
    )

    rectanglePrimitive = viewer.scene.primitives.add(new Cesium.Primitive({
        geometryInstances: new Cesium.GeometryInstance({
            geometry: new Cesium.RectangleGeometry({
                rectangle: tileRectangle,
                vertexFormat: Cesium.EllipsoidSurfaceAppearance.VERTEX_FORMAT,
            }),
            attributes: {
                color: Cesium.ColorGeometryInstanceAttribute.fromColor(
                    Cesium.Color.RED
                ),
            },
        }),
        appearance: new Cesium.EllipsoidSurfaceAppearance({
            flat: true,
            material: Cesium.Material.fromType("Image", {
                image: "Cesium_Logo_Color.jpg",
            }),
        }),
        materialSupport: Cesium.MaterialAppearance.MaterialSupport.TEXTURED,
    }))
    viewer.entities.add({
        rectangle: {
            coordinates: tileRectangle,
            fill: false,
            outline: true,
            outlineColor: Cesium.Color.RED,
            outlineWidth: 2,
        }
    })
    // viewer.extend(Cesium.viewerCesium3DTilesInspectorMixin);
    const gui = new GUI()
    let oldCanvas
    let testCanvas = document.getElementById("test-canvas")
    let ctx = testCanvas.getContext("2d")
    const params = {
        async "着色器内混合多图层"() {
            //保证urls的顺序没变
            rustUrls = update(rustUrls, "originZIndex")
            const task = new RectangleTileTexture(rustUrls, undefined, 4326, false)
            const { canvas, imageInfoList, imageryLayerNum, tileNum } = await task.requestImage(tileRectangle, 13)
            curImageInfoList = imageInfoList
            testCanvas.style.height = "100%"
            testCanvas.width = canvas.width
            testCanvas.height = canvas.height
            ctx.drawImage(canvas, 0, 0)
            const count = imageInfoList.length
            const newMaterial = new Cesium.Material({
                fabric: {
                    type: "my-image",
                    uniforms: {
                        image: testCanvas,
                    },
                    source:
                        `
                    uniform vec4 u_clipList[${count}];
                    uniform vec4 u_translationAndScaleList[${count}];
                    uniform int u_tileNum;
                    uniform int u_imageryLayerNum;
                    czm_material czm_getMaterial(czm_materialInput materialInput)
                    {
                        czm_material material = czm_getDefaultMaterial(materialInput);
                        vec2 st = materialInput.st;
                        int clipOffset = 0;
                        vec4 final_color = vec4(0.0);
                        for(int tileIndex=0; tileIndex<u_tileNum; tileIndex++){
                            for(int layerIndex=0; layerIndex < u_imageryLayerNum; layerIndex++){
                                int index = tileIndex * u_imageryLayerNum + layerIndex;
                                vec4 clip = u_clipList[index];
                                if(st.s>=clip.x && st.t>=clip.y && st.s<clip.z && st.t<clip.w){
                                    vec4 translationAndScale = u_translationAndScaleList[index];
                                    vec2 new_st = st * translationAndScale.zw +  translationAndScale.xy;
                                    vec4 source_color = texture(image, new_st);
                                    final_color = mix(final_color,source_color,source_color.a);
                                }
                            }
                        }
                        material.diffuse = final_color.rgb;
                        material.alpha = final_color.a;
                        return material;
                    }
                    `
                },
            })
            rectanglePrimitive.appearance.uniforms = {
                u_clipList: imageInfoList.clipList(Cesium.Cartesian4),
                u_translationAndScaleList: imageInfoList.translationAndScaleList(Cesium.Cartesian4),
                u_tileNum: tileNum,
                u_imageryLayerNum: imageryLayerNum,
            }
            console.log(rectanglePrimitive.appearance.uniforms)
            rectanglePrimitive.appearance.material = newMaterial
        },
        async "js主线程混合多图层"() {
            const task = new RectangleTileTexture(rustUrls, undefined, 4326, true)
            const { canvas } = await task.requestImage(tileRectangle, 13)
            testCanvas.style.height = "unset"
            testCanvas.width = canvas.width
            testCanvas.height = canvas.height
            ctx.drawImage(canvas, 0, 0)
            rectanglePrimitive.appearance.material = Cesium.Material.fromType("Image", {
                image: testCanvas,
            })
        },
        后端服务类型: "rust",
        "后端服务"() {
            serverType = "后端服务"
            let url = imageProvider.makeRequestUrl(tileRectangle);
            axios.get(url, { responseType: "blob" }).then(res => {
                let img = new Image()
                img.src = URL.createObjectURL(res.data);
                img.onload = () => {
                    testCanvas.style.height = "unset"
                    testCanvas.width = img.width
                    testCanvas.height = img.height
                    ctx.drawImage(img, 0, 0)
                    rectanglePrimitive.appearance.material = Cesium.Material.fromType("Image", {
                        image: testCanvas,
                    })
                }
            })
        },
        "worker服务"() {
            serverType = "worker服务"
            let url = imageProvider.makeRequestUrl(tileRectangle);
            worker.postMessage(url).then(res => {
                console.log("response is ", res)
                const imageData = res.imageData;
                testCanvas.style.height = "unset"
                testCanvas.width = imageData.width
                testCanvas.height = imageData.height
                ctx.putImageData(imageData, 0, 0)
                rectanglePrimitive.appearance.material = Cesium.Material.fromType("Image", {
                    image: testCanvas,
                })
            })
        },
        async "添加倾斜摄影模型"() {
            if (tileset) {
                params.移除倾斜摄影模型(tileset)
                tileset = null
            }
            tileset = viewer.scene.primitives.add(
                await Cesium.Cesium3DTileset.fromUrl(
                    // "http://intenal.geoway-atlas.com:31280/ime-cloud/rest/hongkong_2_0327/3dtiles/tileset.json",
                    // "http://intenal.geoway-atlas.com:31280/ime-cloud/rest/qx_huashan_20240109/3dtiles/tileset.json",
                    "http://intenal.geoway-atlas.com:31280/ime-cloud/rest/qx_zhucheng_20240301/3dtiles/tileset.json",
                    {
                        skipLevelOfDetail: true
                    }
                )
            );
            viewer.scene.primitives.add(tileset);
            viewer.zoomTo(tileset);
        },
        "移除倾斜摄影模型"() {
            if (tileset) {
                viewer.scene.primitives.remove(tileset);
                tileset = null
            }
        },
        "是否纹理替换": false,
        "web墨卡托": false
    }
    const imageProvider = new ImaeryProvider({
        transferUrl: getTransferUrl(),
        urls: getUrls()
    })
    const rectangleFolder = gui.addFolder("测试多边形")
    rectangleFolder.add(params, "着色器内混合多图层")
    rectangleFolder.add(params, "js主线程混合多图层")
    rectangleFolder.add(params, "后端服务")
    rectangleFolder.add(params, "worker服务")
    rectangleFolder.open()
    gui.add(params, "后端服务类型", ["rust", "webworker", "node", "tianditu"])
    gui.add(params, "添加倾斜摄影模型")
    gui.add(params, "移除倾斜摄影模型")
    gui.add(params, "是否纹理替换").onChange(v => {
        if (v) {
            if (tileset) {
                tileset.imageProvider = imageProvider
            }
        } else {
            tileset.imageProvider = null;
        }
    })
    gui.add(params, "web墨卡托")
})
const onUpdate = (evt) => {
    console.log(list.value)
    list.value.forEach((item, index) => {
        item.zIndex = index;
    })
    if (curImageInfoList) {
        curImageInfoList.reorderLayer(list.value)
        let newList = curImageInfoList.translationAndScaleList().map(x => new Cesium.Cartesian4(...x));
        rectanglePrimitive.appearance.uniforms.u_translationAndScaleList = newList
    }
    if (tileset) {
        tileset.imageProvider.reset()
    }
    rustUrls = update(rustUrls, "zIndex")
    nodeUrls = update(nodeUrls, "zIndex")
}
</script>

<template>
    <div id="cesium-map" style="position: relative">
        <canvas id="test-canvas" width="256" height="256" />
    </div>
    <VueDraggable ref="el" v-model="list" class="drag-sort-layer" :animation="150" ghostClass="ghost"
        @update="onUpdate">
        <div v-for="(item, index) in list" :key="item.name" class="item">
            {{ item.zIndex }}:{{ item.name }}
        </div>
    </VueDraggable>
</template>

<style lang="scss">
#test-canvas {
    z-index: 1;
    border: 1px solid black;
    position: absolute;
    top: 0px;
    overflow: auto;
    background: white;
}

.cesium-performanceDisplay-defaultContainer {
    top: 500px;
}

.cesium-viewer-cesium3DTilesInspectorContainer {
    right: 18vw;
    top: 1vh;
}
</style>
