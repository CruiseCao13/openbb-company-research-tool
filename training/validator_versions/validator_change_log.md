# Validator Change Log

## wrong_framework_lunr_telecom

- Missed by validator: true
- New rule: provider identity clues for space, lunar, NASA, aerospace, lander, mission, satellite, cislunar, launch, or spacecraft forbid telecom frames and telecom revenue engines unless the provider description explicitly supports telecom carrier activity.
- Expected validator behavior: set `wrong_framework_conflict=true`, `framework_fit_check=FAIL`, and `human_review_required=true`; report status cannot be PASS.
- Tests added:
  - `lunr_not_telecom`
  - `lunr_detects_space_lunar_frame`
  - `lunr_forbids_wireless_broadband_subscriber_churn`
  - `wrong_framework_sets_human_review_required`
  - `wrong_framework_cannot_pass_report_status`
  - `hallucinated_revenue_engines_detected`
- Misfire watch: do not classify actual telecom infrastructure companies as aerospace unless provider identity contains space/aerospace clues.
