# BeanScript

Simple scripting language for easy use in other projects.

## Basic Usage

`beans ./path/to/script.bean`

Use --help for more information or `-i` for interactive mode.

## Example

```
fn(<greet>): {
	print("Hello,", p(0))
}

greet("World") // Hello, World
```
