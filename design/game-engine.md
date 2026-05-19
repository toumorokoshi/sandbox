# Game Engine

If this repository is for a video game, it uses [Godot](https://godotengine.org/).

## Godot MCP server

You **must** use the Godot MCP server to interact with the Godot editor.

## Core Requirements

1. **Godot 4.x (.NET)**: Use the latest version of Godot 4 with C# (.NET) support.
2. **Language**: The game is written in C#. Script files (`.cs`) must be PascalCase and exactly match the class name.
3. **Partial Classes**: All Godot C# scripts MUST be declared as `partial` classes for source generators to function properly.

## Directory Structure

- **assets/**: Game engine assets (2D/3D models, textures, sounds, etc.). Use constants in code to reference these hard-coded paths.
- **src/**: C# source code, ideally organized by feature (e.g., `src/Player/`, `src/UI/`).
- **scenes/**: Godot scene files (`.tscn`). They can also be collocated with their feature scripts in `src/`.
- **addons/**: Third-party plugins.

## Architectural Best Practices

- **Composition Over Inheritance**: Build complex entities by attaching child nodes with specific, single responsibilities, avoiding deep class hierarchies.
- **Top-Down Communication**: Call methods down the scene tree (from parent to child). Use Godot Signals or C# Events to communicate up the tree (from child to parent).
- **Data/Logic Separation**: Use Godot `Resource` classes for defining and storing state, configuration, and data separately from node logic. Make helper functions pure with minimal logic where possible.
- **C# Events vs Godot Signals**: Use standard C# events for purely C# communication. Use Godot `[Signal]` for editor integration and native engine events.

## Instructions for AI Agents

- **Avoid `.tscn` Editing**: The `.tscn` file format is delicate. Prefer providing C# code that programmatically instantiates and configures nodes within `_Ready()` if you cannot confidently and securely modify the scene structure natively.
- **Type Safety**: Leverage C#'s type safety. Always use `GetNode<T>("Path")` over untyped `GetNode()`.
- **Connecting Signals**: Prefer standard C# event syntax for connecting Godot signals from code: `button.Pressed += OnButtonPressed;`.
- **Scene Unique Nodes**: Use `%` (Scene Unique Nodes) to retrieve references to internal nodes and avoid brittle string paths: `GetNode<Label>("%ScoreLabel");`.
