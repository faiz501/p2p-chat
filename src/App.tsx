import React, { useState, useEffect } from 'react';
import { core } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { QRCodeSVG } from 'qrcode.react';
import "./App.css";

const App: React.FC = () => {
  const [ticket, setTicket] = useState<string>('');
  const [joinTicket, setJoinTicket] = useState<string>('');
  const [message, setMessage] = useState<string>('');
  const [chatLog, setChatLog] = useState<string[]>([]);

  // Generate cryptographic key in Rust backend
  const generateCryptoKey = async (): Promise<string> => {
    return await core.invoke('generate_crypto_key');
  };

  // Modified createRoom with crypto key generation
  const createRoom = async () => {
    try {
      const cryptoKey = await generateCryptoKey();
      const result = await core.invoke('create_chat_room', { cryptoKey });
      setTicket(cryptoKey); // Store the cryptographic key as ticket
    } catch (err) {
      console.error('Error creating room:', err);
    }
  };

  // Join an existing chat room using the provided ticket.
  const joinRoom = async () => {
    try {
      await core.invoke('join_chat_room', { ticket: joinTicket });
    } catch (err) {
      console.error('Error joining room:', err);
    }
  };

  // Send a message on the active chat session.
  const sendMsg = async () => {
    try {
      await core.invoke('send_message', { message });
      setChatLog((prev) => [...prev, `Me: ${message}`]);
      setMessage('');
    } catch (err) {
      console.error('Error sending message:', err);
    }
  };

  return (
    <div style={{ margin: '2rem' }}>
      <h2>P2P Chat</h2>

      <section>
        <h3>Create Chat Room</h3>
        <button onClick={createRoom}>Create Room</button>
        {ticket && (
          <div>
            <p>Your secure room key:</p>
            <code>{ticket}</code>
            <div style={{ margin: '1rem 0' }}>
            <QRCodeSVG
                value={ticket}
                size={128}
                level="H"
                includeMargin={true}
            />
            </div>
          </div>
        )}
      </section>

      <section>
        <h3>Join Chat Room</h3>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <input
            type="text"
            value={joinTicket}
            onChange={(e) => setJoinTicket(e.target.value)}
            placeholder="Enter room key or scan QR"
          />
          <button onClick={joinRoom}>Join Room</button>
        </div>
      </section>

      <section> 
        <h2>Chat</h2> 
        <div style={{ border: '1px solid #ccc', padding: '1rem', height: '160px', overflowY: 'scroll' }}> 
          {chatLog.map((msg, idx) => (<p key={idx}>{msg}</p>))} 
        </div> 
        <input type="text" value={message} onChange={(e) => setMessage(e.target.value)} placeholder="Type your message" style={{ width: '70%' }} /> 
        <button onClick={sendMsg}>Send</button> 
      </section>
    </div>
  );
};

export default App;