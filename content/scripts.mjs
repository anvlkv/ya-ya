import { main } from "../wasm/front/pkg/ya_ya_front.js";
import "./styles.css";

console.debug("loaded content scripts and styles");

main();

const rtm = typeof browser !== "undefined" ? browser : chrome;

export async function sendMessage(message) {
  return await rtm.runtime.sendMessage(message);
}

rtm.runtime.onMessage.addListener((req, sensder, sendResponse) => {
  if (req.action === "getSelectedText") {
    const selectedText = window.getSelection().toString();
    sendResponse({ selectedText });
  }
  return true;
});
