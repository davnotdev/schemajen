import init from "./pkg/schemajen.js";
import { supported_accumulators, generate_js } from "./pkg/schemajen_bg.js";

(async () => {
    await init();
    console.log(init, supported_accumulators(), generate_js);
})();
