// TODO: Rename mod?

pub mod i3_ipc {
    use byteorder::{LittleEndian, ReadBytesExt};
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::io;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    // Enums, Structs
    #[derive(Debug, Serialize, Deserialize)]
    struct Rect {
        x: u64,
        y: u64,
        width: u64,
        height: u64,
    }

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

    #[derive(Debug, Serialize, Deserialize)]
    pub struct CommandResponse {
        success: bool,
        #[serde(default)]
        parse_error: bool,
    }

    #[derive(Debug)]
    pub enum I3ResponseType {
        RunCommand(Vec<CommandResponse>),
        GetWorkspace(Vec<WorkspaceResponse>),
        None,
    }

    #[derive(Debug)]
    pub enum I3MessageType {
        RunCommand = 0,
        GetWorkspace = 1,
    }

    impl I3MessageType {
        fn from_u32(value: u32) -> I3MessageType {
            match value {
                0 => I3MessageType::RunCommand,
                1 => I3MessageType::GetWorkspace,
                _ => I3MessageType::GetWorkspace,
            }
        }
    }

    pub trait I3Command {
        fn send_i3_command(
            &mut self,
            message_type: I3MessageType,
            payload: &str,
        ) -> io::Result<Vec<u8>>;
        fn recv_i3_command(&mut self) -> io::Result<(I3MessageType, Vec<u8>)>;
    }

    impl I3Command for UnixStream {
        // Message bytes:
        //     00 - 05: "i3-ipc"
        //     06 - 09: message length (little endian)
        //     10 - 14: message type 0 - 12 (little endian)
        //     15 - xx: payload (big endian?)
        fn send_i3_command(
            &mut self,
            message_type: I3MessageType,
            payload: &str,
        ) -> io::Result<Vec<u8>> {
            let mut message: Vec<u8> = Vec::with_capacity(6 + 4 + 4 + payload.len());
            message.extend_from_slice("i3-ipc".as_bytes()); // 00 - 05
            message.extend((payload.len() as u32).to_le_bytes()); // 06 - 09
            message.extend((message_type as u32).to_le_bytes()); // 10 - 14
            message.extend_from_slice(payload.as_bytes()); // 15 - xx
            self.write_all(&message)?;
            Ok(message)
        }

        // Check for 'i3-ipc', get message length and type, get message
        fn recv_i3_command(&mut self) -> io::Result<(I3MessageType, Vec<u8>)> {
            let mut i3_ipc_check = [0u8; 6];
            self.read_exact(&mut i3_ipc_check)?;
            if String::from_utf8_lossy(&i3_ipc_check) != String::from("i3-ipc") {
                println!("IPC did not return 'i3-ipc'");
            }
            let message_length = self.read_u32::<LittleEndian>()?;
            let message_type = I3MessageType::from_u32(self.read_u32::<LittleEndian>()?);
            let mut message = vec![0u8; message_length as usize];
            self.read_exact(&mut message)?;
            Ok((message_type, message))
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

    pub fn response_to_json(
        message_type: I3MessageType,
        response: &str,
    ) -> serde_json::Result<I3ResponseType> {
        match message_type {
            I3MessageType::RunCommand => {
                let response_json: Vec<CommandResponse> = serde_json::from_str(&response)?;
                Ok(I3ResponseType::RunCommand(response_json))
            }
            I3MessageType::GetWorkspace => {
                let response_json: Vec<WorkspaceResponse> = serde_json::from_str(&response)?;
                Ok(I3ResponseType::GetWorkspace(response_json))
            }
            _ => Ok(I3ResponseType::None),
        }
    }
}
