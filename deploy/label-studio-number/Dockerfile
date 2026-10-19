# syntax=docker/dockerfile:1
FROM heartexlabs/label-studio@sha256:afcc516a22775a39d0d66f4c2ddc95d01be3e3d3862b21fcf4f9de34b4ad4e12 AS runtime-base

FROM node:22-alpine@sha256:0a7108bf6c7bf5de370ffb1a3ed6be93d405b43ff159f681a8d18c0e2bc2e402 AS frontend-builder
ENV BUILD_NO_SERVER=true \
    BUILD_NO_HASH=true \
    BUILD_NO_CHUNKS=true \
    BUILD_MODULE=true \
    YARN_CACHE_FOLDER=/root/web/.yarn \
    NX_CACHE_DIRECTORY=/root/web/.nx \
    NODE_ENV=production \
    NODE_OPTIONS="--max-old-space-size=4096"
WORKDIR /label-studio/web
RUN apk add --no-cache build-base pkgconfig cairo-dev giflib-dev libjpeg-turbo-dev libpng-dev pango-dev git python3
COPY web/package.json web/yarn.lock ./
COPY web/tools tools
RUN yarn install --prefer-offline --no-progress --pure-lockfile --frozen-lockfile --ignore-engines --non-interactive --production=false
COPY web/ .
COPY pyproject.toml ../pyproject.toml
COPY number-empty.patch /tmp/number-empty.patch
RUN cd /label-studio && git apply --check /tmp/number-empty.patch && git apply /tmp/number-empty.patch
RUN yarn run build
# Keep the release metadata from the unchanged official runtime.
COPY --from=runtime-base /label-studio/web/dist/apps/labelstudio/version.json dist/apps/labelstudio/version.json
COPY --from=runtime-base /label-studio/web/dist/libs/editor/version.json dist/libs/editor/version.json
COPY --from=runtime-base /label-studio/web/dist/libs/datamanager/version.json dist/libs/datamanager/version.json

FROM runtime-base
COPY --chown=1001:0 --from=frontend-builder /label-studio/web/dist /label-studio/web/dist
