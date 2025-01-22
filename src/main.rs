mod loader;
mod database;

use crate::database::Database;
use crate::loader::{load_requests_from_txt, Request};
use reqwest::blocking::{Client, Response};
use reqwest::Method;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};


fn main() {
    let mut db = Database::new("request_db").expect("Erro ao inicializar o banco de dados");
    let client = Client::new();
    let mut requests: Vec<Request> = Vec::new();

    println!(
r#" __  __     ______   ______   ______     ______     __    __     __     __   __     ______     __        
/\ \_\ \   /\__  _\ /\__  _\ /\  ___\   /\  == \   /\ "-./  \   /\ \   /\ "-.\ \   /\  __ \   /\ \       
\ \  __ \  \/_/\ \/ \/_/\ \/ \ \  __\   \ \  __<   \ \ \-./\ \  \ \ \  \ \ \-.  \  \ \  __ \  \ \ \____  
 \ \_\ \_\    \ \_\    \ \_\  \ \_____\  \ \_\ \_\  \ \_\ \ \_\  \ \_\  \ \_\\"\_\  \ \_\ \_\  \ \_____\ 
  \/_/\/_/     \/_/     \/_/   \/_____/   \/_/ /_/   \/_/  \/_/   \/_/   \/_/ \/_/   \/_/\/_/   \/_____/                                                                   
"#
);


    loop {
        println!("\n--- htterminal ---");
        println!("1. Listar requisições");
        println!("2. Criar nova requisição");
        println!("3. Executar uma requisição");
        println!("4. Executar uma requisição e exportar resultado");
        println!("5. Carregar requisições de um arquivo");
        println!("6. Salvar requisiões de um endpoint no banco");
        println!("7. Listar requisições salvas no banco");
        println!("8. Excluir requisição do banco");
        println!("9. Sair");
        print!("Escolha uma opção: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => list_requests(&requests),
            "2" => create_request(&mut requests),
            "3" => execute_request(&requests, &client, &db),
            "4" => execute_and_export_request(&requests, &client),
            "5" => load_from_file(&mut requests),
            "6" => save_requests_to_db(&requests, &db),
            "7" => list_saved_requests(&db),
            "8" => manage_excluded_endpoints(&mut db),
            "9" => {
                println!("Saindo...");
                break;
            }
            _ => println!("Opção inválida! Tente novamente."),
        }
    }
}

fn list_requests(requests: &[Request]) {
    if requests.is_empty() {
        println!("Nenhuma requisição criada.");
        return;
    }
    for (i, req) in requests.iter().enumerate() {
        println!("{}: [{}] {} {}", i + 1, req.name, req.method, req.url);
    }
}

fn create_request(requests: &mut Vec<Request>) {
    let mut name = String::new();
    let mut method = String::new();
    let mut url = String::new();
    let mut headers = HashMap::new();
    let mut body = String::new();

    print!("Nome da requisição: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut name).unwrap();

    print!("Método (GET, POST, PUT, DELETE, etc.): ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut method).unwrap();

    print!("URL: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut url).unwrap();

    loop {
        print!("Adicionar cabeçalho? (s/n): ");
        io::stdout().flush().unwrap();
        let mut add_header = String::new();
        io::stdin().read_line(&mut add_header).unwrap();

        if add_header.trim().eq_ignore_ascii_case("n") {
            break;
        }

        let mut header_key = String::new();
        let mut header_value = String::new();

        print!("Chave: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut header_key).unwrap();

        print!("Valor: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut header_value).unwrap();

        headers.insert(
            header_key.trim().to_string(),
            header_value.trim().to_string(),
        );
    }

    if method.trim().eq_ignore_ascii_case("POST") || method.trim().eq_ignore_ascii_case("PUT") {
        print!("Corpo da requisição (JSON): ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut body).unwrap();
    }

    requests.push(Request {
        name: name.trim().to_string(),
        method: method.trim().parse().unwrap_or(Method::GET),
        url: url.trim().to_string(),
        headers,
        body: if body.trim().is_empty() {
            None
        } else {
            Some(body.trim().to_string())
        },
    });

    println!("Requisição criada com sucesso!");
}

fn execute_single_request(client: &Client, request: &Request)
    -> Result<String, Box<dyn std::error::Error>>
{
    let mut req_builder = client.request(request.method.clone(), &request.url);

    for (key, value) in &request.headers {
        req_builder = req_builder.header(key, value);
    }

    if let Some(body) = &request.body {
        req_builder = req_builder.body(body.clone());
    }

    let response = req_builder.send()?;
    let status = response.status();
    let text = response.text()?;
    Ok(format!("Status: {}\nBody:\n{}", status, text))
}

fn execute_request(requests: &[Request], client: &Client, db: &Database) {
    if requests.is_empty() {
        println!("Nenhuma requisição disponível para executar.");
        return;
    }

    println!("Selecione o número da requisição para executar:");
    list_requests(requests);

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let index: usize = match choice.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Entrada inválida.");
            return;
        }
    };

    if index == 0 || index > requests.len() {
        println!("Requisição inválida.");
        return;
    }

    let request = &requests[index - 1];
    println!("\nExecutando requisição...");

    match execute_single_request(client, request) {
        Ok(response) => {
            println!("Resposta:\n{}", response);
            let status_line = response.lines().next().unwrap_or("");
            let body = response.lines().skip(1).collect::<Vec<&str>>().join("\n");

            db.save_request(request, status_line, &body)
                .expect("Erro ao salvar a requisição no banco de dados.");
        }
        Err(e) => println!("Erro ao executar a requisição: {}", e),
    }
}

fn execute_and_export_request(requests: &[Request], client: &Client) {
    if requests.is_empty() {
        println!("Nenhuma requisição disponível para executar.");
        return;
    }

    println!("Selecione o número da requisição para executar e exportar:");
    list_requests(requests);

    let mut choice = String::new();
    if io::stdin().read_line(&mut choice).is_err() {
        println!("Falha na leitura da escolha.");
        return;
    }

    let index: usize = match choice.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Entrada inválida.");
            return;
        }
    };

    if index == 0 || index > requests.len() {
        println!("Requisição inválida.");
        return;
    }

    let request = &requests[index - 1];
    println!("\nExecutando requisição...");

    match execute_single_request(client, request) {
        Ok(res) => {
            println!("Resposta obtida com sucesso!");
            println!("Digite o nome/path do arquivo para exportar o resultado (ex: resultado.txt):");
            let mut file_name = String::new();
            if io::stdin().read_line(&mut file_name).is_err() {
                println!("Falha ao ler o nome do arquivo. Abortando exportação.");
                return;
            }
            let file_name = file_name.trim();
            if file_name.is_empty() {
                println!("Nome do arquivo vazio. Exportação cancelada.");
                return;
            }

            match File::create(&file_name) {
                Ok(mut file) => {
                    if let Err(e) = writeln!(file, "{}", res) {
                        println!("Falha ao escrever no arquivo: {}", e);
                    } else {
                        println!("Resultado exportado com sucesso para '{}'.", file_name);
                    }
                }
                Err(e) => {
                    println!("Falha ao criar/abrir o arquivo '{}': {}", file_name, e);
                }
            }
        }
        Err(e) => println!("Erro ao executar a requisição: {}", e),
    }
}

fn load_from_file(requests: &mut Vec<Request>) {
    print!("Informe o caminho do arquivo: ");
    io::stdout().flush().unwrap();

    let mut path = String::new();
    if io::stdin().read_line(&mut path).is_err() {
        println!("Falha ao ler o caminho do arquivo.");
        return;
    }
    let path = path.trim();

    match load_requests_from_txt(path) {
        Ok(loaded_requests) => {
            println!("{} requisição(ões) carregada(s) do arquivo '{}'.", loaded_requests.len(), path);
            requests.extend(loaded_requests);
        }
        Err(e) => println!("Erro ao carregar requisições do arquivo: {}", e),
    }
}

#[allow(dead_code)]
fn display_response(response: Response) {
    println!("Status: {}", response.status());
    println!("Headers: {:?}", response.headers());

    match response.text() {
        Ok(text) => println!("Body: {}", text),
        Err(e) => println!("Erro ao ler o corpo da resposta: {}", e),
    }
}

fn list_saved_requests(db: &Database) {
    match db.list_requests() {
        Ok(requests) => {
            if requests.is_empty() {
                println!("Nenhuma requisição salva no banco.");
            } else {
                for (key, value) in requests {
                    println!("{}: {}", key, value);
                }
            }
        }
        Err(e) => println!("Erro ao listar requisições do banco: {}", e),
    }
}

fn save_requests_to_db(requests: &[Request], db: &Database) {
    if requests.is_empty() {
        println!("Nenhuma requisição disponível para salvar no banco.");
        return;
    }

    println!("Selecione o número da requisição para salvar no banco:");
    list_requests(requests);

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    let index: usize = match choice.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Entrada inválida.");
            return;
        }
    };

    if index == 0 || index > requests.len() {
        println!("Requisição inválida.");
        return;
    }

    let request = &requests[index - 1];
    match db.save_request(request, "Não executado", "Sem resposta") {
        Ok(_) => println!("Requisição '{}' salva no banco de dados.", request.name),
        Err(e) => println!(
            "Erro ao salvar a requisição '{}' no banco de dados: {}",
            request.name, e
        ),
    }
}

fn manage_excluded_endpoints(db: &mut Database) {
    // Obtenha as requisições salvas no banco
    let requests = match db.list_requests() {
        Ok(reqs) => reqs,
        Err(e) => {
            println!("Erro ao listar requisições salvas: {}", e);
            return;
        }
    };

    if requests.is_empty() {
        println!("Nenhuma requisição salva no banco.");
        return;
    }

    // Exibir requisições salvas com status de exclusão
    println!("Requisições salvas no banco:");
    for (i, (key, value)) in requests.iter().enumerate() {
        let is_excluded = db.list_excluded_endpoints().contains(key);
        println!("{}: {} [{}]", i + 1, value, if is_excluded { "Excluído" } else { "Incluído" });
    }

    // Escolha entre adicionar ou remover da lista de exclusão
    println!("\n1. Adicionar requisição à lista de exclusão");
    println!("2. Remover requisição da lista de exclusão");
    println!("Escolha uma opção: ");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    match choice.trim() {
        "1" => {
            println!("Escolha o número da requisição para adicionar à lista de exclusão:");
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            if let Ok(index) = input.trim().parse::<usize>() {
                if index > 0 && index <= requests.len() {
                    let (key, _) = &requests[index - 1];
                    db.exclude_endpoint(key.clone());
                    println!("Requisição '{}' adicionada à lista de exclusão.", key);
                } else {
                    println!("Opção inválida.");
                }
            } else {
                println!("Entrada inválida.");
            }
        }
        "2" => {
            println!("Escolha o número da requisição para remover da lista de exclusão:");
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            if let Ok(index) = input.trim().parse::<usize>() {
                if index > 0 && index <= requests.len() {
                    let (key, _) = &requests[index - 1];
                    db.remove_excluded_endpoint(key);
                    println!("Requisição '{}' removida da lista de exclusão.", key);
                } else {
                    println!("Opção inválida.");
                }
            } else {
                println!("Entrada inválida.");
            }
        }
        _ => println!("Opção inválida."),
    }
}
