# Global Development Guidelines

## Architecture Principles

### SOLID Principles
- **Single Responsibility Principle (SRP)**: A class should have only one reason to change
- **Open/Closed Principle (OCP)**: Open for extension, closed for modification
- **Liskov Substitution Principle (LSP)**: Subclasses must be able to replace their base classes
- **Interface Segregation Principle (ISP)**: Clients should not be forced to depend on interfaces they don't use
- **Dependency Inversion Principle (DIP)**: Depend on abstractions, not concrete implementations

### Design Pattern Preferences
- Prefer composition over inheritance
- Use dependency injection to improve testability
- Apply repository pattern to separate data access logic
- Use strategy pattern to handle algorithmic variations

## Code Quality

### Naming Conventions
- Use descriptive and clear variable/function names
- Avoid abbreviations and magic numbers
- Follow naming conventions of the project language

### Code Organization
- Keep files and functions concise
- Single functions should not exceed 20 lines (except for complex logic)
- Use meaningful comments to explain "why" rather than "what"

### Error Handling
- Gracefully handle all possible error scenarios
- Provide meaningful error messages
- Avoid silent failures

## Development Practices

### General Recommendations
- **Search first when uncertain**: Conduct web search when encountering technical issues
- **Test-driven**: Write tests for core functionality
- **Documentation updates**: Update relevant documentation when modifying code
- **Security first**: Always consider security, avoid hardcoding sensitive information

### Performance Considerations
- Avoid premature optimization
- Focus on algorithmic complexity
- Use caching appropriately
- Monitor memory usage

### Code Review
- Focus on code readability and maintainability
- Check boundary condition handling
- Verify error handling logic
- Ensure test coverage

### Package Management and Project Standards
- Prefer pnpm for Node.js projects
- Commit titles limited to 70 characters and must be lowercase
- Keep CLAUDE.md file concise, avoid bloat

### General Conventions
- Do not use emojis

## Project Replication Best Practices
When replicating software like Spokenly, follow these guidelines to avoid repeated modifications and inefficient development. The goal is to improve efficiency and ensure progress from 50% to 90%+ completion.

### 1. Thorough Requirements Analysis Phase
- **Reverse Engineer the Original Product**: Don't just look at surface UI; use tools (e.g., browser inspection, network capture) to analyze all functions, API calls, and permission requirements of the original software. List core vs. secondary features with priority ranking.
- **Gap Assessment**: Compare with existing code (e.g., spokenly-clone) and quantify implemented percentages (e.g., transcription 70%, AI 30%). Use tables to record missing modules.
- **Avoid Problems**: The current project overlooked real-time audio capture and text injection, leading to large internal gaps. Dedicate 20% of time to analysis.

### 2. Prototype Validation of Key Features
- **Quick PoC**: Build minimal prototypes to test core features (e.g., Whisper integration for real-time transcription), aiming for validation within 1-2 days.
- **Tool Selection**: Prioritize open-source libraries (e.g., cpal for audio, Whisper.cpp for local processing) and test compatibility (macOS priority).
- **Avoid Problems**: Avoid investing in full features before validation; current repeated modifications stem from unverified features like Agent Mode.

### 3. Modular Development and Iteration
- **Layered Architecture**: Frontend (React) + Backend (Rust/Tauri), with each module independent (e.g., transcription module, AI module) connected via interfaces.
- **Small-Step Iteration**: Test and integrate after each module. Use Git branches for management to avoid major rewrites.
- **Avoid Problems**: Current back-and-forth modifications due to high module coupling; recommend CI/CD for automated testing.

### 4. Early Testing and User Feedback
- **End-to-End Testing**: Simulate real scenarios from permission requests to output, targeting 80% case coverage.
- **Feedback Loop**: Share prototypes at each stage and collect opinions (e.g., prioritize local mode).
- **Avoid Problems**: The current assumption of 90% completion when actually 50% due to lack of testing validation.

### 5. Documentation and Knowledge Management
- **Real-Time Documentation**: Record each step in docs/ (e.g., MODEL_MANAGEMENT_SYSTEM.md), including decision rationale.
- **Risk Assessment**: Anticipate challenges (e.g., permission compatibility) with backup plans.
- **Avoid Problems**: Summarize experiences to prevent repetition in future projects.

Following these practices can double replication efficiency and reduce modification cycles.
