pub mod i3_ipc {
    use byteorder::{LittleEndian, ReadBytesExt};
    use pretty_hex;
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::io;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    // Enums, Structs
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Rect {
        x: u64,
        y: u64,
        width: u64,
        height: u64,
    }

    // Responses
    #[derive(Debug, Serialize, Deserialize)]
    pub struct WorkspaceResponse {
        num: u64,
        name: String,
        visible: bool,
        focused: bool,
        urgent: bool,
        rect: Rect,
        output: String,
    }

    impl WorkspaceResponse {
        pub fn new() -> WorkspaceResponse {
            WorkspaceResponse {
                num: 0,
                name: "".to_string(),
                visible: false,
                focused: false,
                urgent: false,
                rect: Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                },
                output: "".to_string(),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CommandResponse {
        success: bool,
        #[serde(default)]
        parse_error: bool,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SubscribeResponse {
        success: bool,
    }

    #[derive(Debug)]
    pub enum Response {
        RunCommand(Vec<CommandResponse>),
        GetWorkspace(Vec<WorkspaceResponse>),
        Subscribe(SubscribeResponse),
        None,
    }

    // Messages
    #[derive(Debug)]
    pub enum Message {
        RunCommand = 0,
        GetWorkspace = 1,
        Subscribe = 2,
    }

    impl Message {
        fn from_u32(value: u32) -> Message {
            match value {
                0 => Message::RunCommand,
                1 => Message::GetWorkspace,
                _ => Message::GetWorkspace,
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum WorkspaceChange {
        Focus,
        Init,
        Empty,
        Urgent,
        Reload,
        Rename,
        Restored,
        Move,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct WorkspaceEvent {
        change: WorkspaceChange,
        current: WorkspaceResponse,
        old: WorkspaceResponse,
    }

    pub enum Event {
        Workspace,
        Output,
        Mode,
        Window,
        BarConfigUpdate,
        Binding,
        Shutdown,
        Tick,
    }

    impl Event {
        fn as_str(&self) -> String {
            // TODO: Learn about static lifetimes b/c I think it would be
            // better here but I don't really know how they work
            match &self {
                Event::Workspace => "workspace".to_string(),
                Event::Output => "output".to_string(),
                Event::Mode => "mode".to_string(),
                Event::Window => "window".to_string(),
                Event::BarConfigUpdate => "barconfigupdate".to_string(),
                Event::Binding => "binding".to_string(),
                Event::Shutdown => "shutdown".to_string(),
                Event::Tick => "tick".to_string(),
            }
        }
    }

    pub trait Command {
        fn send_i3_command(&mut self, message_type: Message, payload: &str) -> io::Result<Vec<u8>>;
        fn recv_i3_command(&mut self) -> io::Result<(Message, Vec<u8>)>;
        fn send_and_recv_command(
            &mut self,
            message_type: Message,
            payload: &str,
        ) -> io::Result<Response>;
        fn subscribe(&mut self, events: Vec<Event>) -> io::Result<Response>;
    }

    impl Command for UnixStream {
        // Message bytes:
        //     00 - 05: "i3-ipc"
        //     06 - 09: message length (little endian)
        //     10 - 14: message type 0 - 12 (little endian)
        //     15 - xx: payload (big endian?)
        fn send_i3_command(&mut self, message_type: Message, payload: &str) -> io::Result<Vec<u8>> {
            let mut message: Vec<u8> = Vec::with_capacity(6 + 4 + 4 + payload.len());
            message.extend_from_slice("i3-ipc".as_bytes()); // 00 - 05
            message.extend((payload.len() as u32).to_le_bytes()); // 06 - 09
            message.extend((message_type as u32).to_le_bytes()); // 10 - 14
            message.extend_from_slice(payload.as_bytes()); // 15 - xx
            self.write_all(&message)?;
            Ok(message)
        }

        // Check for 'i3-ipc', get message length and type, get message
        fn recv_i3_command(&mut self) -> io::Result<(Message, Vec<u8>)> {
            let mut i3_ipc_check = [0u8; 6];
            self.read_exact(&mut i3_ipc_check)?;
            if String::from_utf8_lossy(&i3_ipc_check) != String::from("i3-ipc") {
                println!("IPC did not return 'i3-ipc'");
            }
            let message_length = self.read_u32::<LittleEndian>()?;
            let message_type = Message::from_u32(self.read_u32::<LittleEndian>()?);
            let mut message = vec![0u8; message_length as usize];
            self.read_exact(&mut message)?;
            Ok((message_type, message))
        }

        fn send_and_recv_command(
            &mut self,
            message_type: Message,
            payload: &str,
        ) -> io::Result<Response> {
            let _sent_message = self.send_i3_command(message_type, payload)?;
            println!("Sent message:\n{}", pretty_hex::pretty_hex(&_sent_message));
            let (recv_message_type, recv_message) = self.recv_i3_command()?;
            let response = String::from_utf8_lossy(&recv_message);
            println!("String response: {}", response);
            response_to_json(recv_message_type, &response)
        }

        fn subscribe(&mut self, events: Vec<Event>) -> io::Result<Response> {
            let payload = format!(
                "[{}]",
                events
                    .iter()
                    .map(|event| format!("\"{}\"", event.as_str()))
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            let subscribe_response = self.send_and_recv_command(Message::Subscribe, &payload)?;
            Ok(subscribe_response)
        }
    }

    // Functions
    pub fn get_stream() -> io::Result<UnixStream> {
        let socket_path = match env::var("I3SOCK") {
            Ok(path) => path,
            _ => "/run/user/1000/i3/ipc-socket.1684".to_string(),
        };
        UnixStream::connect(&socket_path)
    }

    pub fn response_to_json(message_type: Message, response: &str) -> io::Result<Response> {
        match message_type {
            Message::RunCommand => {
                let response_json: Vec<CommandResponse> = serde_json::from_str(&response)?;
                Ok(Response::RunCommand(response_json))
            }
            Message::GetWorkspace => {
                let response_json: Vec<WorkspaceResponse> = serde_json::from_str(&response)?;
                Ok(Response::GetWorkspace(response_json))
            }
            // FIX: Not parsing: "invalid type: map, expected a sequence"
            Message::Subscribe => {
                let response_json: SubscribeResponse = serde_json::from_str(&response)?;
                Ok(Response::Subscribe(response_json))
            }
            _ => Ok(Response::None),
        }
    }
}
