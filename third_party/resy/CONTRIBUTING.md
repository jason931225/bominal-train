# Contributing to Resy Booking Skill

Thank you for your interest in contributing! This document provides guidelines for contributing to the Resy Booking skill.

## 🎯 How to Contribute

### Reporting Bugs

Before creating a bug report:

1. Check if the issue already exists in the [issue tracker](../../issues)
2. Verify your credentials are correctly set
3. Test with the latest version

When reporting bugs, please include:

- Clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Your environment (OS, Python version)
- Any error messages (redact sensitive info!)

### Suggesting Enhancements

Enhancement suggestions are welcome! Please:

- Check if the enhancement has already been suggested
- Provide clear use case and rationale
- Explain expected behavior

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`python3 tests/test_booking.py`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 🛡️ Security

- Never commit credentials or API keys
- Mask sensitive data in logs and error messages
- Only connect to official Resy API endpoints
- Validate all user inputs
- Follow the existing security patterns

## 📝 Code Style

- Follow PEP 8 style guide
- Use type hints where appropriate
- Write docstrings for all functions
- Keep functions focused and small
- Add comments for complex logic

## 🧪 Testing

All contributions should include tests:

```bash
# Run existing tests
python3 tests/test_booking.py

# Run with coverage (if available)
python3 -m coverage run tests/test_booking.py
python3 -m coverage report
```

## 📚 Documentation

Update documentation for any changes:

- Update `SKILL.md` for user-facing changes
- Update `references/api-docs.md` for API changes
- Update `references/error-codes.md` for new error scenarios
- Update `README.md` for feature additions

## 🏷️ Commit Messages

Use clear, descriptive commit messages:

- `feat: Add waitlist support`
- `fix: Handle expired tokens gracefully`
- `docs: Update setup guide`
- `security: Add input validation`
- `test: Add tests for booking flow`

## 🎨 Skill Design Principles

1. **Security First** - Never compromise on security
2. **User-Friendly** - Clear error messages, helpful suggestions
3. **Modular** - Reusable components, single responsibility
4. **Well-Documented** - Clear docs for users and contributors
5. **Tested** - Comprehensive test coverage

## 🔍 Code Review Process

All submissions require review:

1. Automated checks (syntax, tests)
2. Security review
3. Code quality review
4. Documentation review

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

## 🙏 Recognition

Contributors will be acknowledged in the README.

---

Questions? Open an issue or reach out to the maintainers.
