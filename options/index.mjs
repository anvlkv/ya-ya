import "../wasm/options/pkg/ya_ya_options.js";
import "../style.css";

console.debug("loaded popup scripts and styles");

const rtm = typeof browser !== "undefined" ? browser : chrome;

export async function sendMessage(message) {
  return await rtm.runtime.sendMessage(message);
}
