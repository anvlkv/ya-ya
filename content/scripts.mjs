import { main } from "../wasm/front/pkg/ya_ya_front.js";
import "./styles.css";

console.debug("loaded content scripts and styles");

main();

export async function sendMessage(message) {
  return await browser.runtime.sendMessage(message);
}
