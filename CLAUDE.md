# CLAUDE.md - AI Assistant Guidelines for vdl2025

This document provides guidance for AI assistants working with the vdl2025 codebase.

## Project Overview

**Project Name:** vdl2025
**Status:** Newly initialized repository
**Created:** November 2025

### Current State

This is a freshly initialized repository. The project structure and technology stack have not yet been established. This document will be updated as the project evolves.

## Repository Structure

```
vdl2025/
├── README.md          # Project overview (minimal)
├── CLAUDE.md          # This file - AI assistant guidelines
└── .git/              # Git repository metadata
```

### Planned Directories (TBD)

As the project develops, expect directories such as:
- `src/` - Source code
- `tests/` - Test files
- `docs/` - Documentation
- `config/` - Configuration files

## Development Workflow

### Git Conventions

- **Main Branch:** To be established
- **Commit Messages:** Use conventional commits format
  - `feat:` - New features
  - `fix:` - Bug fixes
  - `docs:` - Documentation changes
  - `refactor:` - Code refactoring
  - `test:` - Test additions/modifications
  - `chore:` - Maintenance tasks

- **Branch Naming:**
  - Feature branches: `feature/<description>`
  - Bug fixes: `fix/<description>`
  - Claude Code sessions: `claude/<session-id>`

### Code Style (To Be Defined)

Code style guidelines will be established once the technology stack is chosen. Expected considerations:
- Linting configuration
- Formatting rules
- Naming conventions
- File organization patterns

## For AI Assistants

### Key Principles

1. **Ask First, Act Later**
   - For ambiguous requirements, ask clarifying questions
   - Confirm destructive operations before executing
   - Validate assumptions about project structure

2. **Preserve Existing Patterns**
   - Once established, follow existing code conventions
   - Maintain consistency with surrounding code
   - Respect the project's architectural decisions

3. **Documentation**
   - Update this CLAUDE.md as the project evolves
   - Document new patterns and conventions as they emerge
   - Keep README.md current with project changes

4. **Testing**
   - Write tests for new functionality
   - Run existing tests before committing changes
   - Follow the project's testing conventions once established

### Common Tasks

#### Adding New Features
1. Understand existing code patterns
2. Plan the implementation
3. Write tests (if test framework is set up)
4. Implement the feature
5. Update documentation as needed
6. Commit with descriptive message

#### Setting Up Project Structure
When initializing project infrastructure:
1. Choose appropriate technology stack
2. Set up build tooling
3. Configure linting and formatting
4. Establish testing framework
5. Create .gitignore for the stack
6. Update this document with new conventions

### Important Files to Watch

- `CLAUDE.md` - This file (keep updated)
- `README.md` - Project overview
- Configuration files (package.json, etc.) - Once created
- Test configuration - Once established

## Technology Stack

**Status:** Not yet defined

When the stack is chosen, update this section with:
- Primary language(s)
- Frameworks and libraries
- Build tools
- Testing framework
- Package manager
- CI/CD setup

## Build & Run Commands

**Status:** Not yet configured

Commands will be documented here once the project is set up. Expected format:

```bash
# Install dependencies
# <command TBD>

# Run development server
# <command TBD>

# Run tests
# <command TBD>

# Build for production
# <command TBD>

# Lint code
# <command TBD>
```

## Testing

**Status:** Test framework not yet established

Testing conventions will be documented here, including:
- Test file naming patterns
- Test organization
- Coverage requirements
- How to run specific tests

## Environment Setup

**Status:** No special setup required yet

Requirements and setup instructions will be added as the project develops.

## API Documentation

**Status:** No APIs defined yet

API endpoints and interfaces will be documented as they are created.

## Known Issues & TODOs

- [ ] Define project purpose and scope
- [ ] Choose technology stack
- [ ] Set up project structure
- [ ] Configure build system
- [ ] Establish testing framework
- [ ] Create .gitignore
- [ ] Set up CI/CD (if needed)
- [ ] Add linting/formatting configuration

## Contributing Guidelines

### For AI Assistants

1. **Before Making Changes:**
   - Read relevant existing code
   - Understand the context and patterns
   - Check for related tests

2. **When Writing Code:**
   - Follow established patterns (once defined)
   - Keep functions focused and small
   - Add appropriate comments for complex logic
   - Avoid introducing security vulnerabilities

3. **After Making Changes:**
   - Run tests (once available)
   - Update documentation if needed
   - Create clear commit messages
   - Review changes before committing

### Security Considerations

- Never commit secrets or credentials
- Validate all user inputs
- Follow OWASP security guidelines
- Use parameterized queries for databases
- Sanitize outputs to prevent XSS

## Changelog

### 2025-11-17
- Initial CLAUDE.md created
- Repository initialized with minimal README
- Project structure and conventions to be defined

---

**Last Updated:** 2025-11-17
**Updated By:** Claude Code Assistant

*This document should be updated whenever significant changes are made to the project structure, conventions, or workflows.*
