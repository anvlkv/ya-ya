const rtm = typeof browser !== "undefined" ? browser : chrome;

export async function sendMessage(message) {
  const [activeTab] = await rtm.tabs.query({
    active: true,
    currentWindow: true,
  });

  const response = await rtm.tabs.sendMessage(activeTab.id, message);

  console.log(response);

  return response
}
