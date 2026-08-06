
import * as Cesium from 'cesium';
window.CESIUM_BASE_URL = './CesiumUnminified';
// import 'cesium/Build/Cesium/Widgets/widgets.css';
// window.CESIUM_BASE_URL = 'node_modules/cesium/Build/Cesium';
Cesium.Ion.defaultAccessToken =
    'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJqdGkiOiI4ODZmNWFmYi03MDZhLTRhNDMtOTg2Ny01N2YwNjE2NjMxNTEiLCJpZCI6Mjk5MjQsImlhdCI6MTY5ODgzMTY3NH0.EFStVMUXMy-HTbhQ87mgMyXt-5eYFWiVnq9xmZLmUOY';
export function initViewer(id, options) {
    const viewer = new Cesium.Viewer(id, {
        animation: false,
        baseLayerPicker: false,
        fullscreenButton: false,
        geocoder: false,
        homeButton: false,
        sceneModePicker: false,
        selectionIndicator: false,
        shadows: false,
        timeline: false,
        navigationHelpButton: false,
        infoBox: false,
        navigationInstructionsInitiallyVisible: false,
        shouldAnimate: false,
        contextOptions: {
            webgl: {
                alpha: true,
                depth: true,
                stencil: true,
                antialias: true,
                premultipliedAlpha: true,
                //通过canvas.toDataURL()实现截图需要将该项设置为true
                preserveDrawingBuffer: true,
                failIfMajorPerformanceCaveat: true,
            },
        },
        ...options
    });
    if (Cesium.FeatureDetection.supportsImageRenderingPixelated()) {//判断是否支持图像渲染像素化处理
        viewer.resolutionScale = window.devicePixelRatio;
    }
    //是否开启抗锯齿
    // viewer.scene.fxaa = true;
    // viewer.scene.postProcessStages.fxaa.enabled = true;
    // viewer.scene.debugShowFramesPerSecond = true;
    debugViewer(viewer);
    return viewer;
}

// import { pickPositionWorldCoordinates } from './geo/projection/my_geo_util'
export function debugViewer(viewer) {
    const handler = new Cesium.ScreenSpaceEventHandler(viewer.canvas);
    handler.setInputAction((movement) => {
        let cartesian3 = viewer.scene.camera.pickEllipsoid(movement.position);
        if (!cartesian3) return;
        let lonlat = cartesian3ToLonLatHeight(cartesian3);
        console.log(`cartesian3: ${cartesian3.x},${cartesian3.y},${cartesian3.z}`);
        console.log(
            `lonlat: ${lonlat.longitude},${lonlat.latitude},${lonlat.height}`,
        );
        cartesian3 = viewer.camera.position;
        lonlat = cartesian3ToLonLatHeight(cartesian3);
        console.log(
            `camera cartesian3: ${cartesian3.x},${cartesian3.y},${cartesian3.z}`,
        );
        console.log(
            `camera lonlat: ${lonlat.longitude},${lonlat.latitude},${lonlat.height}`,
        );
        console.log(
            `camera hpr: ${viewer.camera.heading},${viewer.camera.pitch},${viewer.camera.roll}`,
        );

        // {
        //   let t1 = viewer.scene.pickPositionWorldCoordinates(movement.position);
        //   t1 = cartesian3ToLonLatHeight(t1)
        //   console.log(t1)
        //   let t2 = pickPositionWorldCoordinates(viewer.scene, movement.position)
        //   t2 = cartesian3ToLonLatHeight(t2)
        //   console.log(t2)
        //   let t3 = viewer.scene.clampToHeight(cartesian3)
        //   t3 = cartesian3ToLonLatHeight(t3)
        //   console.log(t3)
        // }
    }, Cesium.ScreenSpaceEventType.LEFT_CLICK);
}

export function cartesian3ToLonLatHeight(cartesian3) {
    let position = Cesium.Cartographic.fromCartesian(cartesian3);
    position = {
        longitude: Cesium.Math.toDegrees(position.longitude),
        latitude: Cesium.Math.toDegrees(position.latitude),
        height: position.height,
    };
    return position;
}