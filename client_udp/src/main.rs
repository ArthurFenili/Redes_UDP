use std::net::UdpSocket;
use std::io::{self, Write};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Packet<'a> {
    sequence_number: u32,
    data: &'a [u8], // Alterado para um slice
}
fn main() -> io::Result<()> {
    // Configuração do endereço IP e porta do servidor
    let server_ip = "192.168.1.8"; // Altere para o IP do servidor
    let server_port = 8080; // Altere para a porta do servidor
    let server_addr = format!("{}:{}", server_ip, server_port);

    // Criação do socket UDP
    let socket = UdpSocket::bind("0.0.0.0:0")?;

    // Solicita ao usuário que escolha entre GET e TEG
    println!("Escolha a opção:");
    println!("1. GET (enviar arquivo completo)");
    println!("2. TEG (enviar arquivo corrompido)");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice: u32 = choice.trim().parse().expect("Opção inválida");

    // Solicita ao usuário que digite o nome do arquivo
    println!("Digite o nome do arquivo:");
    let mut filename = String::new();
    io::stdin().read_line(&mut filename)?;

    // Remove qualquer espaço em branco ou quebra de linha do nome do arquivo
    let filename = filename.trim();

    // Constrói a mensagem de requisição GET ou TEG
    let message = match choice {
        1 => format!("GET /{}", filename),
        2 => format!("TEG /{}", filename),
        _ => panic!("Opção inválida"),
    };


    // Envia a mensagem para o servidor
    socket.send_to(message.as_bytes(), &server_addr)?;
    socket.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;

    // Buffer para armazenar os dados recebidos do servidor
    let mut received_data = Vec::new();
    let mut last_packet :u32 = 0;
    let mut resended = false;
    // Loop para receber os pacotes do servidor
    loop {
        // Buffer para armazenar os dados do pacote recebido
        let mut buffer = [0; 10000];

        match socket.recv_from(&mut buffer) {
            Ok((amt, _)) => {
                // Se não houver mais dados, saia do loop
                if amt == 0 {
                    break;
                }
                let packet: Packet = bincode::deserialize(&buffer[..amt]).unwrap();
                println!("Pacote {} recebido", packet.sequence_number);
                resended = false;
                let mut response = [0; 4];
                if packet.sequence_number != 0 && packet.sequence_number != last_packet + 1 {
                    response.copy_from_slice(&(last_packet + 1).to_be_bytes());
                    socket.send_to(&response, &server_addr);
                    // Espere até que o servidor reenvie o pacote
                    loop {
                        let mut new_buffer = [0; 10000];
                        match socket.recv_from(&mut new_buffer) {
                            Ok((amt, _)) => {
                                let new_packet: Packet = bincode::deserialize(&new_buffer[..amt]).unwrap();
                                println!("Pacote {} recebido NOVAMENTE", new_packet.sequence_number);
                                received_data.extend_from_slice(&new_packet.data);
                                last_packet = new_packet.sequence_number;
                                resended = true;
                                if new_packet.sequence_number == packet.sequence_number {
                                    break;
                                }
                            }
                            Err(_) => continue, // Ignore errors and keep waiting
                        }
                    }
                }
                if resended == false {
                    received_data.extend_from_slice(&packet.data);
                    last_packet = packet.sequence_number;
                }
                
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
    }
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
