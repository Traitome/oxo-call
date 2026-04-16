---
name: ssh
category: networking
description: OpenSSH secure shell client for encrypted remote login and command execution
tags: [ssh, remote, networking, security, tunnel, key, sftp]
author: oxo-call built-in
source_url: "https://www.openssh.com/manual.html"
---

## Concepts
- SSH connects to a remote host as: 'ssh [options] [user@]hostname [command]'. If user is omitted, the current local username is used. The hostname or IP is required.
- Key-based authentication is preferred over passwords: generate a key pair with 'ssh-keygen', then copy the public key with 'ssh-copy-id user@host' or append to ~/.ssh/authorized_keys on the server.
- SSH config file (~/.ssh/config) lets you define aliases and per-host settings (HostName, User, IdentityFile, Port, etc.) so you can connect with just 'ssh myalias'.
- Port forwarding: '-L local:host:remote' forwards a local port to a remote host; '-R remote:host:local' forwards a remote port back; '-D port' creates a SOCKS proxy.
- Use '-i /path/to/key' to specify a private key file when multiple keys exist. The default keys are ~/.ssh/id_rsa, ~/.ssh/id_ed25519, etc.
- Multiplexing with ControlMaster reduces connection overhead: add 'ControlMaster auto' and 'ControlPath ~/.ssh/cm-%r@%h:%p' to ~/.ssh/config for fast repeated connections.
- ProxyJump (-J) connects through a bastion/jump host: 'ssh -J user@jumphost user@targethost' for accessing internal networks.
- -N prevents remote command execution; use with port forwarding for tunnel-only connections.
- -f backgrounds the SSH process after authentication; useful for long-running tunnels.
- -C enables compression; beneficial on slow networks but adds overhead on fast connections.
- -A enables agent forwarding; allows using local SSH keys on remote hosts (use with caution).
- -T disables pseudo-terminal allocation; useful for scripting and non-interactive commands.

## Pitfalls
- Always include the remote hostname or IP address: 'ssh user@hostname'. Calling 'ssh' without a host will produce a usage error.
- If key authentication is rejected, check permissions: ~/.ssh/ must be 700, ~/.ssh/authorized_keys must be 600, private key files must be 600.
- The -X flag (X11 forwarding) is slow and insecure; prefer -Y (trusted forwarding) only when needed for GUI applications.
- Avoid using -o StrictHostKeyChecking=no in production as it opens you to MITM attacks. Instead, add the host key properly with 'ssh-keyscan host >> ~/.ssh/known_hosts'.
- Passing commands to ssh: 'ssh user@host ls /tmp' runs 'ls /tmp' on the remote host; quote complex commands: ssh user@host 'cmd1 && cmd2'.
- ssh-keygen generates keys in ~/.ssh/ by default. Always set a passphrase for private keys used in production environments.
- Agent forwarding (-A) is a security risk on untrusted hosts; malicious root users can access your agent.
- -f backgrounds SSH but doesn't provide feedback on tunnel status; verify with netstat or ss.
- Multiple -J jump hosts must be comma-separated without spaces: -J host1,host2,host3.
- ControlMaster sockets can persist indefinitely; use ControlPersist with a timeout (e.g., 10m).
- -C compression can actually slow down transfers on fast networks; test with and without.
- SSH config file syntax errors can cause silent failures; validate with 'ssh -G hostname'.

## Examples

### connect to a remote server as a specific user
**Args:** `user@hostname`
**Explanation:** basic SSH connection; replace user with username and hostname with the server IP or domain

### connect using a specific private key file
**Args:** `-i ~/.ssh/id_ed25519 user@hostname`
**Explanation:** -i specifies the identity (private key) file; useful when multiple keys exist

### connect on a non-standard port
**Args:** `-p 2222 user@hostname`
**Explanation:** -p changes the port from the default 22 to 2222

### forward a local port to a remote service (local port forwarding)
**Args:** `-L 8080:localhost:80 user@hostname`
**Explanation:** -L 8080:localhost:80 maps local port 8080 to port 80 on the remote server; useful for accessing remote web services

### run a command on a remote host without an interactive shell
**Args:** `user@hostname 'df -h && free -h'`
**Explanation:** commands after the hostname are executed remotely; quote multi-word commands to pass them as a single argument

### enable X11 forwarding to run graphical applications remotely
**Args:** `-X user@hostname`
**Explanation:** -X enables X11 forwarding; allows running graphical apps on the remote server displayed locally

### connect and set up reverse port forwarding (expose local service to remote)
**Args:** `-R 9090:localhost:3000 user@hostname`
**Explanation:** -R 9090:localhost:3000 makes the remote port 9090 forward to local port 3000; useful for exposing local dev servers

### create a SOCKS5 proxy tunnel through the remote host
**Args:** `-D 1080 -N user@hostname`
**Explanation:** -D 1080 creates a SOCKS5 proxy on local port 1080; -N prevents executing a remote command (tunnel only)

### keep connection alive and reconnect automatically
**Args:** `-o ServerAliveInterval=60 -o ServerAliveCountMax=3 user@hostname`
**Explanation:** ServerAliveInterval sends keepalive packets every 60s; helps maintain idle connections through firewalls

### use jump host (bastion) to reach a machine not directly accessible
**Args:** `-J bastion_user@bastion_host target_user@target_host`
**Explanation:** -J specifies the jump/bastion host used to proxy the final connection

### chain multiple jump hosts for complex networks
**Args:** `-J user1@host1,user2@host2 user3@final_host`
**Explanation:** comma-separated jump hosts; connects through host1, then host2, then to final destination

### run SOCKS proxy in background for browser use
**Args:** `-D 1080 -N -f user@hostname`
**Explanation:** -D 1080 creates SOCKS5 proxy; -N no command; -f backgrounds; configure browser to use localhost:1080

### enable compression for slow network connections
**Args:** `-C -o ServerAliveInterval=60 user@hostname`
**Explanation:** -C enables gzip compression; beneficial on slow networks; combine with keepalive for stability

### forward multiple ports in one connection
**Args:** `-L 8080:localhost:80 -L 3306:db.internal:3306 user@gateway`
**Explanation:** multiple -L flags forward different ports; useful for accessing web and database simultaneously

### disable pseudo-terminal for non-interactive scripts
**Args:** `-T user@hostname 'cat /var/log/app.log | grep ERROR'`
**Explanation:** -T disables PTY allocation; prevents "stdin is not a tty" errors in scripts

### test SSH config without connecting
**Args:** `-G myhost | grep -E "^(hostname|port|user)"`
**Explanation:** -G parses config and prints effective settings; useful for debugging config file issues

### force new connection bypassing multiplexing
**Args:** `-o "ControlMaster=no" user@hostname`
**Explanation:** overrides ControlMaster auto in config; forces fresh connection for troubleshooting

### copy file through jump host using scp
**Args:** `-o ProxyJump=jumphost file.txt target:~/path/`
**Explanation:** scp uses same ProxyJump as ssh; transfers files to hosts behind bastions

### create reverse tunnel exposing local service
**Args:** `-R '*:8080:localhost:3000' -o GatewayPorts=yes user@public_server`
**Explanation:** -R creates reverse tunnel; GatewayPorts=yes allows external access to forwarded port
