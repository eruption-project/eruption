# Contributing to Eruption

First off, thanks for taking the time to contribute to Eruption!

## How Can I Contribute

Every contribution counts!

* Please feel free to open up GitHub issues any time
* Feature requests are always welcome

## GitHub Issues

### Reporting a Bug

* Please include as much context as possible, like e.g. log files

## Pull Requests

* Fork the repo and create your branch from `develop`
* Implement your changes
* Make sure your code lints with `cargo clippy`
* Run `cargo fmt`
* Please submit your pull request

### Device Support Contributions

If your change adds support for new hardware, please follow the checklist in
[docs/howtos/NEW_DEVICE_BRINGUP_CHECKLIST.md](docs/howtos/NEW_DEVICE_BRINGUP_CHECKLIST.md)
and include validation notes in the PR.

### Documentation and i18n Checks

Before opening a PR, run:

* `bash support/tests/check-docs.sh`
* `bash support/tests/check-i18n.sh`

Translation process and review checklist:

* [docs/howtos/I18N_TRANSLATION_WORKFLOW.md](docs/howtos/I18N_TRANSLATION_WORKFLOW.md)

### Styleguides

* Use rustfmt

#### Git Commit Messages

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Reference issues and pull requests after the first line

## License

By contributing, you agree that your contributions will be licensed under the GPLv3 License.
