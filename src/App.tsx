import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const App: React.FC = () => {
  const [ticket, setTicket] = useState<string>('');
  const [joinTicket, setJoinTicket] = useState<string>('');
  const [message, setMessage] = useState<string>('');
  const [chatLog, setChatLog] = useState<string[]>([]);

  // Listen for incoming messages from the backend.
  useEffect(() => {
    const unlistenPromise = listen<string>('new-message', (event) => {
      setChatLog((prev) => [...prev, event.payload]);
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  // Create a new chat room.
  const createRoom = async () => {
    try {
      const result: string = await invoke('create_chat_room');
      setTicket(result);
    } catch (err) {
      console.error('Error creating room:', err);
    }
  };

  // Join an existing chat room using the provided ticket.
  const joinRoom = async () => {
    try {
      await invoke('join_chat_room', { ticket: joinTicket });
    } catch (err) {
      console.error('Error joining room:', err);
    }
  };

  // Send a message on the active chat session.
  const sendMsg = async () => {
    try {
      await invoke('send_message', { message });
      setChatLog((prev) => [...prev, `Me: ${message}`]);
      setMessage('');
    } catch (err) {
      console.error('Error sending message:', err);
    }
  };

  return (
    <div style={{ margin: '2rem' }}>
      <h1>Tauri P2P Chat App</h1>

      <section>
        <h2>Create Chat Room</h2>
        <button onClick={createRoom}>Create Room</button>
        {ticket && (
          <p>
            Your room ticket: <code>{ticket}</code>
          </p>
        )}
      </section>

      <section>
        <h2>Join Chat Room</h2>
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
        <div style={{ border: '1px solid #ccc', padding: '1rem', height: '200px', overflowY: 'scroll' }}>
          {chatLog.map((msg, idx) => (
            <p key={idx}>{msg}</p>
          ))}
        </div>
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          placeholder="Type your message"
          style={{ width: '70%' }}
        />
        <button onClick={sendMsg}>Send</button>
      </section>
    </div>
  );
};

export default App;
