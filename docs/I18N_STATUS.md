# i18n Status

This file tracks localization coverage for currently supported locales.

## Current Locale Scope

- Source locale: `en-US`
- Priority translated locale: `de-DE`

## Coverage Snapshot

- Locale files (`en-US`): 16
- Locale files (`de-DE`): 16
- Pairing gaps: none detected
- Placeholder parity: validated by `support/tests/check-i18n.sh`

## Validation Gates

Run from repository root:

```bash
bash support/tests/check-i18n.sh
bash support/tests/check-docs.sh
```

## Notes

- M2 translation expansion is considered complete for current priority locale scope.
- Adding new locales requires extending validation scope and roadmap status updates.
