use byteorder::{LittleEndian, ReadBytesExt};
use pretty_hex::*;
use std::env;
use std::io;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

fn get_stream() -> io::Result<UnixStream> {
    let socket_path = match env::var("I3SOCK") {
        Ok(path) => path,
        _ => "/run/user/1000/i3/ipc-socket.1684".to_string(),
    };
    UnixStream::connect(&socket_path)
}

trait I3Command {
    fn send_i3_command(&mut self, message_type: u32, payload: &str) -> io::Result<Vec<u8>>;
    fn recv_i3_command(&mut self, message_type: u32, payload: &str) -> io::Result<Vec<u8>>;
}

impl I3Command for UnixStream {
    fn send_i3_command(&mut self, message_type: u32, payload: &str) -> io::Result<Vec<u8>> {
        // Message bytes:
        //     00 - 05: "i3-ipc"
        //     06 - 09: message length (little endian)
        //     10 - 14: message type 0 - 12 (little endian)
        //     15 - xx: payload (big endian?)
        let mut message: Vec<u8> = Vec::with_capacity(6 + 4 + 4 + payload.len());
        message.extend_from_slice("i3-ipc".as_bytes()); // 00 - 05
        message.extend((payload.len() as u32).to_le_bytes()); // 06 - 09
        message.extend(message_type.to_le_bytes()); // 10 - 14
        message.extend_from_slice(payload.as_bytes()); // 15 - xx
        self.write_all(&message)?;
        Ok(message)
    }
    fn recv_i3_command(&mut self, message_type: u32, payload: &str) -> io::Result<Vec<u8>> {
        let mut i3_ipc_check = [0u8; 6];
        self.read_exact(&mut i3_ipc_check)?;
        if String::from_utf8_lossy(&i3_ipc_check) != String::from("i3-ipc") {
            println!("IPC did not return 'i3-ipc'");
        }
        let message_length = self.read_u32::<LittleEndian>()?;
        let message_type = I3MessageType::from_u32(self.read_u32::<LittleEndian>()?);
        let mut message = vec![0u8; message_length as usize];
        self.read_exact(&mut message)?;
        Ok(message)
    }
}

enum I3MessageType {
    Exit = 0,
    GetWorkspace = 1,
}

impl I3MessageType {
    fn from_u32(value: u32) -> I3MessageType {
        match value {
            0 => I3MessageType::Exit,
            1 => I3MessageType::GetWorkspace,
            _ => I3MessageType::GetWorkspace,
        }
    }
}

fn main() -> io::Result<()> {
    let mut stream = get_stream()?;
    let message = stream.send_i3_command(I3MessageType::GetWorkspace as u32, "exit")?;
    println!("Sent message:\n{}", pretty_hex(&message));
    let recv_message = stream.recv_i3_command(0, "")?;
    println!("{}", pretty_hex(&recv_message));
    Ok(())
}
