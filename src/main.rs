mod ipc;

use crate::ipc::i3_ipc;
use ipc::i3_ipc::Command;
use std::io;

fn main() -> io::Result<()> {
    let mut stream = i3_ipc::get_stream()?;

    // let _response = match stream.send_and_recv_command(i3_ipc::Message::GetWorkspace, "") {
    //     Ok(i3_ipc::Response::GetWorkspace(response)) => response,
    //     _ => vec![i3_ipc::WorkspaceResponse::new()],
    // };
    // println!("{:#?}\n", response);

    let _run_command_response = stream
        .send_and_recv_command(i3_ipc::Message::RunCommand, "focus left")
        .unwrap();
    println!("{:#?}", _run_command_response);

    let payload = stream.subscribe(vec![i3_ipc::Event::Workspace, i3_ipc::Event::Output]);
    println!("Payload: {:#?}", payload);

    Ok(())
}
