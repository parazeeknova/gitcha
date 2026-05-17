# Project guidelines:

- when installing new packages, use cargo instead of manually editing the cargo.toml file
- run make check to check for linting & formatting & test errors, and make check-types to check for errors after making changes
- use context7 to get the latest docs about the library or package you are using, and to get help with any issues you encounter
- write tests for your code to ensure it works as expected and to catch any potential bugs early on
- write unit, integration, and end-to-end tests as appropriate for the functionality you are implementing
- use descriptive variable and function names to improve code readability and maintainability
- use git for version control, and commit your changes with descriptive commit messages in the format of `feat`: New feature, `fix`: Bug fix, `docs`: Documentation, `style`: Formatting, `refactor`: Code change, `test`: Adding tests, `chore`: Maintenance, `perf`: Performance, `ci`: Continuous integration, `build`: Build system, `revert`: Revert changes, `wip`: Work in progress example: `feat[MODULE]: Add new module`
