use crate::error::AppError;
use crate::services::database::{DatabaseError, DatabaseService};
use crate::services::dataset::Column;
use crate::services::dataset::{Row, RowData};
use crate::services::{DatasetService, ModelService};
use serde_json::Value;
use std::fmt;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

use stop_words::{get, LANGUAGE};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::{BatchAddError, LlamaBatch};
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::token::LlamaToken;
use llama_cpp_2::{
    DecodeError, LLamaCppError, LlamaContextLoadError, LlamaModelLoadError, StringToTokenError, TokenToStringError,
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub max_tokens: usize,
    pub temperature: f32,
    pub top_k: i32,
    pub top_p: f32,
    pub batch_size: usize,
    pub context_size: u32,
    pub add_bos: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            temperature: 0.9,
            top_k: 40,
            top_p: 0.90,
            batch_size: 512,
            context_size: 2048,
            add_bos: true,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowGenerationProgress {
    pub dataset_id: i64,
    pub generation_id: String,
    pub last_row_generated: Row,
    pub total_rows_generated: i64,
    pub total_rows_to_generate: i64,
    pub status: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowGenerationStatus {
    pub generation_id: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug)]
pub enum GenerationError {
    DatabaseError(String),
    FsError(String),
    ModelError(String),
    RegexError(String),
    ParseError(String),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerationError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            GenerationError::FsError(msg) => write!(f, "File system error: {}", msg),
            GenerationError::ModelError(msg) => write!(f, "Model error: {}", msg),
            GenerationError::RegexError(msg) => write!(f, "Regex error: {}", msg),
            GenerationError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for GenerationError {}

impl From<regex::Error> for GenerationError {
    fn from(err: regex::Error) -> Self {
        GenerationError::RegexError(err.to_string())
    }
}

impl From<rusqlite::Error> for GenerationError {
    fn from(err: rusqlite::Error) -> Self {
        GenerationError::DatabaseError(err.to_string())
    }
}

impl From<json5::Error> for GenerationError {
    fn from(err: json5::Error) -> Self {
        GenerationError::ParseError(err.to_string())
    }
}

impl From<DatabaseError> for GenerationError {
    fn from(err: DatabaseError) -> Self {
        GenerationError::DatabaseError(err.to_string())
    }
}

impl From<LLamaCppError> for GenerationError {
    fn from(err: LLamaCppError) -> Self {
        GenerationError::ModelError(err.to_string())
    }
}

impl From<LlamaModelLoadError> for GenerationError {
    fn from(err: LlamaModelLoadError) -> Self {
        GenerationError::ModelError(err.to_string())
    }
}

impl From<LlamaContextLoadError> for GenerationError {
    fn from(err: LlamaContextLoadError) -> Self {
        GenerationError::ModelError(err.to_string())
    }
}

impl From<BatchAddError> for GenerationError {
    fn from(err: BatchAddError) -> Self {
        GenerationError::ModelError(err.to_string())
    }
}

impl From<StringToTokenError> for GenerationError {
    fn from(err: StringToTokenError) -> Self {
        GenerationError::ModelError(err.to_string())
    }
}

impl From<DecodeError> for GenerationError {
    fn from(err: DecodeError) -> Self {
        GenerationError::ModelError(err.to_string())
    }
}

impl From<TokenToStringError> for GenerationError {
    fn from(err: TokenToStringError) -> Self {
        GenerationError::ModelError(err.to_string())
    }
}

static STOP_WORDS: OnceLock<HashSet<String>> = OnceLock::new();

fn get_stop_words() -> &'static HashSet<String> {
    STOP_WORDS.get_or_init(|| {
        let mut all_stop_words = HashSet::new();
        let languages = [
            LANGUAGE::English,
            LANGUAGE::French,
            LANGUAGE::Spanish,
            LANGUAGE::German,
            LANGUAGE::Italian,
            LANGUAGE::Russian,
            LANGUAGE::Arabic,
            LANGUAGE::Chinese,
        ];

        for language in languages {
            all_stop_words.extend(get(language).iter().map(|word| word.to_string()));
        }
        all_stop_words
    })
}

static COLUMN_REF_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_column_ref_regex() -> &'static Regex {
    COLUMN_REF_REGEX.get_or_init(|| Regex::new(r"@(\w+)").expect("Invalid regex pattern"))
}

static CELL_PROMPT_TEMPLATE: &str = r#"Generate a {format} value for column "{column_name}".

Rule: {column_rule}

CRITICAL: Return ONLY the raw value, nothing else. No explanations, no labels, no markdown, no formatting.
Perspective: {persona}

{words_to_avoid}

{return} :"#;

#[derive(Debug, Clone)]
pub struct PersonaManager {
    personas: Vec<String>,
    current_index: usize,
}

impl PersonaManager {
    pub fn new() -> Self {
        Self {
            personas: vec![
                "Balanced/neutral".to_string(),
                "Conservative".to_string(),
                "Optimist/Maximalist".to_string(),
                "Contrarian/Outlier".to_string(),
                "Pessimist/Minimalist".to_string(),
            ],
            current_index: 0,
        }
    }

    pub fn get_current_and_rotate(&mut self) -> &str {
        let current = &self.personas[self.current_index];
        self.current_index = (self.current_index + 1) % self.personas.len();
        current
    }

    pub fn get_current(&self) -> &str {
        &self.personas[self.current_index]
    }
}

#[derive(Debug, Clone)]
pub struct WordFrequencyTracker {
    word_counts: HashMap<String, HashMap<String, i64>>,
    phrase_counts: HashMap<String, HashMap<String, i64>>,
}

impl WordFrequencyTracker {
    pub fn new() -> Self {
        Self {
            word_counts: HashMap::new(),
            phrase_counts: HashMap::new(),
        }
    }

    pub fn update_word_frequency(&mut self, column_name: &str, text: &str, excluded_keys: &[String]) {
        let words = self.extract_words(text, excluded_keys);

        let column_counts = self
            .word_counts
            .entry(column_name.to_string())
            .or_insert_with(HashMap::new);

        for word in words {
            *column_counts.entry(word).or_insert(0) += 1;
        }
    }

    pub fn get_top_words_to_avoid(&self, column_name: &str) -> Vec<String> {
        if let Some(column_counts) = self.word_counts.get(column_name) {
            let mut word_freq_pairs: Vec<_> = column_counts.iter().collect();
            word_freq_pairs.sort_by(|a, b| b.1.cmp(a.1));

            word_freq_pairs
                .into_iter()
                .take(10)
                .map(|(word, _)| word.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn reset_word_counts(&mut self) {
        self.word_counts.clear();
        self.phrase_counts.clear();
    }

    pub fn extract_words(&self, text: &str, excluded_keys: &[String]) -> Vec<String> {
        let stop_words = get_stop_words();
        let excluded_set: HashSet<&str> = excluded_keys.iter().map(|s| s.as_str()).collect();

        text.split_whitespace()
            .map(|word| {
                word.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|word| {
                !word.is_empty()
                    && word.chars().all(|c| c.is_alphabetic())
                    && !stop_words.contains(word)
                    && !excluded_set.contains(word.as_str())
            })
            .collect()
    }

    pub fn extract_phrases(&self, text: &str, n_gram_size: usize) -> Vec<String> {
        let words: Vec<String> = text
            .split_whitespace()
            .map(|w| {
                w.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|w| !w.is_empty())
            .collect();

        if words.len() < n_gram_size {
            return Vec::new();
        }

        words.windows(n_gram_size).map(|window| window.join(" ")).collect()
    }

    pub fn update_phrase_frequency(&mut self, column_name: &str, text: &str) {
        let phrases = self.extract_phrases(text, 3);

        let column_counts = self
            .phrase_counts
            .entry(column_name.to_string())
            .or_insert_with(HashMap::new);

        for phrase in phrases {
            *column_counts.entry(phrase).or_insert(0) += 1;
        }
    }

    pub fn get_top_phrases_to_avoid(&self, column_name: &str) -> Vec<String> {
        if let Some(column_counts) = self.phrase_counts.get(column_name) {
            let mut phrase_freq_pairs: Vec<_> = column_counts.iter().collect();
            phrase_freq_pairs.sort_by(|a, b| b.1.cmp(a.1));

            phrase_freq_pairs
                .into_iter()
                .filter(|(_, count)| **count >= 2)
                .take(5)
                .map(|(phrase, _)| phrase.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn extract_json_structure_keys(&self, column_type_details: &str) -> Vec<String> {
        if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(column_type_details) {
            self.flatten_json_keys(&json_value)
        } else {
            self.extract_keys_with_regex(column_type_details)
        }
    }

    pub fn flatten_json_keys(&self, value: &serde_json::Value) -> Vec<String> {
        let mut keys = Vec::new();

        match value {
            serde_json::Value::Object(map) => {
                for (key, val) in map {
                    keys.push(key.to_lowercase());
                    keys.extend(self.flatten_json_keys(val));
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    keys.extend(self.flatten_json_keys(item));
                }
            }
            _ => {}
        }

        keys
    }

    pub fn extract_keys_with_regex(&self, text: &str) -> Vec<String> {
        let key_pattern = r#""([^"]+)":\s*"#;
        let regex = Regex::new(key_pattern).unwrap_or_else(|_| Regex::new(r#""([^"]+)""#).unwrap());

        regex
            .captures_iter(text)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().to_lowercase())
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationProgress {
    pub total_rows_to_generate: i64,
    pub rows_generated: Vec<DraftRow>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationProgressCallback {
    pub id: Option<i64>,
    pub total_rows_to_generate: i64,
    pub remaining_rows_to_generate: i64,
    pub row: Row,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRow {
    pub data: Vec<RowData>,
}

#[derive(Clone)]
pub struct GenerationService {
    pub db: DatabaseService,
    pub dataset_service: DatasetService,
    pub model_service: ModelService,
    pub llama_backend: Arc<LlamaBackend>,
    model_cache: Arc<Mutex<HashMap<PathBuf, Arc<LlamaModel>>>>,
    active_generations: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

const MAX_CACHED_MODELS: usize = 2;

impl GenerationService {
    pub fn new(
        db: DatabaseService,
        dataset_service: DatasetService,
        model_service: ModelService,
    ) -> Result<Self, AppError> {
        let llama_backend = LlamaBackend::init().map_err(|e| AppError::Io(e.to_string()))?;

        // Note: Logging is enabled for debugging. Uncomment the line below to disable llama.cpp logs:
        // llama_backend.void_logs();

        Ok(Self {
            db,
            dataset_service,
            model_service,
            llama_backend: Arc::new(llama_backend),
            model_cache: Arc::new(Mutex::new(HashMap::new())),
            active_generations: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn register_generation(&self, generation_id: &str, cancel_token: CancellationToken) {
        self.active_generations
            .lock()
            .unwrap()
            .insert(generation_id.to_string(), cancel_token);
    }

    pub fn unregister_generation(&self, generation_id: &str) {
        self.active_generations.lock().unwrap().remove(generation_id);
    }

    pub fn cancel_generation(&self, generation_id: &str) -> Result<(), GenerationError> {
        let active_generations = self.active_generations.lock().unwrap();

        if let Some(cancel_token) = active_generations.get(generation_id) {
            cancel_token.cancel();
            Ok(())
        } else {
            Err(GenerationError::DatabaseError(format!(
                "Generation {} not found or already completed",
                generation_id
            )))
        }
    }

    pub fn clear_model_cache(&self) -> Result<(), GenerationError> {
        let mut cache = self
            .model_cache
            .lock()
            .map_err(|e| GenerationError::ModelError(format!("Failed to lock model cache: {}", e)))?;

        cache.clear();
        Ok(())
    }

    pub fn generate(
        &self,
        dataset_id: i64,
        model_id: i64,
        total_rows_to_generate: i64,
        gpu_layers: u32,
        cancel_token: CancellationToken,
        progress_callback: impl Fn(Vec<RowData>, i64, i64) + Send + 'static,
    ) -> Result<(), GenerationError> {
        let columns = self
            .dataset_service
            .get_columns(dataset_id)
            .map_err(|e| GenerationError::DatabaseError(e.to_string()))?;
        let model_info = self
            .model_service
            .get_model_info(model_id)
            .map_err(|e| GenerationError::DatabaseError(e.to_string()))?;
        let sorted_columns = self
            .sort_columns_by_dependency(&columns, r"@(\w+)")
            .expect("Failed to sort columns");

        let params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers);
        let model_path = self.model_service.models_dir.join(model_info.filename.clone());

        let mut word_tracker = WordFrequencyTracker::new();
        let mut persona_manager = PersonaManager::new();

        for row_index in 0..total_rows_to_generate {
            if cancel_token.is_cancelled() {
                return Err(GenerationError::DatabaseError(
                    "Generation cancelled by user".to_string(),
                ));
            }

            let row_data = self.generate_row(
                &params,
                &model_path,
                &sorted_columns,
                &mut word_tracker,
                &mut persona_manager,
            )?;

            progress_callback(row_data, row_index + 1, total_rows_to_generate);

            if (row_index + 1) % 20 == 0 {
                word_tracker.reset_word_counts();
            }
        }

        Ok(())
    }

    pub fn generate_row(
        &self,
        params: &LlamaModelParams,
        model_path: &PathBuf,
        columns: &[Column],
        word_tracker: &mut WordFrequencyTracker,
        persona_manager: &mut PersonaManager,
    ) -> Result<Vec<RowData>, GenerationError> {
        if columns.is_empty() {
            return Ok(Vec::new());
        }

        let mut data: Vec<RowData> = Vec::new();

        let current_persona = persona_manager.get_current_and_rotate();
        eprintln!("current_persona: {:?}", current_persona);

        let model = self.get_or_load_model(model_path, params)?;
        let config = InferenceConfig::default();

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(config.context_size))
            .with_n_batch(config.batch_size as u32)
            .with_n_ubatch(config.batch_size as u32);

        let mut ctx = model.new_context(&*self.llama_backend, ctx_params)?;

        for column in columns {
            let prompt = self.prepare_prompt(columns, column, &data, word_tracker, &current_persona)?;

            let excluded_keys = if column.column_type == "JSON" {
                column
                    .column_type_details
                    .as_ref()
                    .map(|details| word_tracker.extract_json_structure_keys(details))
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            if column.column_type == "TEXT" {
                let value = self.generate_text(&model, &mut ctx, &prompt, &config)?;

                let row_data: RowData = RowData {
                    column_id: column.id.expect("Column should have an ID").to_string(),
                    value: value.clone(),
                };

                word_tracker.update_word_frequency(&column.name, &value, &excluded_keys);
                word_tracker.update_phrase_frequency(&column.name, &value);
                data.push(row_data);
            }

            if column.column_type == "INT" {
                let value = self.generate_integer(&model, &mut ctx, &prompt, &config)?;

                let row_data: RowData = RowData {
                    column_id: column.id.expect("Column should have an ID").to_string(),
                    value: value.to_string(),
                };
                data.push(row_data);
            }

            if column.column_type == "FLOAT" {
                let value = self.generate_float(&model, &mut ctx, &prompt, &config)?;

                let row_data: RowData = RowData {
                    column_id: column.id.expect("Column should have an ID").to_string(),
                    value: value.to_string(),
                };

                data.push(row_data);
            }

            if column.column_type == "BOOL" {
                let value = self.generate_bool(&model, &mut ctx, &prompt, &config)?;
                let row_data: RowData = RowData {
                    column_id: column.id.expect("Column should have an ID").to_string(),
                    value: value.to_string(),
                };

                data.push(row_data);
            }

            if column.column_type == "JSON" {
                let value = self.generate_json(&model, &mut ctx, &prompt, &config)?;
                let row_data: RowData = RowData {
                    column_id: column.id.expect("Column should have an ID").to_string(),
                    value: value.to_string(),
                };

                word_tracker.update_word_frequency(&column.name, &value.to_string(), &excluded_keys);
                word_tracker.update_phrase_frequency(&column.name, &value.to_string());
                data.push(row_data);
            }
        }

        Ok(data)
    }

    fn generate_text(
        &self,
        model: &LlamaModel,
        ctx: &mut llama_cpp_2::context::LlamaContext,
        prompt: &str,
        config: &InferenceConfig,
    ) -> Result<String, GenerationError> {
        let response = self.inference(model, ctx, prompt, config, None::<fn(&str)>)?;
        let cleaned = Self::clean_text_artifacts(&response);
        Ok(cleaned)
    }

    fn generate_integer(
        &self,
        model: &LlamaModel,
        ctx: &mut llama_cpp_2::context::LlamaContext,
        prompt: &str,
        config: &InferenceConfig,
    ) -> Result<i64, GenerationError> {
        let response = self.inference(model, ctx, prompt, config, None::<fn(&str)>)?;

        let mut numeric_part = String::new();
        for c in response.chars() {
            if c.is_numeric() || c == '.' || c == '-' || c == '+' {
                numeric_part.push(c);
            } else if !numeric_part.is_empty() {
                break;
            }
        }
        eprintln!("response: {:?}", response);

        Ok(numeric_part.parse::<f64>().ok().map(|n| n.round() as i64).unwrap_or(0))
    }

    fn generate_float(
        &self,
        model: &LlamaModel,
        ctx: &mut llama_cpp_2::context::LlamaContext,
        prompt: &str,
        config: &InferenceConfig,
    ) -> Result<f64, GenerationError> {
        let response = self.inference(model, ctx, prompt, config, None::<fn(&str)>)?;

        let mut numeric_part = String::new();
        for c in response.chars() {
            if c.is_numeric() || c == '.' || c == '-' || c == '+' {
                numeric_part.push(c);
            } else if !numeric_part.is_empty() {
                break;
            }
        }

        Ok(numeric_part.parse::<f64>().unwrap_or(0.0))
    }

    fn generate_json(
        &self,
        model: &LlamaModel,
        ctx: &mut llama_cpp_2::context::LlamaContext,
        prompt: &str,
        config: &InferenceConfig,
    ) -> Result<Value, GenerationError> {
        let response = self.inference(model, ctx, prompt, config, None::<fn(&str)>)?;

        let mut cleaned = response
            .trim()
            .replace("```json", "")
            .replace("```", "")
            .trim()
            .to_string();

        if let Some(start) = cleaned.find(|c| c == '{' || c == '[') {
            let first_char = cleaned.chars().nth(start).unwrap();
            let last_char = if first_char == '{' { '}' } else { ']' };

            let slice_after_start = &cleaned[start..];
            let mut extracted = if let Some(rel_end) = slice_after_start.rfind(last_char) {
                slice_after_start[..=rel_end].to_string()
            } else {
                slice_after_start.to_string()
            };

            let mut balance: i32 = 0;
            for ch in extracted.chars() {
                if ch == first_char {
                    balance += 1;
                } else if ch == last_char {
                    balance -= 1;
                }
            }

            if balance > 0 {
                for _ in 0..balance {
                    extracted.push(last_char);
                }
            } else if balance < 0 {
                for _ in 0..(-balance) {
                    extracted.insert(0, first_char);
                }
            }

            cleaned = extracted;
        }

        eprintln!("cleaned: {:?}", cleaned);
        Ok(json5::from_str(&cleaned)?)
    }

    fn generate_bool(
        &self,
        model: &LlamaModel,
        ctx: &mut llama_cpp_2::context::LlamaContext,
        prompt: &str,
        config: &InferenceConfig,
    ) -> Result<bool, GenerationError> {
        let response = self.inference(model, ctx, prompt, config, None::<fn(&str)>)?;
        Ok(response.parse::<bool>().unwrap_or(false))
    }

    pub fn get_or_load_model(
        &self,
        model_path: &PathBuf,
        params: &LlamaModelParams,
    ) -> Result<Arc<LlamaModel>, GenerationError> {
        let mut cache = self
            .model_cache
            .lock()
            .map_err(|e| GenerationError::ModelError(format!("Failed to lock model cache: {}", e)))?;

        if let Some(model) = cache.get(model_path) {
            return Ok(Arc::clone(model));
        }

        if cache.len() >= MAX_CACHED_MODELS {
            if let Some(key) = cache.keys().next().cloned() {
                cache.remove(&key);
            }
        }

        let model = LlamaModel::load_from_file(&*self.llama_backend, model_path, params)?;
        let model_arc = Arc::new(model);
        cache.insert(model_path.clone(), Arc::clone(&model_arc));

        Ok(model_arc)
    }

    pub fn inference(
        &self,
        model: &LlamaModel,
        ctx: &mut llama_cpp_2::context::LlamaContext,
        prompt: &str,
        config: &InferenceConfig,
        token_callback: Option<impl Fn(&str)>,
    ) -> Result<String, GenerationError> {
        ctx.clear_kv_cache();

        let add_bos = if config.add_bos { AddBos::Always } else { AddBos::Never };
        let tokens = model.str_to_token(prompt, add_bos)?;

        let mut batch = LlamaBatch::new(config.batch_size, 1);

        let last_idx = tokens.len().saturating_sub(1);
        for (i, token) in tokens.iter().enumerate() {
            let is_last = i == last_idx;
            batch.add(*token, i as i32, &[0], is_last)?;
        }

        ctx.decode(&mut batch)?;

        let mut response = String::with_capacity(256);
        let mut tokens_generated = 0;
        let mut current_pos = tokens.len() as i32;

        let mut repetition_count = 0;
        let mut last_tokens: VecDeque<LlamaToken> = VecDeque::with_capacity(10);

        loop {
            let logits_iter = ctx.candidates_ith(batch.n_tokens() - 1);

            let candidates: Vec<_> = if config.top_k > 0 {
                let mut top_candidates = Vec::with_capacity(config.top_k as usize);
                for candidate in logits_iter {
                    if top_candidates.len() < config.top_k as usize {
                        top_candidates.push(candidate);
                        if top_candidates.len() == config.top_k as usize {
                            top_candidates.sort_unstable_by(|a, b| {
                                b.logit().partial_cmp(&a.logit()).unwrap_or(std::cmp::Ordering::Equal)
                            });
                        }
                    } else if candidate.logit() > top_candidates.last().unwrap().logit() {
                        top_candidates.pop();
                        top_candidates.push(candidate);
                        top_candidates.sort_unstable_by(|a, b| {
                            b.logit().partial_cmp(&a.logit()).unwrap_or(std::cmp::Ordering::Equal)
                        });
                    }
                }
                top_candidates
            } else {
                let mut all_candidates: Vec<_> = logits_iter.collect();
                all_candidates
                    .sort_unstable_by(|a, b| b.logit().partial_cmp(&a.logit()).unwrap_or(std::cmp::Ordering::Equal));
                all_candidates
            };

            if candidates.is_empty() {
                break;
            }

            let next_token = candidates[0].id();

            if next_token == model.token_eos() {
                break;
            }

            if last_tokens.len() >= 10 && last_tokens.iter().all(|t| *t == next_token) {
                repetition_count += 1;
                if repetition_count > 3 {
                    break;
                }
            } else {
                repetition_count = 0;
            }

            last_tokens.push_back(next_token);
            if last_tokens.len() > 10 {
                last_tokens.pop_front();
            }

            tokens_generated += 1;
            if tokens_generated >= config.max_tokens {
                break;
            }

            let token_str = model.token_to_str(next_token, Special::Plaintext)?;
            response.push_str(&token_str);

            if tokens_generated > 10 {
                let trimmed = response.trim();

                if trimmed.ends_with(".") || trimmed.ends_with("!") || trimmed.ends_with("?") {
                    break;
                }

                if trimmed.contains("\n") {
                    break;
                }

                if response.len() > 200 {
                    break;
                }
            }

            if let Some(ref callback) = token_callback {
                callback(&token_str);
            }

            batch.clear();
            batch.add(next_token, current_pos, &[0], true)?;
            current_pos += 1;

            ctx.decode(&mut batch)?;
        }

        Ok(response)
    }

    pub fn prepare_prompt(
        &self,
        columns: &[Column],
        for_column: &Column,
        row_data: &Vec<RowData>,
        word_tracker: &WordFrequencyTracker,
        persona: &str,
    ) -> Result<String, GenerationError> {
        let column_values: HashMap<String, &str> = row_data
            .iter()
            .map(|row| (row.column_id.clone(), row.value.as_str()))
            .collect();

        let name_to_value: HashMap<String, &str> = columns
            .iter()
            .filter_map(|col| {
                col.id.and_then(|id| {
                    column_values
                        .get(&id.to_string())
                        .map(|&value| (col.name.clone(), value))
                })
            })
            .collect();

        let regex = get_column_ref_regex();
        let processed_rules = regex.replace_all(&for_column.rules, |caps: &regex::Captures| {
            if let Some(column_name) = caps.get(1) {
                name_to_value.get(column_name.as_str()).unwrap_or(&"").to_string()
            } else {
                "".to_string()
            }
        });

        let words_to_avoid = word_tracker.get_top_words_to_avoid(&for_column.name);
        let phrases_to_avoid = word_tracker.get_top_phrases_to_avoid(&for_column.name);

        let words_to_avoid_text = if words_to_avoid.is_empty() && phrases_to_avoid.is_empty() {
            String::new()
        } else {
            let mut avoid_text = String::from("\n\nFor diversity, avoid these:");
            if !phrases_to_avoid.is_empty() {
                avoid_text.push_str(&format!("\n- Phrases: {}", phrases_to_avoid.join(", ")));
            }
            if !words_to_avoid.is_empty() {
                avoid_text.push_str(&format!("\n- Words: {}", words_to_avoid.join(", ")));
            }
            avoid_text
        };

        let prompt = if for_column.column_type != "JSON" {
            CELL_PROMPT_TEMPLATE
                .replace("{persona}", persona)
                .replace("{column_name}", &for_column.name)
                .replace("{column_rule}", &processed_rules)
                .replace("{format}", &for_column.column_type)
                .replace("{words_to_avoid}", &words_to_avoid_text)
                .replace("{return}", "Value")
        } else {
            CELL_PROMPT_TEMPLATE
                .replace("{persona}", persona)
                .replace("{column_name}", &for_column.name)
                .replace("{column_rule}", &processed_rules)
                .replace(
                    "{format}",
                    format!(
                        "well formatted {} structure, structure details: {}",
                        for_column.column_type,
                        for_column.column_type_details.clone().unwrap_or("".to_string())
                    )
                    .as_str(),
                )
                .replace("{words_to_avoid}", &words_to_avoid_text)
                .replace("{return}", "Response (JSON only, no other text)")
        };

        Ok(prompt)
    }

    pub fn sort_columns_by_dependency(&self, columns: &[Column], pattern: &str) -> Result<Vec<Column>, String> {
        if columns.is_empty() {
            return Ok(Vec::new());
        }

        let regex = Regex::new(pattern).map_err(|e| format!("Failed to compile regex pattern '{}': {}", pattern, e))?;

        let name_to_index: HashMap<&str, usize> = columns
            .iter()
            .enumerate()
            .map(|(i, col)| (col.name.as_str(), i))
            .collect();

        let mut reverse_deps: HashMap<usize, Vec<usize>> = HashMap::new();
        let mut in_degree = vec![0; columns.len()];

        for (i, column) in columns.iter().enumerate() {
            for cap in regex.captures_iter(&column.rules) {
                if let Some(dep_name) = cap.get(1) {
                    let dep_name = dep_name.as_str();

                    if let Some(&dep_index) = name_to_index.get(dep_name) {
                        if dep_index != i {
                            reverse_deps.entry(dep_index).or_insert_with(Vec::new).push(i);
                            in_degree[i] += 1;
                        }
                    }
                }
            }
        }

        let mut queue: VecDeque<usize> = in_degree
            .iter()
            .enumerate()
            .filter_map(|(i, &degree)| if degree == 0 { Some(i) } else { None })
            .collect();

        let mut sorted_indices = Vec::with_capacity(columns.len());

        while let Some(current_index) = queue.pop_front() {
            sorted_indices.push(current_index);

            if let Some(dependents) = reverse_deps.get(&current_index) {
                for &dependent_idx in dependents {
                    in_degree[dependent_idx] -= 1;
                    if in_degree[dependent_idx] == 0 {
                        queue.push_back(dependent_idx);
                    }
                }
            }
        }

        if sorted_indices.len() != columns.len() {
            return Err("Circular dependency detected in column rules".to_string());
        }

        Ok(sorted_indices.into_iter().map(|i| columns[i].clone()).collect())
    }

    fn clean_text_artifacts(text: &str) -> String {
        let mut cleaned = text.trim().to_string();

        loop {
            let before = cleaned.clone();

            cleaned = cleaned.trim_end_matches("```").trim_end().to_string();

            cleaned = cleaned.trim_end_matches("\\\"").trim_end().to_string();

            cleaned = cleaned.trim_end_matches("\\n").trim_end().to_string();
            cleaned = cleaned.trim_end_matches('\n').trim_end().to_string();
            cleaned = cleaned.trim_end_matches("\\r").trim_end().to_string();
            cleaned = cleaned.trim_end_matches('\r').trim_end().to_string();

            if before == cleaned {
                break;
            }
        }

        cleaned = cleaned.trim_start_matches("```").trim_start().to_string();
        cleaned = cleaned.trim_start_matches("\\\"").trim_start().to_string();

        if (cleaned.starts_with('"') && cleaned.ends_with('"'))
            || (cleaned.starts_with('\'') && cleaned.ends_with('\''))
        {
            if cleaned.len() > 1 {
                cleaned = cleaned[1..cleaned.len() - 1].to_string();
            }
        } else if cleaned.starts_with('"') || cleaned.starts_with('\'') {
            cleaned = cleaned[1..].to_string();
        }

        cleaned.trim().to_string()
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    mod prompt_manager {
        use super::*;

        #[test]
        fn test_new_prompt_manager() {
            let prompt_manager = PersonaManager::new();
            assert_eq!(prompt_manager.personas.len(), 5);
            assert_eq!(prompt_manager.current_index, 0);
        }

        #[test]
        fn test_get_current_and_rotate() {
            let mut prompt_manager = PersonaManager::new();
            assert_eq!(prompt_manager.get_current_and_rotate(), "Balanced/neutral");
            assert_eq!(prompt_manager.get_current_and_rotate(), "Conservative");
            assert_eq!(prompt_manager.get_current_and_rotate(), "Optimist/Maximalist");
        }

        #[test]
        fn test_get_current() {
            let mut prompt_manager = PersonaManager::new();
            assert_eq!(prompt_manager.get_current_and_rotate(), "Balanced/neutral");
            assert_eq!(prompt_manager.get_current(), "Conservative");
        }
    }

    mod word_frequency_tracker {
        use super::*;

        #[test]
        fn test_new_word_frequency_tracker() {
            let word_frequency_tracker = WordFrequencyTracker::new();
            assert_eq!(word_frequency_tracker.word_counts.len(), 0);
        }

        #[test]
        fn test_update_word_frequency_and_get_top_words_to_avoid() {
            let mut word_frequency_tracker = WordFrequencyTracker::new();
            word_frequency_tracker.update_word_frequency("column", "bicycle", &[]);
            assert_eq!(word_frequency_tracker.get_top_words_to_avoid("column"), vec!["bicycle"]);
        }

        #[test]
        fn test_reset_word_counts() {
            let mut word_frequency_tracker = WordFrequencyTracker::new();
            word_frequency_tracker.update_word_frequency("column", "bicycle", &[]);
            assert_eq!(word_frequency_tracker.word_counts.len(), 1);

            word_frequency_tracker.reset_word_counts();
            assert_eq!(word_frequency_tracker.word_counts.len(), 0);
        }

        #[test]
        fn test_extract_words() {
            let word_frequency_tracker = WordFrequencyTracker::new();

            let result = word_frequency_tracker.extract_words("bicycle", &[]);
            assert_eq!(result, vec!["bicycle"]);

            let result = word_frequency_tracker.extract_words("the bicycle is red", &[]);
            assert_eq!(result, vec!["bicycle", "red"]);

            let result = word_frequency_tracker.extract_words("bicycle motorcycle", &["motorcycle".to_string()]);
            assert_eq!(result, vec!["bicycle"]);

            let result = word_frequency_tracker.extract_words("bicycle, motorcycle!", &[]);
            assert_eq!(result, vec!["bicycle", "motorcycle"]);

            let result = word_frequency_tracker.extract_words("bicycle123 @#$ motorcycle", &[]);
            assert_eq!(result, vec!["motorcycle"]);
        }

        #[test]
        fn test_extract_json_structure_keys() {
            let word_frequency_tracker = WordFrequencyTracker::new();
            let json_structure: &str =
                r#"{ "key1": "test", "key2": { "key3": {"nested_key1": "test", "nested_key2": "test" } }} }"#;

            assert_eq!(
                word_frequency_tracker.extract_json_structure_keys(json_structure),
                vec!["key1", "key2", "key3", "nested_key1", "nested_key2"]
            );
        }

        #[test]
        fn test_flatten_json_keys() {
            let word_frequency_tracker = WordFrequencyTracker::new();
            let json_structure: &str =
                r#"{ "key1": "test", "key2": { "key3": {"nested_key1": "test", "nested_key2": "test" } } }"#;

            assert_eq!(
                word_frequency_tracker.flatten_json_keys(&json5::from_str(json_structure).unwrap()),
                vec!["key1", "key2", "key3", "nested_key1", "nested_key2"]
            );
        }

        #[test]
        fn test_extract_keys_with_regex() {
            let word_frequency_tracker = WordFrequencyTracker::new();
            let json_structure: &str =
                r#"{ "key1": "test", "key2": { "key3": {"nested_key1": "test", "nested_key2": "test" } }} }"#;

            assert_eq!(
                word_frequency_tracker.extract_keys_with_regex(json_structure),
                vec!["key1", "key2", "key3", "nested_key1", "nested_key2"]
            );
        }
    }

    mod generation_service {
        use super::*;
        use std::path::PathBuf;
        use std::sync::Once;

        static INIT: Once = Once::new();

        fn setup_test_environment() {
            INIT.call_once(|| {
                let _ = LlamaBackend::init();
            });
        }

        fn create_generation_service() -> Result<GenerationService, AppError> {
            let db = DatabaseService::new(None).expect("Failed to create database");
            let dataset_service = DatasetService::new(db.clone()).expect("Failed to create dataset service");
            let model_service = ModelService::new(None, db.clone()).expect("Failed to create model service");
            GenerationService::new(db, dataset_service, model_service)
        }

        static TEST_SERVICE: std::sync::OnceLock<Option<GenerationService>> = std::sync::OnceLock::new();

        fn get_test_service() -> Option<&'static GenerationService> {
            let result = TEST_SERVICE.get_or_init(|| match create_generation_service() {
                Ok(service) => Some(service),
                Err(_) => None,
            });
            result.as_ref()
        }

        mod creation {
            use super::*;

            #[test]
            fn test_new_generation_service() {
                setup_test_environment();
                if let Some(service) = get_test_service() {
                    assert!(service.model_cache.lock().is_ok());
                } else {
                    println!("Skipping test due to backend initialization failure");
                }
            }

            #[test]
            fn test_generation_service_has_model_cache() {
                setup_test_environment();
                if let Some(generation_service) = get_test_service() {
                    let cache = generation_service.model_cache.lock();
                    assert!(cache.is_ok(), "Model cache should be accessible");
                    assert!(cache.unwrap().is_empty(), "Model cache should start empty");
                } else {
                    println!("Skipping test due to backend initialization failure");
                }
            }
        }

        mod column_sorting {
            use super::*;

            fn create_test_columns() -> Vec<Column> {
                vec![
                    Column {
                        id: Some(1),
                        table_name: "test_table".to_string(),
                        dataset_id: 1,
                        name: "first_name".to_string(),
                        column_type: "TEXT".to_string(),
                        column_type_details: None,
                        rules: "Generate a first name".to_string(),
                        position: 1,
                    },
                    Column {
                        id: Some(2),
                        table_name: "test_table".to_string(),
                        dataset_id: 1,
                        name: "last_name".to_string(),
                        column_type: "TEXT".to_string(),
                        column_type_details: None,
                        rules: "Generate a last name".to_string(),
                        position: 2,
                    },
                    Column {
                        id: Some(3),
                        table_name: "test_table".to_string(),
                        dataset_id: 1,
                        name: "full_name".to_string(),
                        column_type: "TEXT".to_string(),
                        column_type_details: None,
                        rules: "Generate full name using @first_name and @last_name".to_string(),
                        position: 3,
                    },
                ]
            }

            #[test]
            fn test_sort_columns_by_dependency_success() {
                setup_test_environment();
                if let Some(generation_service) = get_test_service() {
                    let columns = create_test_columns();
                    let sorted = generation_service
                        .sort_columns_by_dependency(&columns, r"@(\w+)")
                        .expect("Failed to sort columns");

                    assert_eq!(sorted[0].name, "first_name");
                    assert_eq!(sorted[1].name, "last_name");
                    assert_eq!(sorted[2].name, "full_name");
                } else {
                    println!("Skipping test due to backend initialization failure");
                }
            }

            #[test]
            fn test_sort_columns_by_dependency_circular_dependency() {
                setup_test_environment();
                if let Some(generation_service) = get_test_service() {
                    let columns = vec![
                        Column {
                            id: Some(1),
                            table_name: "test_table".to_string(),
                            dataset_id: 1,
                            name: "column1".to_string(),
                            column_type: "TEXT".to_string(),
                            column_type_details: None,
                            rules: "Depends on @column2".to_string(),
                            position: 1,
                        },
                        Column {
                            id: Some(2),
                            table_name: "test_table".to_string(),
                            dataset_id: 1,
                            name: "column2".to_string(),
                            column_type: "TEXT".to_string(),
                            column_type_details: None,
                            rules: "Depends on @column1".to_string(),
                            position: 2,
                        },
                    ];

                    let result = generation_service.sort_columns_by_dependency(&columns, r"@(\w+)");
                    assert!(result.is_err(), "Should detect circular dependency");
                } else {
                    println!("Skipping test due to backend initialization failure");
                }
            }

            #[test]
            fn test_sort_columns_by_dependency_invalid_regex() {
                setup_test_environment();
                if let Some(generation_service) = get_test_service() {
                    let columns = create_test_columns();
                    let result = generation_service.sort_columns_by_dependency(&columns, r"[invalid");
                    assert!(result.is_err(), "Should fail with invalid regex");
                } else {
                    println!("Skipping test due to backend initialization failure");
                }
            }
        }

        mod prompt_preparation {
            use super::*;

            fn create_test_columns() -> Vec<Column> {
                vec![
                    Column {
                        id: Some(1),
                        table_name: "test_table".to_string(),
                        dataset_id: 1,
                        name: "first_name".to_string(),
                        column_type: "TEXT".to_string(),
                        column_type_details: None,
                        rules: "Generate a first name".to_string(),
                        position: 1,
                    },
                    Column {
                        id: Some(2),
                        table_name: "test_table".to_string(),
                        dataset_id: 1,
                        name: "last_name".to_string(),
                        column_type: "TEXT".to_string(),
                        column_type_details: None,
                        rules: "Generate a last name using @first_name".to_string(),
                        position: 2,
                    },
                ]
            }

            #[test]
            fn test_prepare_prompt_basic() {
                setup_test_environment();
                if let Some(generation_service) = get_test_service() {
                    let columns = create_test_columns();
                    let row_data = vec![RowData {
                        column_id: "1".to_string(),
                        value: "John".to_string(),
                    }];
                    let word_tracker = WordFrequencyTracker::new();
                    let persona = "Test Persona";

                    let prompt = generation_service
                        .prepare_prompt(&columns, &columns[1], &row_data, &word_tracker, persona)
                        .expect("Failed to prepare prompt");

                    assert!(prompt.contains("Test Persona"));
                    assert!(prompt.contains("last_name"));
                    assert!(prompt.contains("John"));
                    assert!(prompt.contains("TEXT"));
                } else {
                    println!("Skipping test due to backend initialization failure");
                }
            }

            #[test]
            fn test_prepare_prompt_with_json_column() {
                setup_test_environment();
                if let Some(generation_service) = get_test_service() {
                    let columns = vec![Column {
                        id: Some(1),
                        table_name: "test_table".to_string(),
                        dataset_id: 1,
                        name: "user_data".to_string(),
                        column_type: "JSON".to_string(),
                        column_type_details: Some(r#"{"name": "string", "age": "number"}"#.to_string()),
                        rules: "Generate user data".to_string(),
                        position: 1,
                    }];
                    let row_data = vec![];
                    let word_tracker = WordFrequencyTracker::new();
                    let persona = "Test Persona";

                    let prompt = generation_service
                        .prepare_prompt(&columns, &columns[0], &row_data, &word_tracker, persona)
                        .expect("Failed to prepare prompt");

                    assert!(prompt.contains("JSON"));
                    assert!(prompt.contains("structure details"));
                    assert!(prompt.contains(r#"{"name": "string", "age": "number"}"#));
                } else {
                    println!("Skipping test due to backend initialization failure");
                }
            }

            #[test]
            fn test_prepare_prompt_with_words_to_avoid() {
                setup_test_environment();
                if let Some(generation_service) = get_test_service() {
                    let columns = create_test_columns();
                    let row_data = vec![];
                    let mut word_tracker = WordFrequencyTracker::new();
                    word_tracker.update_word_frequency("first_name", "John", &[]);
                    let persona = "Test Persona";

                    let prompt = generation_service
                        .prepare_prompt(&columns, &columns[0], &row_data, &word_tracker, persona)
                        .expect("Failed to prepare prompt");

                    assert!(prompt.contains("For diversity, avoid these:"), "Prompt should contain 'For diversity, avoid these:'");
                    assert!(prompt.contains("- Words:"), "Prompt should contain '- Words:'");
                    assert!(prompt.contains("john"), "Prompt should contain 'john'");
                } else {
                    println!("Skipping test due to backend initialization failure");
                }
            }

            #[test]
            fn test_prepare_prompt_without_words_to_avoid() {
                setup_test_environment();
                if let Some(generation_service) = get_test_service() {
                    let columns = create_test_columns();
                    let row_data = vec![];
                    let word_tracker = WordFrequencyTracker::new();
                    let persona = "Test Persona";

                    let prompt = generation_service
                        .prepare_prompt(&columns, &columns[0], &row_data, &word_tracker, persona)
                        .expect("Failed to prepare prompt");

                    assert!(!prompt.contains("Words to avoid"));
                } else {
                    println!("Skipping test due to backend initialization failure");
                }
            }
        }

        mod generate_row_logic {
            use super::*;

            #[test]
            fn test_generate_row_with_empty_columns() {
                setup_test_environment();
                if let Some(generation_service) = get_test_service() {
                    let params = LlamaModelParams::default();
                    let model_path = PathBuf::from("nonexistent_model.gguf");
                    let columns = vec![];
                    let mut word_tracker = WordFrequencyTracker::new();
                    let mut persona_manager = PersonaManager::new();

                    let result = generation_service.generate_row(
                        &params,
                        &model_path,
                        &columns,
                        &mut word_tracker,
                        &mut persona_manager,
                    );
                    assert!(result.is_ok(), "Should succeed with empty columns");
                    assert_eq!(result.unwrap().len(), 0, "Should return empty row data");
                } else {
                    println!("Skipping test due to backend initialization failure");
                }
            }
        }

        mod text_cleaning {
            use super::*;

            #[test]
            fn test_clean_text_artifacts_basic() {
                let input = r#"  "The shadows lengthen, a flicker of hope in the digital rain."\n```"#;
                let expected = "The shadows lengthen, a flicker of hope in the digital rain.";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }

            #[test]
            fn test_clean_text_artifacts_preserves_internal_quotes() {
                let input = r#""The shadows lengthen, 'a flicker' of hope in the digital rain.""#;
                let expected = "The shadows lengthen, 'a flicker' of hope in the digital rain.";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }

            #[test]
            fn test_clean_text_artifacts_removes_leading_whitespace() {
                let input = "   Some text here";
                let expected = "Some text here";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }

            #[test]
            fn test_clean_text_artifacts_removes_trailing_backticks() {
                let input = "Some text```";
                let expected = "Some text";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }

            #[test]
            fn test_clean_text_artifacts_removes_leading_backticks() {
                let input = "```Some text";
                let expected = "Some text";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }

            #[test]
            fn test_clean_text_artifacts_removes_escaped_quotes() {
                let input = r#"\"The text here\""#;
                let expected = "The text here";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }

            #[test]
            fn test_clean_text_artifacts_removes_newlines() {
                let input = "Some text\n";
                let expected = "Some text";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }

            #[test]
            fn test_clean_text_artifacts_removes_multiple_artifacts() {
                let input = "  ```\n\"Some text\"\n```  ";
                let expected = "Some text";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }

            #[test]
            fn test_clean_text_artifacts_single_quotes() {
                let input = "'Some text with 'inner' quotes'";
                let expected = "Some text with 'inner' quotes";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }

            #[test]
            fn test_clean_text_artifacts_unmatched_quote() {
                let input = "\n```\n\"You look lost.  Perhaps I";
                let expected = "You look lost.  Perhaps I";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }

            #[test]
            fn test_clean_text_artifacts_unmatched_single_quote() {
                let input = "'You look lost.  Perhaps I";
                let expected = "You look lost.  Perhaps I";
                let result = GenerationService::clean_text_artifacts(input);
                assert_eq!(result, expected);
            }
        }
    }
}
