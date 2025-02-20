import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  const [room, update_room] = useState("");
  const [s_msg, update_snd_msg] = useState("");
  const [r_msg, update_rcv_msg] = useState("");

  async function create_room() {
    update_room("test")
    update_room(await invoke("create_room", { room: room }));
  }
  async function send_msg() {
    update_rcv_msg(await invoke("send_msg", { sMsg: s_msg }));
  }

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="container">

    <button
        onClick={(e) => {
          e.preventDefault();
          create_room();
          console.log("create room")
        }}
      >Create Room</button>
    <p id="crt_rm">{room}</p>
    <input type="text" id="join" placeholder="Join"></input>
    <button
      >Join Room</button>
    <input type="text" id="snd_mdg"
      onChange={(e) => update_snd_msg(e.currentTarget.value)}
      placeholder="Send Message"></input>
    <button
        onClick={(e) => {
          e.preventDefault();
          send_msg();
        }}
      >Send Message</button>
    <p id="rcv_mdg">{r_msg}</p>

      
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://reactjs.org" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>
    </main>
  );
}

export default App;
