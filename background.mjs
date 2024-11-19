console.log("hello from background script");

const rtm = typeof browser !== "undefined" ? browser : chrome;

rtm.runtime.onInstalled.addListener(() => {
  rtm.runtime.openOptionsPage(() => {
    console.log("onInstalled openOptionsPage");
  });
});
