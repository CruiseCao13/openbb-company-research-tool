prompt_version: self_review_v1
task: self_review
input_schema: compact locked provider summary and generated v5 artifacts
output_schema: schema-validated JSON for self_review
forbidden_behavior:
  - do not modify locked financial data
  - do not invent segment, clinical, foundry, regulatory, or provider facts
  - do not provide buy/sell advice, target price, or short-term prediction
failure_handling:
  - mark unsupported claims explicitly
  - request human review when data or framework confidence is low
examples:
  - explain what the artifact is, why it matters, how to read it, what it can prove, what it cannot prove, what is missing, and what to check next
