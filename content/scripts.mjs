import "../wasm/front/pkg/ya_ya_front.js";
import "./styles.css";

console.debug("loaded content scripts and styles");

export async function sendMessage(message) {
  return await browser.runtime.sendMessage(message);
}
