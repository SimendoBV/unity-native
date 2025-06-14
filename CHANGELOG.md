# Changelog

This file will document the most important changes for each released version

## [v0.3.0]

### Bugfixes
- Fixed `is_enabled` check for flushing profiling markers

### Changes
- Tweaked mutability requirement for profiler markers

### Additions
- Added API functions to register/unregister threads with the Unity profiler

## [v0.2.7]

### Bugfixes
- Relaxed version requirements
- Fixed unsafe attribute in macro

## [v0.2.6]

### **YANKED**

### Changes

- Update crate to edition 2024

### Bugfixes

- Logger does not panic anymore when given UTF8 with the NUL codepoint. They are now replaced with `char::REPLACEMENT_CHARACTER`

## [v0.2.5]

### Changes

- Derived debug traits
- Made some interfaces Send + Sync

## [v0.2.4]

### Changes

- Added Plane type

## [v0.2.3]

### Changes

- Added optional app name to logger

## [v0.2.2]

### Changes

- Made conversions from mint types into native Unity math types more convenient

## [v0.2.1]

### Changes

- Added string prefixes to Rust Info/Debug/Trace logs to distinguish between them in Unity, because they all map to the Unity "info" log

## [v0.2.0]

### Features
- Added the basic Unity math types: Vector2/3/4(int), Quaternion, Matrix4x4
- Changed bindgen enum generation

## [v0.1.1]

### Features
- More documentation
- Expanded Profling API to include event metadata
- Added Rust log crate implementation for the Unity Logger
- Started splitting the unity native crate into features

## [v0.1.0]

### Features
- Initial version
- Experimental support for Unity Logging API
- Partial, experimental support for Unity Profiling API
- Helper macros for Unity Plugin entry/exit functions
