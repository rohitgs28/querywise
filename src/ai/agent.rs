use anyhow::Result;
use super::provider::AiProvider;
use crate::db::SchemaInfo;

pub struct AiAgent {
    provider: AiProvider,
    conversation: Vec<(String, String)>, // (user, assistant) pairs
}

impl AiAgent {
    pub fn new(provider: AiProvider) -> Self {
        Self {
            provider,
            conversation: Vec::new(),
        }
    }

    pub async fn generate_sql(
        &mut self,
        question: &str,
        schema: &SchemaInfo,
    ) -> Result<String> {
        let system = format!(
            "You are an expert SQL query generator for {} databases.\n\n\
             SCHEMA:\n{}\n\n\
             RULES:\n\
             1. Output ONLY the SQL query. No explanations, no markdown code blocks.\n\
             2. Use proper quoting for identifiers.\n\
             3. Prefer explicit JOINs.\n\
             4. LIMIT results to 500 rows unless specified.\n\
             5. If the question references previous results, use the conversation context.\n\
             6. Never generate destructive statements (DROP, DELETE, TRUNCATE).",
            schema.db_type,
            schema.to_ddl()
        );

        // Build context from conversation history
        let context = if self.conversation.is_empty() {
            question.to_string()
        } else {
            let history: String = self
                .conversation
                .iter()
                .map(|(q, sql)| format!("User: {}\nSQL: {}\n", q, sql))
                .collect();
            format!(
                "Previous conversation:\n{}\nNew question: {}",
                history, question
            )
        };

        let response = self.provider.generate(&system, &context).await?;

        // Clean markdown code blocks if present
        let sql = response
            .trim_start_matches("```sql")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string();

        self.conversation.push((question.to_string(), sql.clone()));

        Ok(sql)
    }

    pub async fn fix_query(
        &self,
        original_sql: &str,
        error: &str,
        schema: &SchemaInfo,
    ) -> Result<String> {
        let system = format!(
            "You are a SQL debugging expert for {} databases.\n\n\
             SCHEMA:\n{}\n\n\
             Fix the SQL query based on the error. Output ONLY the corrected SQL, nothing else.",
            schema.db_type,
            schema.to_ddl()
        );

        let message = format!(
            "Original SQL:\n{}\n\nError:\n{}\n\nFix this query.",
            original_sql, error
        );

        let response = self.provider.generate(&system, &message).await?;

        Ok(response
            .trim_start_matches("```sql")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string())
    }

    pub async fn explain_query(&self, sql: &str) -> Result<String> {
        let system = "Explain this SQL query in plain English. Be concise. Use bullet points.";
        let response = self.provider.generate(system, sql).await?;
        Ok(response)
    }

    pub fn clear_conversation(&mut self) {
        self.conversation.clear();
    }
}
