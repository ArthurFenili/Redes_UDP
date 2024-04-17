use std::net::UdpSocket;
use std::io::{self, Write};
use serde::{Serialize, Deserialize};
use crc32fast::Hasher;

#[derive(Serialize, Deserialize, Debug)]
struct Packet {    
    sequence_number: u32,
    data: Vec<u8>,
    checksum: u32,
}

impl Packet {
    fn calculate_checksum(&mut self) -> u32 {
        let mut hasher = Hasher::new();
        hasher.update(&self.data);
        hasher.finalize()
    }
}

fn main() -> io::Result<()> {
    // configuração de ip e porta pra criação do socket
    let server_ip = "192.168.1.8";
    let server_port = 3002; 
    let server_addr = format!("{}:{}", server_ip, server_port);

    let socket = UdpSocket::bind("0.0.0.0:0")?;

    println!("Escolha a opção:");
    println!("1. GET (enviar arquivo completo)");
    println!("2. TEG (enviar arquivo corrompido)");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice)?;
    let choice: u32 = choice.trim().parse().expect("Opção inválida");

    let mut received_data = Vec::new();
    let mut last_packet: u32 = 0;
    let mut resended = false;
    
    // loop para esperar um arquivo que exista
    loop {
        println!("Digite o nome do arquivo:");
        let mut filename = String::new();
        io::stdin().read_line(&mut filename)?;

        let filename = filename.trim();

        let message = match choice {
            1 => format!("GET /{}", filename),
            2 => format!("TEG /{}", filename),
            _ => panic!("Opção inválida"),
        };


        // envia a mensagem criada para o servidor
        socket.send_to(message.as_bytes(), &server_addr)?;
        socket.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;

        let mut message = String::new();

        // loop pra receber os pacotes
        loop {
            let mut buffer = [0; 10000];

            // recebe o pacote do socket e guarda no buffer
            match socket.recv_from(&mut buffer) {
                // verifica se o servidor avisou que o arquivo não existe
                Ok((amt, _)) => {
                    match bincode::deserialize::<String>(&buffer[..amt]) {
                        Ok(msg) => {
                            message = msg;
                            if message == "Arquivo não encontrado" {
                                println!("Arquivo não encontrado...");
                                break;
                            }
                        }
                        Err(_) => {}
                    }

                    // verifica se os pacotes acabaram
                    if amt == 0 {
                        break;
                    }

                    let mut packet: Packet = bincode::deserialize(&buffer[..amt]).unwrap();
                    println!("Pacote {} recebido", packet.sequence_number);
                    resended = false;

                    // verifica se os pacotes chegaram na ordem certa
                    let mut response = [0; 4];
                    if packet.sequence_number != 0 && packet.sequence_number != last_packet + 1 {
                        response.copy_from_slice(&(last_packet + 1).to_be_bytes());
                        socket.send_to(&response, &server_addr);

                        // se não chegaram, espera o servidor reenviar
                        loop {
                            let mut new_buffer = [0; 10000];
                            match socket.recv_from(&mut new_buffer) {
                                Ok((amt, _)) => {
                                    let mut new_packet: Packet = bincode::deserialize(&new_buffer[..amt]).unwrap();
                                    println!("Pacote {} recebido NOVAMENTE", new_packet.sequence_number);
                                    received_data.extend_from_slice(&new_packet.data);
                                    last_packet = new_packet.sequence_number;
                                    resended = true;
                                    if new_packet.sequence_number == packet.sequence_number {
                                        break;
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                    }

                    // verifica se o pacote veio corrompido
                    let mut response = [0; 4];
                    let checksum = packet.calculate_checksum();
                    if checksum != packet.checksum {
                        response.copy_from_slice(&(packet.sequence_number).to_be_bytes());
                        socket.send_to(&response, &server_addr);

                        // se veio corrompido, espera o servidor reenviar
                        loop {
                            let mut new_buffer = [0; 10000];
                            match socket.recv_from(&mut new_buffer) {
                                Ok((amt, _)) => {
                                    let mut new_packet: Packet = bincode::deserialize(&new_buffer[..amt]).unwrap();
                                    println!("Pacote {} recebido NOVAMENTE", new_packet.sequence_number);
                                    received_data.extend_from_slice(&new_packet.data);
                                    last_packet = new_packet.sequence_number;
                                    resended = true;
                                    if new_packet.sequence_number == packet.sequence_number {
                                        break;
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                    }

                    // se não foi reenviado, os dados recebidos são os do pacote original
                    if resended == false {
                        //println!("checksum: {}", packet.checksum);
                        received_data.extend_from_slice(&packet.data);
                        last_packet = packet.sequence_number;
                    }
                    
                }
                Err(err) => {
                    if let Some(io_err) = err.raw_os_error() {
                        if io_err == 10060 {
                            println!("Timeout de recebimento. Saindo do loop.");
                            break;
                        }
                    }
                    eprintln!("Erro ao receber dados: {}", err);
                }
            }
        }
        if message != "Arquivo não encontrado" {
            break;
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
