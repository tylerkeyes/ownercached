# Ownercached

A memcached server implementation, written in rust.

By default, the memcached server starts on port `11211`. This can be changed using the flag `-p` / `--port`.

## Commands

The following commands are supported:

- `set`
- `get`
- `add`
- `replace`
- `append`
- `prepend`

### Command Structure

All commands are in the format:

```sh
<command name> <key> <flags> <exptime> <byte count> [noreply]\r\n
<data block>\r\n
```

The exception is the `get` command, which is in the format:

```sh
get <key>\r\n
```

- `key` => unique key for referencing the saved value.
- `flags` => arbitrary 16-bit unsigned integer that is stored along with the value.
- `exptime` => expiration time of the key.
  - If the value is zero, the item never expires.
  - If the value is value is below zero, the item expires immediately.
  - If the value is above zero, that is the number of seconds into the future the item expires.
- `byte count` => the number of bytes in the `data block`.
