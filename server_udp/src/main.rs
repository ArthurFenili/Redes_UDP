use std::net::{UdpSocket, SocketAddr};
use std::io::{self, Read, Seek, SeekFrom};
use std::fs::File;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use crc32fast::Hasher;

#[derive(Debug, Serialize, Deserialize)]

struct Packet {    
    sequence_number: u32,
    data: Vec<u8>,
    checksum: u32,
}

impl Packet {
    fn new(sequence_number: u32, data: Vec<u8>, checksum: u32) -> Self {
        Packet { sequence_number, data, checksum }
    }

    fn calculate_checksum(&mut self) {
        let mut hasher = Hasher::new();
        hasher.update(&self.data);
        self.checksum = hasher.finalize();
    }
}

fn main() -> std::io::Result<()> {
    // configuração de ip e porta pra criação do socket
    let ip = "192.168.1.8";
    let port = 8080;
    let addr = format!("{}:{}", ip, port);

    let socket = UdpSocket::bind(addr)?;

    println!("Servidor UDP iniciado. Esperando por mensagens...");

    loop {
        // buffer pra requisição do client
        let mut buf = [0; 1024];
        
        // recebe os dados do socket e guarda no buf, 
        // recebe a quantidade de bytes recebidos e guarda em amt,
        // recebe a source dos dados e guarda em src
        let (amt, src) = socket.recv_from(&mut buf)?;

        println!("Recebido {} bytes de {}", amt, src);
        let received = std::str::from_utf8(&buf[..amt]).expect("Erro ao converter bytes para string");
        println!("Mensagem recebida: {}", received);

        // verifica se a requisição é GET ou TEG
        if received.starts_with("GET /") || received.starts_with("TEG /") {
            let filename = received.split_whitespace().nth(1).unwrap_or("").strip_prefix("/").unwrap_or("");

            // buffer do pacote a ser enviado
            let mut buffer = [0; 4096];
            let mut number = 0; // numero do pacote
            match File::open(filename) {
                Ok(mut file) => {
                    loop {
                        // le x bytes (a quantidade max do buffer) do arquivo file e salva no buffer, salva também a quantidade de bytes que foi lida na operação na variavel bytes_read
                        let bytes_read = file.read(&mut buffer)?;
                        if bytes_read == 0 {
                            socket.set_read_timeout(None);
                            break;
                        }
                        
                        // cria o pacote, calcula checksum e serializa para mandar
                        let mut packet = Packet::new(number, buffer[..bytes_read].to_vec(), 0);
                        packet.calculate_checksum();
                        let serialized_packet = bincode::serialize(&packet).unwrap();

                        // verifica se a requisição é TEG pra quebrar o pacote
                        if received.starts_with("TEG /") { 
                            // corrompe o pacote
                            if number == 2 {
                                packet.data.remove(17);
                            }
                            // pula o pacote
                            if number != 4 && number != 27 {
                                let serialized_packet = bincode::serialize(&packet).unwrap();
                                socket.send_to(&serialized_packet, &src);
                            }
                            else {
                                number += 1;
                                continue;
                            }
                        }
                        else { // se não for TEG envia o pacote normal
                            socket.send_to(&serialized_packet, &src); 
                        }

                        socket.set_read_timeout(Some(Duration::from_millis(1)))?;
                        let mut client_response = [0; 4];
                        // espera uma possivel resposta do client, se recebeu significa que um pacote não chegou, então reenvia
                        // se não recebeu, inicia transferencia do proximo pacote
                        loop {
                            match socket.recv_from(&mut client_response) {
                                Ok((amt, _)) => {
                                    let response = u32::from_be_bytes(client_response[..amt]
                                        .try_into()
                                        .expect("Erro ao converter bytes para u32"));
                                    
                                    file.seek(SeekFrom::Start((response*4096).into()))?;
                                    println!("Resposta recebida, pacote {} faltando. Reenviando...", response);
                                    number = response;
                                    let bytes_read = file.read(&mut buffer)?;
                                    let mut packet = Packet::new(number, buffer[..bytes_read].to_vec(), 0);
                                    packet.calculate_checksum();
                                    let serialized_packet = bincode::serialize(&packet).unwrap();
                                    socket.send_to(&serialized_packet, &src)?; 
                                    number += 1;
                                    break;
                                    
                                }
                                Err(err) => {
                                    if let Some(io_err) = err.raw_os_error() {
                                        if io_err == 10060 {
                                            println!("Pacote {} recebido com sucesso... Iniciando transferência do próximo", number);
                                            number += 1;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    let response = "Arquivo não encontrado";
                    socket.send_to(&bincode::serialize(&response).unwrap(), &src)?; // envia resposta de arquivo não encontrado para o client
                }
            }
        }
    }
}
