use reqwest::Method;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

#[derive(Debug)]
pub struct Request {
    pub name: String,
    pub method: Method,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Lê um arquivo .txt e monta um vetor de [`Request`].
///
/// Formato esperado em cada bloco:
/// 1) Nome da requisição
/// 2) Método (GET, POST, etc.)
/// 3) URL
/// 4) "header: Y" ou "header: N"
/// 5) Se "header: Y", ler cabeçalhos (linhas "Chave: Valor") até achar "body: Y/N" ou o fim do bloco
/// 6) "body: Y" ou "body: N"
/// 7) Se "body: Y", ler o restante do bloco como corpo da requisição
/// 
/// Cada requisição é separada por uma linha contendo "+".


pub fn load_requests_from_txt(path: &str) -> Result<Vec<Request>, Box<dyn Error>> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Erro ao ler o arquivo '{}': {}", path, e))?;

    let blocks = content.split("+");

    let mut requests = Vec::new();

    for block in blocks {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let mut lines: Vec<String> = block
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();

        if lines.len() < 4 {
            eprintln!("Bloco incompleto encontrado. Ignorando:\n{}", block);
            continue;
        }

        let name = lines.remove(0);

        let method_str = lines.remove(0);
        let method = match method_str.parse::<Method>() {
            Ok(m) => m,
            Err(_) => {
                eprintln!("Método inválido '{}' em '{}'. Usando GET como fallback.", method_str, name);
                Method::GET
            }
        };

        let url = lines.remove(0);

        let header_line = lines.remove(0).to_lowercase();
        let header_flag = header_line.replace("header:", "").trim().to_string(); // "y" ou "n"

        let mut headers = HashMap::new();
        let mut body = None;

        if header_flag == "y" {
            while !lines.is_empty() && !lines[0].to_lowercase().starts_with("body:") {
                let hline = lines.remove(0);
                if let Some((k, v)) = hline.split_once(':') {
                    headers.insert(k.trim().to_string(), v.trim().to_string());
                } else {
                    eprintln!("Linha de cabeçalho malformada em '{}': {}", name, hline);
                }
            }
        }

        if !lines.is_empty() && lines[0].to_lowercase().starts_with("body:") {
            let body_line = lines.remove(0).to_lowercase();
            let body_flag = body_line.replace("body:", "").trim().to_string(); // "y" ou "n"

            if body_flag == "y" {
                if !lines.is_empty() {
                    body = Some(lines.join("\n"));
                }
            }
        }

        let request = Request {
            name,
            method,
            url,
            headers,
            body,
        };
        requests.push(request);
    }

    if requests.is_empty() {
        eprintln!("Aviso: Nenhuma requisição válida foi encontrada em '{}'.", path);
    }

    Ok(requests)
}
