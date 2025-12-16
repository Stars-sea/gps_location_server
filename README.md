# Gps Location Server

用于 4G & Gps 模块的服务端

## How to deploy / 部署方法

**前置条件**:
1. 一台公网服务器
2. SSH
3. Git

### 使用 Docker 部署（推荐）

1. 首先 SSH 登录服务器。确保服务器安装了 Docker，安装方法不再赘述
2. 运行以下命令构建 Docker 镜像（感觉卡住了可以仔细看看 `Dockerfile` 的内容）：
   ```bash
   $ git clone git@github.com:Stars-sea/gps_location_server.git
   $ cd ./gps_location_server
   $ sudo docker build . -t gps_location_server --no-cache
   ```
3. 构建成功后，可以用以下命令启动容器：
   ```bash
   $ mkdir -p ~/docker/gps_location_server/output
   $ cp ./settings.json ~/docker/gps_location_server/settings.json

   $ sudo docker run -itd \
     --name gps_location_server_container \
     -v ~/docker/gps_location_server/settings.json:/app/settings.json \
     -v ~/docker/gps_location_server/output:/app/output/ \
     -e RUST_LOG=info \
     -p 1234:1234 \
     gps_location_server
   ```
4. 服务端在 `1234` 端口上部署完成，可以使用以下命令查看服务端日志：
   ```bash
   $ sudo docker logs -f gps_location_server_container
   ```

> #### ⚠️**注意**⚠️
> 
> `Dockerfile` 和 `settings.json` 关联性较强，  
> 因此在修改 `settings.json` 时请仔细核对 Docker 配置

### 直接编译部署

1. SSH 登录服务器（这里以 `Ubuntu 24.04.3 LTS` 为例）
2. 安装 Rust 环境
   ```bash
   $ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
3. 拉取源代码并编译
   ```bash
   $ git clone git@github.com:Stars-sea/gps_location_server.git
   $ cd ./gps_location_server
   $ cargo build --release

   $ cp ./target/release/gps_location_server ./
   ```
4. 运行
   ```bash
   $ RUST_LOG=info ./gps_location_server
   ```
5. 此时服务端在 `1234` 端口上部署完成

## LICENSE / 许可

本软件基于 [GNCL-1.0](https://github.com/giantpreston/giantpreston-non-commercial-license-v1) 开源

**The Giantpreston Non-Commercial License (GNCL-1.0)** allows free use, modification, and distribution of this software, while preventing commercial exploitation. The key terms are:  
**Giantpreston 非商业许可证（GNCL-1.0）** 允许免费使用、修改和分发此软件，但不允许商业利用。关键包括：

* **Non-commercial use only**: Use, copy, modify, and distribute the software for non-commercial purposes only.  
  **仅限非商业用途**：仅可出于非商业目的，使用、复制、修改及分发本软件
* **No commercial use**: Selling, reselling, or monetizing the software is prohibited unless you provide written permission.  
  **禁止商业用途**：禁止出售、转售本软件，或通过本软件获利，除非获得书面许可
* **Attribution**: Users must credit Stars_sea\<Stars_sea@outlook.com\> as the original creator in any derivative works.  
  **署名要求**：用户在任何衍生作品中，必须标明 Stars_sea\<Stars_sea@outlook.com\> 为原创作者
* **No patent/trademark claims**: No one can patent, trademark, or claim exclusive ownership of the software or its derivatives.  
  **禁止专利/商标主张**：任何人不得将本软件或其衍生作品申请专利、注册商标，或主张排他性所有权

### Modify the Software / 关于修改本软件

If you modify the software, you must include the following attribution in the modified version:  
如果你修改了此软件，必须在修改版本中注明以下信息：

```
This software is based on the original project by Stars_sea<Stars_sea@outlook.com>,  
released under the Giantpreston Non-Commercial License (GNCL-1.0).
```