# i18n Translation Workflow

This guide defines the contributor workflow for localization updates.

## Supported Locales

Current baseline locales:

- `en-US` (source locale)
- `de-DE` (priority translated locale)

## Translation Flow

1. Update source keys in `en-US/*.ftl`.
2. Mirror key additions/removals in `de-DE/*.ftl`.
3. Preserve placeholder variables exactly (for example `{ $value }`).
4. Run validation checks before opening PR.

## Review Checklist

- [ ] key parity between `en-US` and `de-DE` files
- [ ] placeholder parity per key
- [ ] no stale or removed keys left behind
- [ ] terminology consistency with existing strings
- [ ] docs updated if user-facing wording changed significantly

## Validation Commands

From repository root:

```bash
bash support/tests/check-i18n.sh
bash support/tests/check-docs.sh
```

## Notes for New Locales

When introducing a new locale:

- clone all keys from `en-US`
- keep placeholders unchanged
- add locale to roadmap status as planned/active
- ensure CI checks are extended for the new locale pair
