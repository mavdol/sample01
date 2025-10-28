---
id: gemma-3-1b-it
label: Gemma3 1B
models:
  - quantization: Q4_K_M
    size: 806
    url: https://huggingface.co/unsloth/gemma-3-1b-it-GGUF/resolve/main/gemma-3-1b-it-Q4_K_M.gguf
    recommended: true

  - quantization: Q5_K_M
    size: 851
    url: https://huggingface.co/unsloth/gemma-3-1b-it-GGUF/resolve/main/gemma-3-1b-it-Q5_K_M.gguf
    recommended: false

  - quantization: Q6_K
    size: 1010
    url: https://huggingface.co/unsloth/gemma-3-1b-it-GGUF/resolve/main/gemma-3-1b-it-Q6_K.gguf
    recommended: false

  - quantization: Q8_0
    size: 1070
    url: https://huggingface.co/unsloth/gemma-3-1b-it-GGUF/resolve/main/gemma-3-1b-it-Q8_0.gguf
    recommended: false
---

![Gemma3](/images/gemma.png)

# Gemma 3 1B

Gemma 3 is a lightweight 1B parameter language model developed by Google. It's part of the Gemma family, designed to be efficient and run on resource-constrained devices.

The model is instruction-tuned (indicated by "it" suffix), making it suitable for chat and task-following applications. While small, with proper quantization (Q4+) it can produce decent results for simple data generation tasks.

**Note:** Only Q4+ quantizations are included. Avoid IQ1/IQ2/Q2/Q3 variants as they produce poor quality output.
