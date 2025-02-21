// App.tsx
import { useEffect, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import { Msg } from "./types/msg";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  // local node id for the chat room (returned when creating a new room)
  const [nodeId, setNodeId] = useState("");
  // input for joining an existing room (node id string)
  const [joinRoomId, setJoinRoomId] = useState("");
  // message to send
  const [sendMessage, setSendMessage] = useState("");
  // latest received message
  const [receivedMsg, setReceivedMsg] = useState("");

  /**
   * Start chat by invoking the `start_chat` Tauri command.
   * If a node id is provided, it will join that chat room.
   * If not, it creates a new chat room and returns your local node id.
   */
  async function startChat(nodeIdParam?: string) {
    try {
      // Pass null if undefined so that Rust knows it's a new chat
      const result = await invoke<string>("start_chat", {
        nodeid: nodeIdParam || null,
      });
      setNodeId(result);
      console.log("Chat started with node id:", result);
    } catch (error) {
      console.error("Error starting chat:", error);
    }
  }

  /**
   * Send a message using the active chat.
   */
  async function handleSendMessage() {
    try {
      await invoke("send_msg", { msg: sendMessage });
      setSendMessage("");
    } catch (error) {
      console.error("Error sending message:", error);
    }
  }

  /**
   * Listen for incoming messages from the background listener.
   * When a new message is received via the Tauri event "new_message",
   * update the UI.
   */
  useEffect(() => {
    const unlisten = listen<string>("new_message", (event) => {
      console.log("New message received:", event.payload);
      setReceivedMsg(event.payload);
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  /**
   * Greet command remains the same.
   */
  async function greet() {
    try {
      const response = await invoke<string>("greet", { name });
      setGreetMsg(response);
    } catch (error) {
      console.error("Error greeting:", error);
    }
  }

  return (
    <main className="container">
      <h1>Tauri Chat App</h1>
      <div>
        <button
          onClick={(e) => {
            e.preventDefault();
            // Create a new chat room (no node id provided)
            startChat();
          }}
        >
          Create New Chat Room
        </button>
        <p>Your Node ID: {nodeId}</p>
      </div>
      <div>
        <input
          type="text"
          placeholder="Join Room (Node ID)"
          value={joinRoomId}
          onChange={(e) => setJoinRoomId(e.target.value)}
        />
        <button
          onClick={(e) => {
            e.preventDefault();
            // Join an existing chat room using the provided node id
            startChat(joinRoomId);
          }}
        >
          Join Existing Chat Room
        </button>
      </div>
      <div>
        <input
          type="text"
          placeholder="Type your message..."
          value={sendMessage}
          onChange={(e) => setSendMessage(e.target.value)}
        />
        <button
          onClick={(e) => {
            e.preventDefault();
            handleSendMessage();
          }}
        >
          Send Message
        </button>
      </div>
      <div>
        <p>Received Message: {receivedMsg}</p>
      </div>
      <hr />
      <div>
        <form
          className="row"
          onSubmit={(e) => {
            e.preventDefault();
            greet();
          }}
        >
          <input
            id="greet-input"
            placeholder="Enter a name..."
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <button type="submit">Greet</button>
        </form>
        <p>{greetMsg}</p>
      </div>
    </main>
  );
}

export default App;
