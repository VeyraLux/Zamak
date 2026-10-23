# Test certificate

`cert.pem` and `key.pem` are a self-signed certificate and its private key for `localhost` and `127.0.0.1`. The TLS tests in `tests/net.rs` use them. They're public, so never use them for anything else.

To make a new pair:

```bash
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes -keyout key.pem -out cert.pem -days 36500 -subj "/CN=localhost" -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```
