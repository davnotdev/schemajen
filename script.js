import init from "./pkg/schemajen.js";
import { supported_accumulators, generate_js } from "./pkg/schemajen.js";

const submit = document.getElementById("submit");
const inputBox = document.getElementById("in");
const outputBox = document.getElementById("out");
const typenameBox = document.getElementById("typename");
const selection = document.getElementById("accumulators");

let supported;

(async () => {
  await init();
  console.log(init, supported_accumulators(), generate_js);
  supported = JSON.parse(supported_accumulators());
  supported.forEach((accumulator) => {
    let option = document.createElement("option");
    option.value = accumulator;
    selection.appendChild(option);
  });
})();

submit.onclick = () => {
  let selectionValue = selection.value;
  let typename = typenameBox.value;
  if (supported.includes(selectionValue)) {
    let input = inputBox.value;
    outputBox.innerText = generate_js(selectionValue, typename, input);
  } else {
    outputBox.innerText = "Error: That accumulator does not exist.";
  }
};
