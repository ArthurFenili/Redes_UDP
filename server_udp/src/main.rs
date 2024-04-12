use std::net::{UdpSocket, SocketAddr};
use std::io::{self, Read};
use std::fs::File;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Debug, Serialize, Deserialize)]
struct Packet {    
    sequence_number: u32,
    data: Vec<u8>,
}

impl Packet {
    fn new(sequence_number: u32, data: Vec<u8>) -> Self {
        Packet { sequence_number, data }
    }
}



fn main() -> std::io::Result<()> {
    // configuração do endereço de ip e da porta
    let ip = "192.168.1.8";
    let port = 8080;
    let addr = format!("{}:{}", ip, port);

    // criação do socket udp
    let socket = UdpSocket::bind(addr)?;

    println!("Servidor UDP iniciado. Esperando por mensagens...");

    loop {
        // cria array para armazenar os dados recebidos do client
        let mut buf = [0; 1024];
        
        // recebe os dados do socket e guarda no buf, 
        // recebe a quantidade de bytes recebidos e guarda em amt,
        // recebe a source dos dados e guarda em src
        let (amt, src) = socket.recv_from(&mut buf)?;

        println!("Recebido {} bytes de {}", amt, src);
        let received = std::str::from_utf8(&buf[..amt]).expect("Erro ao converter bytes para string");
        println!("Mensagem recebida: {}", received);

        if received.starts_with("GET /") {
            let filename = received.split_whitespace().nth(1).unwrap_or("").strip_prefix("/").unwrap_or("");

            // cria o buffer do pacote a ser enviado
            let mut buffer = [0; 4096];
            let mut number = 0;
            match File::open(filename) {
                Ok(mut file) => {
                    loop {
                        // le x bytes (a quantidade max do buffer) do arquivo file e salva no buffer, salva também, a quantidade de bytes que foi lida na operação na variavel bytes_read
                        let bytes_read = file.read(&mut buffer)?;
                        if bytes_read == 0 {
                            socket.set_read_timeout(None);
                            break;
                        }

                        // cria e envia um pacote com o numero do pacote e os dados
                        let packet = Packet::new(number, buffer[..bytes_read].to_vec());
                        let serialized_packet = bincode::serialize(&packet).unwrap();
                        socket.send_to(&serialized_packet, &src);

                        socket.set_read_timeout(Some(Duration::from_millis(1)))?;
                        let mut client_response = [0; 4];
                        match socket.recv_from(&mut client_response) {
                            Ok((amt, _)) => {
                                //reenvia pacote
                            }
                            Err(err) => {
                                // Se for um timeout, saia do loop
                                if let Some(io_err) = err.raw_os_error() {
                                    if io_err == 10060 {
                                        println!("Pacote recebido com sucesso... Iniciando transferência do próximo");
                                        continue;
                                    }
                                }
                                // Se não for um timeout, exiba o erro e continue
                                eprintln!("Erro ao receber dados: {}", err);
                            }
                        }
        

                        number += 1;
                    }
                }
                Err(_) => {
                    let response = "Arquivo não encontrado";
                    socket.send_to(response.as_bytes(), &src)?; // envia resposta de arquivo não encontrado para o client
                }
            }
        }
    }
}
