# RSP10 Development Notes for Claude

This document contains architectural decisions, implementation plans, and context for continuing development of the rsp10 web framework.

## Project Overview

RSP10 (Rust Server Pages) is a web framework focused on:
- **Completely stateless server architecture** - all state serialized to client
- **Server-side logic with graceful degradation** - works without JavaScript
- **Safe Rust only** - no unsafe blocks
- **Clear separation of concerns** - Access, Logic, and Presentation layers
- **Convention over configuration** - minimal boilerplate through naming conventions

## Core Architecture

### State Management Model

Each page is represented by five data elements:

1. **State Key** - Optional query parameters that define initial page load state
2. **Initial State** - State retrieved based on State Key, sent to client
3. **State** - Current state modified by user interactions
4. **Current Initial State** - Freshly recalculated Initial State for comparison
5. **Event** - User action triggering business logic

The state is serialized into hidden form fields (`initial_state_json`, `state_json`) making the server completely stateless. This enables:
- Load balancing without sticky sessions
- Graceful restarts and upgrades
- No session storage required

### Current State (As of 2025-10-01)

**Status**: Work in progress, functional but needs modernization

**Dependencies**:
- Iron framework (deprecated, unmaintained since 2018) ⚠️
- Diesel 1.1 (current is 2.x) ⚠️
- Mustache templates
- SignedCookieBackend for session management

**Known Issues**:
- Templates recompile on every request (no caching despite being mentioned as "trivial")
- State sent to client is not cryptographically signed (security gap)
- Macro-heavy design can make debugging difficult
- No async/await support (Iron is synchronous)

## Major Planned Improvements

### Priority 1: Derive Macro Implementation (CURRENT FOCUS)

**Goal**: Move code generation from procedural macros to derive macros to enable automatic CRUD generation.

**Location**: `rsp10-derive/` crate (currently placeholder)

#### Convention-Based Field Recognition

Fields are identified by naming prefix:
- `txtXXX: String` → Text input field
- `ddXXX: i32` → Dropdown/select element
- `cbXXX: bool` → Checkbox
- `rbXXX: T` → Radio button group
- `taXXX: String` → Textarea (future)
- `btnXXX` → Button (future)
- `dtXXX: DateTime` → Date picker (future)
- `fileXXX: Vec<u8>` → File upload (future)
- No prefix → Plain data for template rendering

#### Dropdown Source Resolution

Use **hybrid convention + attribute override** approach:

```rust
#[derive(RspState)]
pub struct PageState {
    // Convention: looks for get_dd_testing() or get_testing()
    dd_testing: i32,

    // Override: explicit function when convention doesn't fit
    #[rsp_source(dbh_get_dropdown)]
    ddMyDropdown: i32,

    // Reusable function from shared module
    #[rsp_source(common::dropdowns::get_status_list)]
    dd_status: i32,
}
```

**Lookup Strategy**:
1. Check for explicit `#[rsp_source(func)]` attribute
2. Try convention: `get_{full_field_name}()` (e.g., `get_dd_testing()`)
3. Try convention: `get_{name_without_prefix}()` (e.g., `get_testing()`)

**Benefits**:
- Convention works for 80% of cases
- Attributes enable reusability across forms
- Can refactor to shared dropdown functions without changing structs

#### Generated Code

The derive macro should generate:

1. **`fill_data()` implementation** - Converts state to template data
2. **Default `event_handler()`** - Basic event processing
3. **Serialization/deserialization with signing** (see Priority 2)
4. **Form field helpers** - Generate HTML element structs

Example current manual code (teststate.rs:62-89):
```rust
fn fill_data(ri: RspInfo<Self, KeyI32, MyPageAuth>) -> RspFillDataResult<Self> {
    let mut modified = false;
    let mut gd = RspDataBuilder::new();

    rsp10_button!(btnTest, "Test button" => gd);
    rsp10_select!(dd_testing, dbh_get_dropdown(ri.state.dd_testing), ri => gd, modified);
    rsp10_text!(txt_text_message, ri => gd, modified);
    rsp10_check!(cbTestCheck, ri => gd, modified);
    rsp10_data!(modified => gd);

    Self::fill_data_result(ri, gd)
}
```

This should be **completely auto-generated** from the struct definition.

#### Implementation Approach

```rust
// In rsp10-derive/src/lib.rs:
#[proc_macro_derive(RspState, attributes(rsp_source, rsp_key, rsp_auth, rsp_template))]
pub fn derive_rsp_state(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    for field in struct_fields {
        let name = field.ident.to_string();
        match name {
            n if n.starts_with("txt") => generate_text_field(field),
            n if n.starts_with("dd") => generate_dropdown(field),
            n if n.starts_with("cb") => generate_checkbox(field),
            n if n.starts_with("rb") => generate_radio_buttons(field),
            n if n.starts_with("ta") => generate_textarea(field),
            n if n.starts_with("btn") => generate_button(field),
            _ => generate_plain_data(field),
        }
    }

    // Generate RspState trait implementation
}
```

### Priority 2: State Signing for Security

**Problem**: Currently, `initial_state_json` and `state_json` are sent to the client unsigned. While the server recalculates `curr_initial_state` from scratch (defensive approach), this:
- Wastes server resources
- Doesn't prevent state tampering attempts
- No integrity verification

**Solution**: Sign state before sending to client, verify on receipt.

```rust
// On send to client:
initial_state_json = sign(serialize(initial_state), secret)

// On receive from client:
if verify(initial_state_json, secret) {
    use_deserialized_state()
} else {
    // Tampering detected or signature invalid
    log_security_event()
    use_curr_initial_state_from_server()  // fallback
}
```

**Infrastructure Already Exists**:
- `SignedCookieBackend` with secret key management (lib.rs:1017-1020)
- Secret loaded from `.secret` file or generated randomly
- Just needs to be applied to form state fields

**Benefits**:
- State integrity guarantees
- Can trust verified client state (performance improvement)
- Detect tampering attempts for security monitoring
- Optional timestamp in signature for replay prevention

**Integration Point**: This should be integrated into the derive macro's generated serialization code, making it transparent to users.

### Priority 3: Modernization (Future)

**Needed but lower priority than derive macros**:

1. **Migrate from Iron to Axum/Actix-web**
   - Enable async/await
   - Modern HTTP/2, WebSocket support
   - Active maintenance and ecosystem

2. **Implement Template Caching**
   - Currently recompiles on every request (lib.rs:519-525)
   - Easy performance win

3. **Update Dependencies**
   - Diesel 2.x
   - Modern template engine (consider alternatives to Mustache)

4. **Add Modern Web Features**
   - REST API patterns
   - WebSocket support
   - Optional progressive JavaScript enhancement
   - HTMX integration possibilities

## File Structure

```
rsp10/
├── rsp10/                  # Main framework crate
│   ├── src/
│   │   ├── lib.rs         # Core framework (~1124 lines)
│   │   ├── html_types.rs  # HTML element types
│   │   ├── foobuilder.rs  # Data builder for templates
│   │   └── attic.rs       # Old/unused code
│   └── examples/
│       ├── simple.rs      # Example application entry point
│       └── simple_pages/  # Example page implementations
│           ├── teststate.rs   # Interactive form example
│           ├── login.rs       # Authentication example
│           └── ...
├── rsp10-derive/          # Derive macro crate (PLACEHOLDER - TO IMPLEMENT)
│   └── src/
│       └── lib.rs         # Derive macro implementation
└── templates/             # Mustache template files
    └── *.mustache
```

## Key Code Locations

### State Management Core
- `lib.rs:657-966` - `RspState` trait and handler implementation
- `lib.rs:417-467` - Event extraction from requests
- `lib.rs:483-507` - State JSON deserialization
- `lib.rs:567-647` - JSON value amendment from form data

### Template & Response
- `lib.rs:517-525` - Template compilation (NO CACHING)
- `lib.rs:542-555` - Response building
- `lib.rs:799-809` - Build response in trait

### Server Setup
- `lib.rs:969-1034` - `RspServer` with secret management
- `lib.rs:1036-1123` - TCP listener setup with port reuse option

### Macros (TO BE REPLACED BY DERIVE)
- `lib.rs:130-139` - `rsp10_page!` - Page registration
- `lib.rs:203-214` - `rsp10_select!` - Dropdown field
- `lib.rs:234-246` - `rsp10_text!` - Text input field
- `lib.rs:353-366` - `rsp10_check!` - Checkbox field
- `lib.rs:283-293` - `rsp10_button!` - Button element

### Example Implementation
- `examples/simple_pages/teststate.rs` - Complete page example showing all patterns

## Testing the Framework

```bash
# Install prerequisites (Ubuntu)
sudo apt-get install build-essential libsqlite3-dev libpq-dev

# Run example
cargo run --example simple

# Access in browser
# http://127.0.0.1:4480/
# Login: user/pass

# Bind to all interfaces
BIND_IP=0.0.0.0 cargo run --example simple

# Enable port reuse (for multiple workers)
IRON_PORT_REUSE=true cargo run --example simple
```

## Development Workflow

### Adding a New Page (Current Manual Approach)

1. Define state struct with field naming conventions
2. Implement `RspState` trait
3. Write `get_state()` to load initial state
4. Write `event_handler()` for user actions
5. Write `fill_data()` to populate template data (BOILERPLATE - TO BE AUTOMATED)
6. Create Mustache template with form fields
7. Register page in router with `rsp10_page!` macro

### After Derive Macro Implementation

1. Define state struct with field naming conventions
2. Add `#[derive(RspState)]`
3. Write `get_state()` to load initial state (could also be generated with DB integration)
4. Optionally override `event_handler()` for custom logic
5. Create/generate template
6. Register page

**Ultimate Goal**: Full CRUD auto-generation from struct definition + database schema.

## Design Principles

1. **Convention over Configuration** - Naming conventions reduce boilerplate
2. **Stateless Server** - All state in client, server is pure function
3. **Safe Rust Only** - No unsafe blocks
4. **Stable Rust** - No nightly features
5. **Graceful Degradation** - Works without JavaScript
6. **Type Safety** - Leverage Rust's type system throughout

## Security Considerations

### Current State
- ⚠️ State sent to client unsigned (integrity risk)
- ✅ Server always recalculates fresh state (defensive)
- ✅ Authentication via trait pattern
- ⚠️ No explicit CSRF protection shown in examples
- ✅ Cookie signing infrastructure exists

### Planned Improvements
- Sign all state data sent to client
- Add CSRF token generation/verification
- Document security best practices
- Add rate limiting examples
- Security audit after modernization

## Questions for Future Development

1. **Database Integration**: Should derive macro generate Diesel queries?
2. **Validation**: How to specify validation rules in struct attributes?
3. **Relations**: How to handle foreign key dropdowns automatically?
4. **File Uploads**: State management for binary data?
5. **Multi-step Forms**: Wizard pattern support?
6. **API Mode**: Generate REST endpoints from same structs?

## Notes for Claude

- The current macro approach works but is repetitive for users
- Owner values the stateless architecture - preserve this!
- Convention-based field naming is a key design principle
- Derive macro is the foundation for CRUD generation
- Security improvements should be transparent to users
- Start with derive macro, then add signing, then modernize

## References

- README.md - Complete conceptual documentation with examples
- examples/simple_pages/teststate.rs - Reference implementation
- lib.rs - Core framework implementation
