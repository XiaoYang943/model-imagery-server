# 倾斜摄影模型叠加影像图层
## 介绍
本服务有两个版本，一个是webworker前端，一个是rust后端，两个的算法一样

倾斜摄影模型叠加影像的服务，影像用于叠加、裁剪、偏移。

Cesium源码中加载该服务的功能参考[ModelImageryProvider](http://172.16.67.163:8083/globe-sdk/cesium_110/-/blob/cesium-merge-115/packages/engine/Source/Scene/Model/ModelImageryProvider.js)

Cesium Sandcastle示例参考[3dtiles矢量叠加.html](http://172.16.67.163:8083/globe-sdk/cesium_110/-/blob/cesium-merge-115/Apps/Sandcastle/gallery/geoway-demo/3dtiles%E7%9F%A2%E9%87%8F%E5%8F%A0%E5%8A%A0.html)

sdk示例参考[geoway-3dtile-image-provider](http://latest.geoway-atlas.com:31980/web-globe-sdk-v115/?menu=true&url=./core/examples/scene/geoway-3dtile-image-provider/index.example.ts)

原始源码有本地git，文件夹在`qiuzhenyu/wsl/Ubuntu-20.04-new/home/catnuko/image-layer/server`
## 背景
原来倾斜摄影模型瓦片的bbox，请求一个中转，中转请求wms服务动态切片

改为请求已切片的wmts，再叠加到倾斜瓦片上

能叠加多层，能调整顺序

走webworker，或者走一个nodejs的中转服务，中转服务去请求图片瓦片，做切割。

https://github.com/dsanders11/imagebitmap-getimagedata-demo/blob/main/worker.js

## 功能
功能:给一个多边形,计算多边形覆盖的瓦片集合,按xyz请求瓦片集合并生成瓦片集合纹理,按多边形裁剪瓦片集合纹理,返回结果,结果是瓦片集合纹理或者裁剪后的瓦片集合纹理.

实现:
1. js主线程生成瓦片集合纹理,配合片元着色器代码使用
2. js主线程生成裁剪后的瓦片集合纹理
3. js webworker线程生成瓦片集合纹理,配合片元着色器代码使用
4. js webworker线程生成裁剪后的瓦片集合纹理
5. rust cpu生成裁剪后的瓦片集合纹理
6. rust gpu生成裁剪后的瓦片集合纹理


注意:
1. rust端不支持设置epgs,目前只支持4326切片规则的瓦片集


## 打包worker
`yarn build`打包后生成`dist\modelImageryServer/getMap.worker-BK5moCdY.js`，将文件拷贝到`geoway-globe/sdk-updategrade-new`分支的`src/assets/worker`中，在示例[geoway-3dtile-image-provider](http://latest.geoway-atlas.com:31980/web-globe-sdk-v115/?menu=true&url=./core/examples/scene/geoway-3dtile-image-provider/index.example.ts)中查看应用结果


