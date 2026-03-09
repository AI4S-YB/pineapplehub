// Trunk initializer: hook into WASM initialization to start the rayon thread pool.
// See: https://trunkrs.dev/assets/#initializer
export default function initRayon() {
    return {
        onStart() {},
        onProgress(_current, _total) {},
        onComplete() {},
        onSuccess(wasm) {
            // `wasm` is the `* as bindings` import — includes initThreadPool.
            // Limit to 2 workers: each image pipeline needs ~100-200MB working
            // memory (high-res decode + resize + processing buffers).
            // Too many concurrent workers exhaust WASM linear memory (OOM).
            const cores = Math.min(navigator.hardwareConcurrency || 4, 2);
            console.log(`[rayon] Initializing thread pool with ${cores} workers…`);
            return wasm.initThreadPool(cores).then(() => {
                console.log('[rayon] Thread pool ready ✓');
            });
        },
        onFailure(error) {
            console.error('[rayon] WASM init failed:', error);
        },
    };
}
