import { main } from "../wasm/front/pkg/ya_ya_front.js";
import "./styles.css";

console.debug("loaded content scripts and styles");

main();

const rtm = typeof browser !== "undefined" ? browser : chrome;

export async function sendMessage(message) {
  return await rtm.runtime.sendMessage(message);
}

rtm.runtime.onMessage.addListener((message, sensder, sendResponse) => {
  if (message.action === "getSelectedText") {
    const selectedText = window.getSelection().toString();
    const origin = window.location.origin;

    sendResponse({ selectedText, origin });
  }
});
