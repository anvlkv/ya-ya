import "../wasm/popup/pkg/ya_ya_popup.js";
import "./styles.css";

console.debug("loaded popup scripts and styles");

export async function sendMessage(message) {
  return await browser.runtime.sendMessage(message);
}
