use std::{io::{Read, Write}, net::TcpStream};


pub fn send_to_tcp_server(message: String, addr: String) -> Result<String, Box<dyn std::error::Error+ Send + Sync>> {
    // Connect to the TCP server
    let mut stream = match TcpStream::connect(addr){
        Ok(data)=>{data},
        Err(err)=>{
            log::error!(" error connecting to TCP stream {}", err.to_string());
            return Err(err.into())
        }
    };

    
    // Send the message to the server
    match stream.write_all(message.as_bytes()){
        Ok(_)=>{},
        Err(err)=>{
            log::error!(" error writting to stream {}", err.to_string());
            return Err(err.into())
        }
    };

    // Read the response from the server
    let mut buffer = vec![0; 1024];
    let n = match stream.read(&mut buffer){
        Ok(n)=>{n},
        Err(err)=>{
            log::error!(" error reading stream {}", err.to_string());
            return Err(err.into())  
        }
    };

    // Convert the response to a string and return
    let response = String::from_utf8_lossy(&buffer[..n]).to_string();
    Ok(response)
}

