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