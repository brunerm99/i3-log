mod ipc;

use crate::ipc::i3_ipc;
use ipc::i3_ipc::I3Command;
use pretty_hex::*;
use std::io;

fn main() -> io::Result<()> {
    let mut stream = i3_ipc::get_stream()?;

    // GetWorkspace testing
    let message = stream.send_i3_command(i3_ipc::I3MessageType::GetWorkspace, "")?;
    println!("Sent message:\n{}", pretty_hex(&message));
    let (recv_message_type, recv_message) = stream.recv_i3_command()?;
    println!("Message received: {:#?}", recv_message_type);
    let response = String::from_utf8_lossy(&recv_message); // TODO: Move this to recv_i3_command()
    let response_json = i3_ipc::response_to_json(recv_message_type, &response)?;
    println!("{:#?}", response_json);

    // RunCommand testing
    let message = stream.send_i3_command(i3_ipc::I3MessageType::RunCommand, "workspace 8")?;
    println!("Sent message:\n{}", pretty_hex(&message));
    let (recv_message_type, recv_message) = stream.recv_i3_command()?;
    let response = String::from_utf8_lossy(&recv_message); // TODO: Move this to recv_i3_command()
    let response_json = i3_ipc::response_to_json(recv_message_type, &response)?;
    println!("{:#?}", response_json);

    Ok(())
}
