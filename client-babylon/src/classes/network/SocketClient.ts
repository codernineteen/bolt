/**
 * Socket Client class
 */
class SocketClient {
  readonly SERVER_URI: string = "ws://127.0.0.1:8080";
  private socket: WebSocket;

  constructor() {
    this.socket = new WebSocket(this.SERVER_URI);

    this.socket.addEventListener("open", (_ev) => {
      console.log("connected");
    });

    this.socket.addEventListener("error", (ev) => {
      console.log("error : ", ev);
    });

    this.socket.addEventListener("message", (ev) => {
      console.log("message from server " + ev.data);
    });

    this.socket.addEventListener("close", (_ev) => {
      console.log("disconnected");
    });
  }
}

export default SocketClient;
