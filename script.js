import { init, supported_accumulators, generate_js } from "./pkg/schemajen.js";

(async () => {
    await init();
    console.log(init, supported_accumulators(), generate_js);
})();
