prompt_version: company_understanding_v1
task: company_understanding
input_schema: compact locked provider summary and generated v5 artifacts
output_schema: schema-validated JSON for company_understanding
forbidden_behavior:
  - do not modify locked financial data
  - do not invent segment, clinical, foundry, regulatory, or provider facts
  - do not infer telecom unless provider data explicitly supports a carrier model with service plans, subscribers, broadband access, or wireless network revenue
  - if provider identity is space/lunar/NASA/aerospace/mission/lander/satellite/cislunar/launch/spacecraft-linked, do not call it telecom infrastructure
  - do not provide buy/sell advice, target price, or short-term prediction
framework_selection:
  - choose the research frame from provider company name, sector, industry, and description before using generic sector labels
  - list what the company is not before accepting the frame
  - for space or lunar infrastructure companies, use a space/aerospace/project-execution frame or mark data-limited with human review
  - revenue engines must be supported by the provider description or locked data; if inferred, mark the inference and lower confidence
failure_handling:
  - mark unsupported claims explicitly
  - request human review when data or framework confidence is low
examples:
  - explain what the artifact is, why it matters, how to read it, what it can prove, what it cannot prove, what is missing, and what to check next
