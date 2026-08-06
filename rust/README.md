# model-imagery-server
## 介绍
本服务有两个版本，一个是webworker前端，一个是rust后端，两个的算法一样

倾斜摄影模型叠加影像的服务，影像用于叠加、裁剪、偏移。

Cesium源码中加载该服务的功能参考[ModelImageryProvider](http://172.16.67.163:8083/globe-sdk/cesium_110/-/blob/cesium-merge-115/packages/engine/Source/Scene/Model/ModelImageryProvider.js)

Cesium Sandcastle示例参考[3dtiles矢量叠加.html](http://172.16.67.163:8083/globe-sdk/cesium_110/-/blob/cesium-merge-115/Apps/Sandcastle/gallery/geoway-demo/3dtiles%E7%9F%A2%E9%87%8F%E5%8F%A0%E5%8A%A0.html)

sdk示例参考[geoway-3dtile-image-provider](http://latest.geoway-atlas.com:31980/web-globe-sdk-v115/?menu=true&url=./core/examples/scene/geoway-3dtile-image-provider/index.example.ts)

原始源码有本地git，文件夹在`qiuzhenyu/wsl/Ubuntu-20.04-new/home/catnuko/image-layer/server`

## 编译
### 从linux编译到windows
[将Rust应用程序从Linux交叉编译到Windows](https://cloud.tencent.com/developer/ask/sof/28504)

编译后把`target/x86_64-pc-windows-gnu/release/modelImageryServer.exe`拷贝到windows中的`modelImageryServer`文件夹中，同时拷贝`rust/config.yml`文件到`modelImageryServer`文件夹，运行exe文件。
```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

### 交叉编译到 Linux ARM64 (aarch64)
在 x86_64 Linux（如 Ubuntu 24.04 WSL）上交叉编译出 ARM64 程序，产物为 `target/aarch64-unknown-linux-gnu/release/modelImageryServer`。

**1. 添加 Rust 目标**
```bash
rustup target add aarch64-unknown-linux-gnu
```
（若配置了国内 rustup 镜像，会自动走镜像下载。）

**2. 安装交叉编译工具链与 ARM64 系统库**
本项目依赖 `wgpu`（Vulkan/EGL）与 `openssl`，交叉编译需要 aarch64 版的 C 链接库。Ubuntu 的 ARM 架构软件源在 `ports.ubuntu.com`，需先添加 aarch64 源（标准 `archive.ubuntu.com` 不提供 arm64）：

在 `/etc/apt/sources.list.d/arm64-ports.sources` 写入（需 root）：
```text
Types: deb
URIs: http://ports.ubuntu.com/ubuntu-ports
Suites: noble noble-updates noble-backports noble-security
Components: main universe restricted multiverse
Architectures: arm64
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
```
（同时建议在原 `ubuntu.sources` 的每个 stanza 加上 `Architectures: amd64`，避免 update 时去 archive 上拉取不存在的 arm64 索引而报错。）

```bash
sudo dpkg --add-architecture arm64
sudo apt-get update
sudo apt-get install -y \
  gcc-aarch64-linux-gnu pkg-config \
  libssl-dev:arm64 libvulkan-dev:arm64 libxkbcommon-dev:arm64 \
  libwayland-dev:arm64 libx11-dev:arm64 libxrandr-dev:arm64 \
  libx11-xcb-dev:arm64 libxcb1-dev:arm64 libegl-dev:arm64 \
  libgles-dev:arm64 libexpat1-dev:arm64
```
> 注意：Ubuntu 24.04 中 EGL/GLES 的开发包名为 `libegl-dev` / `libgles-dev`（旧名 `libegl1-dev` / `libgles2-mesa-dev` 已不可用）。aarch64 库会安装到 `/usr/lib/aarch64-linux-gnu/`，交叉编译器 `aarch64-linux-gnu-gcc` 默认可检索到该路径。

**3. 设置交叉编译环境变量并编译**
```bash
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_LIBDIR=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/share/aarch64-linux-gnu/pkgconfig

cargo build --release --target aarch64-unknown-linux-gnu
```
（如果使用了国内 crates 镜像，可在 `~/.cargo/config.toml` 中把 `source.crates-io` 替换为 `rsproxy-sparse` 等，加速依赖下载。）

**4. 产物**
```
target/aarch64-unknown-linux-gnu/release/modelImageryServer
```
`file` 显示为 `ELF 64-bit LSB pie executable, ARM aarch64`。

**5. 目标机运行依赖**
二进制本身只硬链接 `libssl.so.3` / `libcrypto.so.3` 与标准库；`wgpu` 的 Vulkan/EGL 在运行时按需动态加载。因此在 ARM64 目标机上需要安装 Vulkan 运行库（无独显时可用 Mesa 软件实现 `libvulkan1` + `mesa-vulkan-drivers`，即 lavapipe）：
```bash
sudo apt-get install -y libvulkan1 mesa-vulkan-drivers
```
然后直接运行 `./modelImageryServer`（同目录需放 `config.yml`）。

### Q&A
error occurred: Failed to find tool. Is `x86_64-w64-mingw32-gcc` installed?

```bash
sudo apt install mingw-w64
```
