prompt_version: self_review_v1
task: self_review
input_schema: compact locked provider summary and generated v5 artifacts
output_schema: schema-validated JSON for self_review
forbidden_behavior:
  - do not modify locked financial data
  - do not invent segment, clinical, foundry, regulatory, or provider facts
  - do not provide buy/sell advice, target price, or short-term prediction
framework_challenge:
  - check whether the selected research frame matches the company name, sector, industry, and description
  - flag any revenue engine that is not supported by provider data
  - flag forbidden industry terms that conflict with provider identity
  - if provider identity is space/lunar/NASA/aerospace/mission/lander/satellite/cislunar-linked, reject telecom carrier framing
  - if unsupported revenue engines appear, require rewrites for company_understanding, business_model, money_flow, and research_blueprint
  - flag generic money-flow language that could apply to any company
  - flag generic chart explanations that do not mention the figure's specific metric or data gap
failure_handling:
  - mark unsupported claims explicitly
  - request human review when data or framework confidence is low
examples:
  - explain what the artifact is, why it matters, how to read it, what it can prove, what it cannot prove, what is missing, and what to check next
