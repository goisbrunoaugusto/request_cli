use std::collections::HashSet;
use sled::Db;
use std::error::Error;
use serde_json::json;
use crate::loader::Request;

pub struct Database {
    db: Db,
    excluded_endpoints: HashSet<String>, // Lista de exclusão
}

impl Database {
    // Inicializa o banco de dados
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let db = sled::open(path)?;
        Ok(Self {
            db,
            excluded_endpoints: HashSet::new(),
        })
    }

    // Salva uma requisição no banco de dados
    pub fn save_request(
        &self,
        request: &Request,
        response_status: &str,
        response_body: &str,
    ) -> Result<(), Box<dyn Error>> {
        // Verifica se o endpoint está na lista de exclusão
        if self.excluded_endpoints.contains(&request.url) {
            println!(
                "Requisição para o endpoint '{}' não será salva (excluída).",
                request.url
            );
            return Ok(());
        }

        let key = format!("request:{}", chrono::Utc::now().timestamp_millis());
        let value = json!({
            "name": &request.name,
            "method": &request.method.to_string(),
            "url": &request.url,
            "headers": &request.headers,
            "body": &request.body,
            "response_status": response_status,
            "response_body": response_body,
        })
        .to_string();

        self.db.insert(key.into_bytes(), value.into_bytes())?;
        Ok(())
    }

    // Retorna todas as requisições salvas no banco
    pub fn list_requests(&self) -> Result<Vec<(String, String)>, Box<dyn Error>> {
        let mut requests = Vec::new();
        for item in self.db.iter() {
            let (key, value) = item?;
            let key = String::from_utf8(key.to_vec())?;
            let value = String::from_utf8(value.to_vec())?;
            requests.push((key, value));
        }
        Ok(requests)
    }

    // Adiciona um endpoint à lista de exclusão
    pub fn exclude_endpoint(&mut self, endpoint: String) {
        self.excluded_endpoints.insert(endpoint);
    }

    // Lista todos os endpoints excluídos
    pub fn list_excluded_endpoints(&self) -> Vec<String> {
        self.excluded_endpoints.iter().cloned().collect()
    }

    // Remove um endpoint da lista de exclusão
    pub fn remove_excluded_endpoint(&mut self, endpoint: &str) {
        self.excluded_endpoints.remove(endpoint);
    }
}
