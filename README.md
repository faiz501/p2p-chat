# P2P Chat

A peer to peer chat application which is end to end encrypted and does not require login to ensure complete user privacy

## Installation

### Pre-requisites
- [Rust toolchain](https://www.rust-lang.org/tools/install)
- [Deno](https://docs.deno.com/runtime/getting_started/installation/)

### Steps:
```bash
git clone https://github.com/mav3ri3k/p2p-chat.git
cd p2p-chat
deno install

# For Desktop development, run:
deno task tauri dev

# For installing on system
deno task tauri build
```

## Documentation
The app is based around idea of virtual rooms. All the people in a room can talk to one another and maintain a shared chat state.

### Create a Room
1. Click on `Create Room` button.

2. A new room will be created along with room ticket.

3. Share the room ticket with peers through a safe channel who wish to join the room.

### Join a Room

1. Enter the room ticket and click `Join Room` button.

2. You will join the room. Due to peer to peer nature, any previous chats in room are not visible.

### Guidelines

It is advised to never share the rooom ticket over unsafe communication channels.
Ideally it is shared in person only, to maintain privacy.
The room can persist even after network changes.
