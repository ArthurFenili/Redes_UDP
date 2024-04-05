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
    let message = "GET /swiss2.jpg"; // Altere conforme necessário

    // Envia a mensagem para o servidor
    socket.send_to(message.as_bytes(), &server_addr)?;
    socket.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;

    // Buffer para armazenar os dados recebidos do servidor
    let mut received_data = Vec::new();

    // Loop para receber os pacotes do servidor
    loop {
        // Buffer para armazenar os dados do pacote recebido
        let mut buffer = [0; 1024];
        match socket.recv_from(&mut buffer) {
            Ok((amt, _)) => {
                // Se não houver mais dados, saia do loop
                if amt == 0 {
                    break;
                }
                // Armazena os dados recebidos do pacote no buffer geral
                received_data.extend_from_slice(&buffer[..amt]);
            }
            Err(err) => {
                // Se for um timeout, saia do loop
                if let Some(io_err) = err.raw_os_error() {
                    if io_err == 10060 {
                        println!("Timeout de recebimento. Saindo do loop.");
                        break;
                    }
                }
                // Se não for um timeout, exiba o erro e continue
                eprintln!("Erro ao receber dados: {}", err);
            }
        }
        // Aguarde um breve momento para evitar consumir muita CPU no loop
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    println!("saiu");
    let mut file = match std::fs::File::create("arquivo_recebido.jpg") {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Erro ao criar o arquivo: {}", err);
            return Err(err);
        }
    };
    match file.write_all(&received_data) {
        Ok(()) => println!("Arquivo recebido e salvo com sucesso."),
        Err(err) => {
            eprintln!("Erro ao salvar o arquivo: {}", err);
            return Err(err);
        }
    };

    Ok(())
}
