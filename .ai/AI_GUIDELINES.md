I gave you this guidlines that have nothing to have with the prompt but is a general guideline for you HAVE TO follow when working on this project. You should always refer to it before doing any change in the codebase, and make sure to respect it.

## 🧠 AI preamble — before any task

Before writing code, changing architecture, or making any design decision in this project, you **must** first read and internalize:

- @README.md — project structure, crate roles, and high-level data flow.
- docs/@ARCHITECTURE.md - project's architectural design, module responsibilities, and interaction patterns.
- .ai/@PRESENTATION.md - detailed explanations of the core algorithms, data structures, and design rationale.

Please, note that you will find your previous notes in `.ai/notes/` — these are your own documentation of the reasoning and changes you made, and they are meant to be read by a human later on. They are not meant to be read by you before starting a new task, but they can be useful if you need to refresh your memory on a specific component.

Then, for each change you make, **explicitly state** in your response:

1. **Which file(s) you consulted** (e.g., `README.md`).
2. **What the relevant guidance was** (quote or summarize the key constraint).
3. **How your change respects that guidance.**

This ensures every decision is grounded in the project's documented design, not in guesswork.

### 🛑 Architecture change protocol

If your task leads you to believe the architecture or README is inaccurate or could be improved:

1. **Stop.** Do not write code yet.
2. **Propose** the change to me verbally — explain what you'd change and why.
3. **Wait for approval** before updating any documentation or code.
4. Once approved, update `ARCHITECTURE.md` and `README.md` **first**, then proceed to code.

This keeps documentation the source of truth and prevents drift between what the docs say and what the code does.

### Good coding practices

Use triple slashes (`///`) for items (functions, structs, enums) and double slashes with an exclamation mark (`//!`) at the very top of files for module-level documentation. 

Every documentation block must follow this structural layout:
1. **Summary Line:** A single, concise sentence explaining *what* the item does.
2. **Detailed Description (Optional):** A blank line followed by a deeper explanation of *how* or *why* it works if the item is complex.
3. **Sections (Mandatory where applicable):**
   - `# Errors`: If the function returns a `Result`, explicitly document what causes it to return an `Err`.
   - `# Panics`: If the function can panic (or uses `.expect()`), explicitly document the conditions that trigger a panic.
   - `# Safety`: If the function is marked `unsafe`, document the preconditions the caller must uphold.
4. **Examples:** A `# Examples` section containing a fully valid, runnable doctest.

#### Example Format for the AI to follow:

Everytime you write new code, you must make sure there is enough comments for it to be easily understandable by a human reader, and also easily maintainable in the future.

Use triple slashes (`///`) for items (functions, structs, enums) and double slashes with an exclamation mark (`//!`) at the very top of files for module-level documentation. 

Every documentation block must follow this structural layout:
1. **Summary Line:** A single, concise sentence explaining *what* the item does.
2. **Detailed Description (Optional):** A blank line followed by a deeper explanation of *how* or *why* it works if the item is complex.
3. **Sections (Mandatory where applicable):**
   - `# Errors`: If the function returns a `Result`, explicitly document what causes it to return an `Err`.
   - `# Panics`: If the function can panic (or uses `.expect()`), explicitly document the conditions that trigger a panic.
   - `# Safety`: If the function is marked `unsafe`, document the preconditions the caller must uphold.
4. **Examples:** A `# Examples` section containing a fully valid, runnable doctest.

#### Example Format for the AI to follow:

```rust
//! This module provides high-performance telemetry processing tools.

/// A user account within the system architecture.
#[derive(Debug, Clone)]
pub struct User {
    /// The unique identifier for the user.
    pub id: u64,
    /// The user's chosen display name.
    pub username: String,
}

impl User {
    /// Creates a new system user.
    ///
    /// Detailed description: This handles initial validation for names and 
    /// maps the user to a unique database-friendly ID sequence.
    ///
    /// # Errors
    ///
    /// Returns an `Err(ValidationError)` if the username is empty or exceeds 32 characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use my_crate::User;
    /// 
    /// let user = User::new(1, "rustacean".to_string()).unwrap();
    /// assert_eq!(user.id, 1);
    /// ```
    pub fn new(id: u64, username: String) -> Result<Self, String> {
        if username.is_empty() || username.len() > 32 {
            return Err("Invalid username length".to_string());
        }
        Ok(Self { id, username })
    }
}
```

## Note taking

You **must** document your reasoning and the changes you make. Notes live in `.ai/notes/` at the project root.

### How to take notes

1. When you start working on a specific feature or component (e.g., `dart_calculator`), create a subfolder under `.ai/notes/` named after that component:

   `.ai/notes/dart_calculator/`

2. Inside that subfolder, create or update a `README.md` file containing:
   - The goal of the component or task.
   - Your reasoning and design decisions.
   - Every change you made and **why**.
   - Impact on the rest of the codebase (dependencies, breaking changes, etc.).
   - Any information that would help a human reader understand the context later.

### Example structure:

```
.ai/notes/
├── dart_calculator/
│   └── README.md
├── server_api/
│   └── README.md
└── some_other_feature/
    └── README.md
```

> Keep notes concise but meaningful. Prioritize **why** something was done over **what** was done — the code already shows the *what*.

## Testing

You must write tests for every new public function, struct, or module you create. Tests should be placed in the same file as the code they test, within a `#[cfg(test)] mod tests` block.

### Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user() {
        let user = User::new(1, "test".to_string()).unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "test");
    }

    #[test]
    fn test_invalid_username() {
        assert!(User::new(2, "".to_string()).is_err());
    }
}
```

> Run `cargo test --workspace` before considering a task complete.
