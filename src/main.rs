use std::env;
use std::os::unix::net::UnixStream;

fn get_stream() -> Option<UnixStream> {
    let socket_path = match env::var("I3SOCK") {
        Ok(path) => path,
        _ => "/run/user/1000/i3/ipc-socket.1684".to_string(),
    };
    match UnixStream::connect(&socket_path) {
        Ok(s) => {
            println!("Connected to socket at {socket_path:?}");
            Some(s)
        }
        _ => {
            println!("Failed to open socket at {socket_path:?}");
            None
        }
    }
}

fn main() {
    get_stream();
}
