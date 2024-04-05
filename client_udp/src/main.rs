use std::net::UdpSocket;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    // Configuração do endereço IP e porta do servidor
    let server_ip = "192.168.1.8"; // Altere para o IP do servidor
    let server_port = 8080; // Altere para a porta do servidor
    let server_addr = format!("{}:{}", server_ip, server_port);

    // Criação do socket UDP
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // Mensagem a ser enviada ao servidor
    let message = "GET /teste.txt"; // Altere conforme necessário

    // Envia a mensagem para o servidor
    socket.send_to(message.as_bytes(), &server_addr)?;

    // Buffer para receber a resposta do servidor
    let mut buffer = [0; 1024];
    // Recebe a resposta do servidor
    let (amt, _) = socket.recv_from(&mut buffer)?;
    // Exibe a resposta do servidor
    println!("Recebido do servidor: {}", String::from_utf8_lossy(&buffer[..amt]));

    Ok(())
}
