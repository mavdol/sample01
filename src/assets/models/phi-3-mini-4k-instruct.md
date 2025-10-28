---
id: phi-3-mini-4k-instruct
label: Phi3 mini
models:
  - quantization: q4
    size: 2930
    url: https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4.gguf
    recommended: true

  - quantization: fp16
    size: 7640
    url: https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-fp16.gguf
    recommended: false
---

![microsoft](/images/microsoft.svg)

# Phi-3 Mini

Phi-3 Mini is a 3.8B parameters, lightweight, state-of-the-art open model trained with the Phi-3 datasets that includes both synthetic data and the filtered publicly available websites data with a focus on high-quality and reasoning dense properties.

The model has underwent a post-training process that incorporates both supervised fine-tuning and direct preference optimization to ensure precise instruction adherence and robust safety measures.

When assessed against benchmarks testing common sense, language understanding, math, code, long context and logical reasoning, Phi-3 Mini-4K-Instruct showcased a robust and state-of-the-art performance among models with less than 13 billion parameters.
