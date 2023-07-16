import Engine from "./classes/core/Engine";
import SocketClient from "./classes/network/SocketClient";

let socket;
const connectButton = document.querySelector(".connect") as HTMLButtonElement;
if (connectButton) {
  connectButton.addEventListener("click", () => {});
}

window.addEventListener("DOMContentLoaded", () => {
  let engine = new Engine();
  engine.Run();
  socket = new SocketClient();
});
