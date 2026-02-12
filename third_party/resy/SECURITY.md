# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |

## Security Features

This skill implements multiple security layers:

### Authentication
- Environment variable only credential storage
- No hardcoded secrets
- Token validation on every request
- Automatic retry on auth failure

### Network Security
- HTTPS only (enforced)
- Host validation (only `api.resy.com`)
- Request timeout (30s)
- Rate limiting awareness

### Input Validation
- Date format validation
- Time format validation
- Party size limits (1-20)
- Venue ID validation
- SQL injection prevention (parameterized requests)

### Data Protection
- Sensitive data masking in logs
- No credential exposure in errors
- Memory-only credential handling
- No persistent credential storage

### Audit Logging
- All API calls logged to stderr
- Sensitive fields automatically masked
- Request/response status tracking
- Error logging without credential exposure

## Reporting Security Issues

**Do NOT open public issues for security vulnerabilities.**

Instead:

1. Email security concerns to the maintainers
2. Provide detailed description of the vulnerability
3. Include steps to reproduce (if applicable)
4. Allow time for response before public disclosure

## Security Checklist for Contributors

- [ ] No hardcoded credentials
- [ ] Environment variables for all secrets
- [ ] Input validation on all user inputs
- [ ] URL/hostname validation
- [ ] Sensitive data masking in logs
- [ ] Safe error messages (no credential exposure)
- [ ] HTTPS enforcement
- [ ] Timeout configuration
- [ ] Rate limiting awareness

## Known Limitations

1. **Unofficial API**: This uses Resy's unofficial/reverse-engineered API
   - Resy may change endpoints without notice
   - No SLA or stability guarantees

2. **Token Expiration**: Auth tokens may expire
   - Users must re-extract tokens periodically
   - No automatic token refresh (not supported by API)

3. **Rate Limiting**: Resy may rate limit requests
   - Implemented basic rate limiting awareness
   - Users should avoid excessive API calls

## Security Audit History

### v1.0.0 (2026-02-09)
- Initial security audit completed
- All checklist items passed
- No vulnerabilities found

---

This security policy is a living document. Please suggest improvements.
