import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { QRCodeCanvas } from "qrcode.react"; // import
import "./App.css";

const App: React.FC = () => {
  const [ticket, setTicket] = useState<string>("");
  const [joinTicket, setJoinTicket] = useState<string>("");
  const [message, setMessage] = useState<string>("");
  const [chatLog, setChatLog] = useState<string[]>([]);

  useEffect(() => {
    const unlistenPromise = listen<string>("new-message", (event) => {
      setChatLog((prev) => [...prev, event.payload]);
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const createRoom = async () => {
    try {
      const result: string = await invoke("create_chat_room");
      setTicket(result);
    } catch (err) {
      console.error("Error creating room:", err);
    }
  };

  const joinRoom = async () => {
    try {
      await invoke("join_chat_room", { ticket: joinTicket });
    } catch (err) {
      console.error("Error joining room:", err);
    }
  };

  const sendMsg = async () => {
    try {
      await invoke("send_message", { message });
      setChatLog((prev) => [...prev, ⁠ Me: ${message} ⁠]);
      setMessage("");
    } catch (err) {
      console.error("Error sending message:", err);
    }
  };

  return (
    <div style={{ margin: "2rem" }}>
      <h2>P2P Chat</h2>

      <section>
        <h3>Create Chat Room</h3>
        <button onClick={createRoom}>Create Room</button>
        {ticket && (
          <div>
            <p>Your room ticket: <code>{ticket}</code></p>
            {/* Fixed QR Code generation */}
            <QRCodeCanvas value={ticket} size={150} />
          </div>
        )}
      </section>

      <section>
        <h3>Join Chat Room</h3>
        <input
          type="text"
          value={joinTicket}
          onChange={(e) => setJoinTicket(e.target.value)}
          placeholder="Enter room ticket"
        />
        <button onClick={joinRoom}>Join Room</button>
      </section>

      <section>
        <h2>Chat</h2>
        <div
          style={{
            border: "1px solid #ccc",
            padding: "1rem",
            height: "160px",
            overflowY: "scroll",
          }}
        >
          {chatLog.map((msg, idx) => (
            <p key={idx}>{msg}</p>
          ))}
        </div>
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          placeholder="Type your message"
          style={{ width: "70%" }}
        />
        <button onClick={sendMsg}>Send</button>
      </section>
    </div>
  );
};

export default App;
