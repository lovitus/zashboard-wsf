# Zashboard Native v1.0.2

Release date: 2026-03-14

## Summary

`v1.0.2` is a narrow Android interaction fix for non-built-in upstream setup pages.

It improves tapping saved backend entries on touch devices after entering the active upstream `#/setup` page, without changing built-in / upstream switching semantics.

## What Changed

- add a route-scoped touch patch for the non-built-in upstream setup page
- protect backend action buttons from being interpreted as drag start on touch devices
- preserve backend reorder behavior instead of disabling drag support entirely

## Why This Fix Was Needed

The built-in setup page already includes Android-oriented drag and touch separation, but downloaded upstream UI versions do not necessarily include the same wrapper-side tuning.

As a result, tapping saved backend entries in upstream `#/setup` could be misread as drag initiation on Android, making backend access unreliable.

## Scope

This release is intentionally small:

- fixed: touch interaction for saved backend entries in non-built-in upstream setup pages
- unchanged: built-in UI behavior
- unchanged: upstream / built-in mode switching semantics
- unchanged: tunnel runtime logic

## Implementation Boundary

The patch is intentionally defensive:

- it only runs on the upstream `#/setup` route
- it only targets setup backend list rows that match the expected button layout
- if a future upstream version does not match the expected structure, the patch is designed to fail open instead of breaking the page

## Release Targets

Validated CI targets for `v1.0.2` remain:

- Windows x64
- Windows x86
- macOS arm64
- Android APK
