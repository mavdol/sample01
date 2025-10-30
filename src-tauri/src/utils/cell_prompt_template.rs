pub static CELL_PROMPT_TEMPLATE: &str = r#"<|begin_of_text|><|start_header_id|>system<|end_header_id|>
You are a data generator. You must respond with ONLY the requested value. No explanations, no code, no markdown, no extra text.<|eot_id|>

<|start_header_id|>user<|end_header_id|>
Generate a {format} value for column "{column_name}".

Rule: {column_rule}

Perspective: {persona}
{words_to_avoid}

CRITICAL: If the rule references other values from the same record (like names, IDs, or previously generated fields), your response MUST be logically consistent with those values. Consistency is MORE IMPORTANT than diversity.

Reply with ONLY the raw value, nothing else.<|eot_id|>

<|start_header_id|>assistant<|end_header_id|>
"#;
