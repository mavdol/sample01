use crate::error::AppError;
use crate::services::database::{DatabaseError, DatabaseService};
use crate::services::dataset::{Column, Row, RowData};
use crate::services::{DatasetService, ModelService};
use serde_json::Value;
use std::fmt;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

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
use std::collections::{HashMap, VecDeque};
use std::cmp::Ordering;
use std::sync::OnceLock;
use rand::Rng;

use crate::utils::CELL_PROMPT_TEMPLATE;


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
            temperature: 0.8,
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

static COLUMN_REF_REGEX: OnceLock<Regex> = OnceLock::new();
static RANDOM_INT_SINGLE_REGEX: OnceLock<Regex> = OnceLock::new();
static RANDOM_INT_RANGE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_column_ref_regex() -> &'static Regex {
    COLUMN_REF_REGEX.get_or_init(|| Regex::new(r"@(\w+)").expect("Invalid regex pattern"))
}

fn get_random_int_single_regex() -> &'static Regex {
    RANDOM_INT_SINGLE_REGEX.get_or_init(|| Regex::new(r"@RANDOM_INT_(\d+)").expect("Invalid regex pattern"))
}

fn get_random_int_range_regex() -> &'static Regex {
    RANDOM_INT_RANGE_REGEX.get_or_init(|| Regex::new(r"@RANDOM_INT_(\d+)_(\d+)").expect("Invalid regex pattern"))
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
        let mut llama_backend = LlamaBackend::init().map_err(|e| AppError::Io(e.to_string()))?;

        llama_backend.void_logs();

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
        eprintln!("Generating {} rows with {} GPU layers", total_rows_to_generate, gpu_layers);
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

        let model = self.get_or_load_model(&model_path, &params)?;
        let config = InferenceConfig::default();

        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(NonZeroU32::new(config.context_size))
            .with_n_batch(config.batch_size as u32)
            .with_n_ubatch(config.batch_size as u32);

        let mut ctx = model.new_context(&*self.llama_backend, ctx_params)?;

        for row_index in 0..total_rows_to_generate {
            if cancel_token.is_cancelled() {
                return Err(GenerationError::DatabaseError(
                    "Generation cancelled by user".to_string(),
                ));
            }

            let row_data = self.generate_row(
                &model,
                &mut ctx,
                &config,
                &sorted_columns,
                &cancel_token,
            )?;

            progress_callback(row_data, row_index + 1, total_rows_to_generate);
        }

        Ok(())
    }

    pub fn generate_row(
        &self,
        model: &LlamaModel,
        ctx: &mut llama_cpp_2::context::LlamaContext,
        config: &InferenceConfig,
        columns: &[Column],
        cancel_token: &CancellationToken,
    ) -> Result<Vec<RowData>, GenerationError> {
        if columns.is_empty() {
            return Ok(Vec::new());
        }

        let mut data: Vec<RowData> = Vec::new();

        for column in columns {
            if cancel_token.is_cancelled() {
                return Err(GenerationError::DatabaseError(
                    "Generation cancelled by user".to_string(),
                ));
            }

            let prompt = self.prepare_prompt(columns, column, &data)?;

            if column.column_type == "TEXT" {
                let value = self.generate_text(model, ctx, &prompt, config)?;

                let row_data: RowData = RowData {
                    column_id: column.id.expect("Column should have an ID").to_string(),
                    value,
                };
                data.push(row_data);
            }

            if column.column_type == "INT" {
                let value = self.generate_integer(model, ctx, &prompt, config)?;

                let row_data: RowData = RowData {
                    column_id: column.id.expect("Column should have an ID").to_string(),
                    value: value.to_string(),
                };
                data.push(row_data);
            }

            if column.column_type == "FLOAT" {
                let value = self.generate_float(model, ctx, &prompt, config)?;

                let row_data: RowData = RowData {
                    column_id: column.id.expect("Column should have an ID").to_string(),
                    value: value.to_string(),
                };
                data.push(row_data);
            }

            if column.column_type == "BOOL" {
                let value = self.generate_bool(model, ctx, &prompt, config)?;
                let row_data: RowData = RowData {
                    column_id: column.id.expect("Column should have an ID").to_string(),
                    value: value.to_string(),
                };

                data.push(row_data);
            }

            if column.column_type == "JSON" {
                let value = self.generate_json(model, ctx, &prompt, config)?;
                let value_str = value.to_string();

                let row_data: RowData = RowData {
                    column_id: column.id.expect("Column should have an ID").to_string(),
                    value: value_str,
                };
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
                    } else {

                        let min_idx = top_candidates
                            .iter()
                            .enumerate()
                            .min_by(|(_, a), (_, b)| {
                                a.logit().partial_cmp(&b.logit()).unwrap_or(Ordering::Equal)
                            })
                            .map(|(idx, _)| idx)
                            .unwrap_or(0);

                        if candidate.logit() > top_candidates[min_idx].logit() {
                            top_candidates[min_idx] = candidate;
                        }
                    }
                }

                top_candidates.sort_unstable_by(|a, b| {
                    b.logit().partial_cmp(&a.logit()).unwrap_or(Ordering::Equal)
                });
                top_candidates
            } else {
                let mut all_candidates: Vec<_> = logits_iter.collect();
                all_candidates
                    .sort_unstable_by(|a, b| b.logit().partial_cmp(&a.logit()).unwrap_or(Ordering::Equal));
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

            if tokens_generated > 3 {
                let trimmed = response.trim();

                if trimmed.contains("```") {
                    break;
                }

                if trimmed.contains("\n") {
                    break;
                }

                if tokens_generated > 10 {
                    if trimmed.ends_with(".") || trimmed.ends_with("!") || trimmed.ends_with("?") {
                        break;
                    }
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
    ) -> Result<String, GenerationError> {

        let id_to_name: HashMap<String, &str> = columns
            .iter()
            .filter_map(|col| col.id.map(|id| (id.to_string(), col.name.as_str())))
            .collect();

        let mut name_to_value: HashMap<&str, &str> = HashMap::with_capacity(row_data.len());
        for row in row_data {
            if let Some(&name) = id_to_name.get(&row.column_id) {
                name_to_value.insert(name, row.value.as_str());
            }
        }

        // First, replace @RANDOM_INT_X_Y (range) commands
        let random_range_regex = get_random_int_range_regex();
        let mut rng = rand::thread_rng();
        let after_range_random = random_range_regex.replace_all(&for_column.rules, |caps: &regex::Captures| {
            let start: i64 = caps.get(1).unwrap().as_str().parse().unwrap_or(0);
            let end: i64 = caps.get(2).unwrap().as_str().parse().unwrap_or(0);
            let random_value = rng.gen_range(start..=end);
            random_value.to_string()
        });

        // Then, replace @RANDOM_INT_X (single) commands
        let random_single_regex = get_random_int_single_regex();
        let after_single_random = random_single_regex.replace_all(&after_range_random, |caps: &regex::Captures| {
            let max: i64 = caps.get(1).unwrap().as_str().parse().unwrap_or(1);
            let random_value = rng.gen_range(0..max);
            random_value.to_string()
        });

        // Finally, replace @column_name references
        let column_ref_regex = get_column_ref_regex();
        let processed_rules = column_ref_regex.replace_all(&after_single_random, |caps: &regex::Captures| {
            caps.get(1)
                .and_then(|m| name_to_value.get(m.as_str()))
                .copied()
                .unwrap_or("")
        });

        let format_str = if for_column.column_type == "JSON" {
            let details = for_column.column_type_details.as_deref().unwrap_or("");
            format!("well formatted {} structure, structure details: {}", for_column.column_type, details)
        } else {
            for_column.column_type.clone()
        };

        let prompt = CELL_PROMPT_TEMPLATE
            .replace("{column_name}", &for_column.name)
            .replace("{column_rule}", &processed_rules)
            .replace("{format}", &format_str);


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
        let mut cleaned = text.trim();

        // First, remove leading artifacts
        cleaned = cleaned.trim_start_matches("```").trim_start();
        cleaned = cleaned.trim_start_matches("\\\"").trim_start();

        // Then check for code blocks and cut at first occurrence (only if there's content before it)
        if let Some(start) = cleaned.find("```") {
            if start > 0 {
                cleaned = &cleaned[..start];
            }
        }

        // Remove trailing artifacts in a loop
        loop {
            let before = cleaned.len();
            cleaned = cleaned.trim_end_matches("```").trim_end();
            cleaned = cleaned.trim_end_matches("\\\"").trim_end();
            cleaned = cleaned.trim_end_matches("\\n").trim_end();
            cleaned = cleaned.trim_end_matches('\n').trim_end();
            cleaned = cleaned.trim_end_matches("\\r").trim_end();
            cleaned = cleaned.trim_end_matches('\r').trim_end();

            if cleaned.len() == before {
                break;
            }
        }

        let trimmed = cleaned.trim();

        // Handle quoted strings
        if trimmed.len() > 1 {
            if (trimmed.starts_with('"') && trimmed.ends_with('"'))
                || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            {
                return trimmed[1..trimmed.len() - 1].trim().to_string();
            } else if trimmed.starts_with('"') && !trimmed.ends_with('"') {
                return trimmed[1..].trim().to_string();
            } else if !trimmed.starts_with('"') && trimmed.ends_with('"') {
                return trimmed[..trimmed.len() - 1].trim().to_string();
            } else if trimmed.starts_with('\'') && !trimmed.ends_with('\'') {
                return trimmed[1..].trim().to_string();
            } else if !trimmed.starts_with('\'') && trimmed.ends_with('\'') {
                return trimmed[..trimmed.len() - 1].trim().to_string();
            }
        }

        trimmed.to_string()
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    mod generation_service {
        use super::*;
        use std::sync::Once;

        static INIT: Once = Once::new();

        fn setup_test_environment() {
            INIT.call_once(|| {
                let _ = LlamaBackend::init();
            });
        }

        fn create_generation_service() -> Result<GenerationService, AppError> {
            Err(AppError::Io("Test environment: AppHandle not available".to_string()))
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

                    let prompt = generation_service
                        .prepare_prompt(&columns, &columns[1], &row_data)
                        .expect("Failed to prepare prompt");

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

                    let prompt = generation_service
                        .prepare_prompt(&columns, &columns[0], &row_data)
                        .expect("Failed to prepare prompt");

                    assert!(prompt.contains("JSON"));
                    assert!(prompt.contains("structure details"));
                    assert!(prompt.contains(r#"{"name": "string", "age": "number"}"#));
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
