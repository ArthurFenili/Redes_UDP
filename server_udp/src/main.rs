use std::net::{UdpSocket};
use std::io::{self, Read};
use std::fs::File;

fn main() -> std::io::Result<()> {
    // configuração do endereço de ip e da porta
    let ip = "192.168.1.8";
    let port = 8080;
    let addr = format!("{}:{}", ip, port);

    // criação do socket udp
    let socket = UdpSocket::bind(addr)?;

    println!("Servidor UDP iniciado. Esperando por mensagens...");

    loop {
        // Buffer para armazenar os dados recebidos
        let mut buf = [0; 1024];
        // Recebe dados do cliente
        let (amt, src) = socket.recv_from(&mut buf)?;
        // Exibe os dados recebidos
        println!("Recebido {} bytes de {}", amt, src);
        // Converte os dados em uma string
        let received = std::str::from_utf8(&buf[..amt]).expect("Erro ao converter bytes para string");
        println!("Mensagem recebida: {}", received);

        // Verifica se a mensagem é uma requisição GET /arquivo
        if received.starts_with("GET /") {
            // Extrai o nome do arquivo da mensagem
            let filename = received.split_whitespace().nth(1).unwrap_or("").strip_prefix("/").unwrap_or("");

            match File::open(filename) {
                Ok(mut file) => {
                    // Buffer para armazenar o conteúdo do arquivo
                    let mut file_content = Vec::new();
                    // Lê o conteúdo do arquivo para o buffer
                    file.read_to_end(&mut file_content)?;

                    // Envia o conteúdo do arquivo para o cliente
                    socket.send_to(&file_content, &src)?;
                }
                Err(_) => {
                    // Arquivo não encontrado, envia uma resposta de erro para o cliente
                    let response = "Arquivo não encontrado";
                    socket.send_to(response.as_bytes(), &src)?;
                }
            }
        } else {
            // Mensagem inválida, envia uma resposta de erro
            let response = "Requisição inválida. Utilize o formato GET /arquivo";
            socket.send_to(response.as_bytes(), &src)?;
        }
    }
}