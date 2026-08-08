# Eruption Shell SDK Workflows

This document describes shell-based automation workflows around Eruption.

## Table of Contents

- [Eruption Shell SDK Workflows](#eruption-shell-sdk-workflows)
	- [Table of Contents](#table-of-contents)
	- [Prerequisites](#prerequisites)
	- [Connect and Query Flow](#connect-and-query-flow)
	- [Update Flow](#update-flow)
	- [Error Handling Flow](#error-handling-flow)
	- [Example Scripts](#example-scripts)

## Prerequisites

- `eruptionctl` installed and available in `PATH`
- running Eruption daemon

## Connect and Query Flow

Query current profile, active slot, and global brightness:

```bash
eruptionctl status profile
eruptionctl status slot
eruptionctl config brightness
```

## Update Flow

Switch slot and update global brightness:

```bash
eruptionctl switch slot 2
eruptionctl config brightness 70
```

## Error Handling Flow

Recommended pattern in shell automation:

1. Verify command availability.
2. Handle known invalid input paths (for example, invalid slot index).
3. Recover to a safe state (for example, fallback slot 1).
4. Continue only when daemon checks pass.

## Example Scripts

See runnable scripts in:

- `sdk/examples/shell/connect_and_query.sh`
- `sdk/examples/shell/update_state.sh`
- `sdk/examples/shell/error_handling.sh`
