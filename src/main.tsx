import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <button>Create Room</button>
    <p id="crt_rm"></p>
    <input type="text" id="join" placeholder="Join"></input>
    <button>Join Room</button>
    <input type="text" id="snd_mdg" placeholder="Send Message"></input>
    <button>Send Message</button>
    <br />
    <p id="rcv_mdg"></p>
    <App />
  </React.StrictMode>,
);
