# license

Verify and display license information.

## Synopsis

```
oxo-call license verify
oxo-call license show
```

## Subcommands

### `license verify`

Check whether the current license is valid:

```bash
oxo-call license verify
```

### `license show`

Display license details (organization, type, expiry):

```bash
oxo-call license show
```

## License Resolution Order

oxo-call searches for a license in this order:

1. `--license` CLI argument
2. `OXO_CALL_LICENSE` environment variable
3. Platform-specific config directory (`license.oxo.json`)

See the [License Setup tutorial](../tutorials/license.md) for complete details.
