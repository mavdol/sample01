---
id: mistral-7b-instruct-v0.3
label: Mistral 7B v0.3
models:
  - quantization: Q4_K_M
    size: 4370
    url: https://huggingface.co/MaziyarPanahi/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3.Q4_K_M.gguf
    recommended: true

  - quantization: Q5_K_M
    size: 5140
    url: https://huggingface.co/MaziyarPanahi/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3.Q5_K_M.gguf
    recommended: false

  - quantization: Q6_K
    size: 5950
    url: https://huggingface.co/MaziyarPanahi/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3.Q6_K.gguf
    recommended: false

  - quantization: Q8_0
    size: 7700
    url: https://huggingface.co/MaziyarPanahi/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3.Q8_0.gguf
    recommended: false
---

![mistral](/images/models/mistral.png)

# Mistral 7B Instruct v0.3

Mistral 7B Instruct v0.3 is a 7 billion parameter language model developed by Mistral AI. This instruction-tuned variant is designed for conversational AI and instruction-following tasks, offering excellent performance while remaining efficient enough to run on consumer hardware.

The v0.3 release includes improvements in instruction following and extended vocabulary support. The model excels at various tasks including text generation, question answering, code generation, and reasoning. With an Apache 2.0 license, it's freely available for commercial use.

Mistral 7B has been shown to outperform larger models on several benchmarks, making it an excellent choice for local deployment where you need strong performance without excessive resource requirements.

**Note:** Q4_K_M is recommended for the best balance between quality and resource usage. Higher quantizations (Q6_K, Q8_0) provide marginally better quality at the cost of significantly more disk space and memory.
