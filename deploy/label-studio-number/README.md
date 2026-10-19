# Label Studio 1.23.2：原生 Number 缺测修复

这是 [产品跟踪 #22](https://github.com/AI4S-YB/pineapplehub/issues/22) 的临时源码构建配方，对应 [上游 PR #9989](https://github.com/HumanSignal/label-studio/pull/9989)。Number 是 Label Studio 的原生数值输入控件；未修复时，清空输入会因 `+""` 生成零，无法可靠撤回测量。补丁只将空字符串交给模型既有的 `null` 表达，非空输入沿用原处理，通用控件中真正输入的零仍保留。本项目的正数限制由 [XML](../../docs/annotation/label-studio.xml) 表达。

## 固定输入

- 发布源码：`HumanSignal/label-studio` 的 **1.23.2**，提交 `df96d79183156e556d91825d15fa7721f1145e1a`。不使用 `develop` 构建运行镜像。
- 修复提交：`8d829fd884e69b70b16e5ec3790514e3010a4766`，仅将其中的一行源码修改回植至 1.23.2；[number-empty.patch](number-empty.patch) 与独立审阅的源修改一致。
- 后端基础：`heartexlabs/label-studio@sha256:afcc516a22775a39d0d66f4c2ddc95d01be3e3d3862b21fcf4f9de34b4ad4e12`。
- 构建器：`node:22-alpine@sha256:0a7108bf6c7bf5de370ffb1a3ed6be93d405b43ff159f681a8d18c0e2bc2e402`。

[官方发布版 Dockerfile](https://github.com/HumanSignal/label-studio/blob/df96d79183156e556d91825d15fa7721f1145e1a/Dockerfile) 和 `web/package.json` 是构建依据。本配方保留官方 Yarn 锁文件、安装参数、构建环境和 `yarn run build`（内部使用 NX）步骤；只复制编译后的 `web/dist`，三个前端 `version.json` 从固定运行镜像保留。Python 后端、数据库格式、入口与现有版本元数据均沿用运行镜像。不要用修改压缩 JS 的原型代替源码构建。

## 在主机准备源码上下文并构建

使用已有的官方源码检出和此产品工作树，替换以下两个路径。构建上下文取自固定提交的 `git archive`，不包含源码工作树的未提交文件；随后加入本配方和补丁。临时目录保留至验收结束。

```bash
ls_source=/path/to/label-studio
product_source=/path/to/pineapplehub
build_context=$(mktemp -d /tmp/label-studio-number-1.23.2.XXXXXX)
source_sha=df96d79183156e556d91825d15fa7721f1145e1a
git -C "$ls_source" archive --format=tar --output="$build_context/source.tar" "$source_sha" web pyproject.toml
tar -xf "$build_context/source.tar" -C "$build_context"
rm "$build_context/source.tar"
cp "$product_source/deploy/label-studio-number/Dockerfile" "$build_context/Dockerfile"
cp "$product_source/deploy/label-studio-number/number-empty.patch" "$build_context/number-empty.patch"
image_tag=pineapplehub-label-studio:1.23.2-number-empty-8d829fd-20261019
docker build --tag "$image_tag" "$build_context"
docker image inspect --format '{{.Id}}' "$image_tag"
```

该标签专用于此补丁构建，不覆盖官方标签或既有本地镜像。重复构建若产物改变，使用新的唯一标签并记录原因。2026-10-19 在 Linux/amd64 构建的本地镜像 ID 为 `sha256:bcdc34e0e7c64f511a6363d333eb0b35f348f4fa7f38efd813816652dea4100a`，对应上面的标签。协调者的私有执行记录需保存源码 SHA、修复 SHA、两个摘要、实际标签和最终 ID，并核对前端版本 JSON、后端版本及启动行为。

## 接入前验收与替换条件

协调者在隔离的真实 1.23.2 服务运行正式 XML：验证六字段初始缺测、填写后清空、区域切换、保存、重开、导出；清空只撤回对应数值结果，不留下伪零，不删除轮廓或其他测量；选中完整果眼时才显示对应字段。另验普通／重叠起笔、修补及最后画青色的行为。上游单元、Cypress 和格式检查由上游 CI 承担，不将源码回植检查冒充运行验收。

部署由协调者在现有主机配置中完成：应用和网页服务容器使用同一固定补丁镜像，激活前保留数据库、旧镜像与系统代次，并验证原项目标注及草稿保真。本目录不发布注册服务、不升级后端，也不执行生产数据转换。

当包含此修复的官方发布版通过同样的缺测、区域和数据往返验收后，再安排切换官方镜像并移除此临时配方；上游合并或发布本身不触发自动升级。
