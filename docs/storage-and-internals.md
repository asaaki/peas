# Storage & Internals

## Storage Architecture

```mermaid
graph TB
    subgraph "Repository Layer"
        PR[PeaRepository]
        MR[MemoryRepository]
    end

    subgraph "Caching"
        LC[List Cache - Vec of Pea]
        MC[Map Cache - HashMap by ID]
        INV[Cache Invalidation]
    end

    subgraph "Serialization"
        MP[Markdown Parser]
        TOML_FMT[TOML Frontmatter]
        YAML_FMT[YAML Frontmatter]
    end

    subgraph "File System"
        ACTIVE[.peas/*.md]
        ARCHIVED[.peas/archive/*.md]
        MEMS[.peas/memories/*.md]
        ASSETS_DIR[.peas/assets/]
        UNDO_FILE[.peas/.undo]
        ID_FILE[.peas/.id]
    end

    PR --> LC & MC
    PR --> MP
    MR --> MP

    MP --> TOML_FMT & YAML_FMT

    PR --> ACTIVE & ARCHIVED
    MR --> MEMS

    INV --> LC & MC
```

## File Format Parsing

The markdown parser (`storage/markdown.rs`) auto-detects frontmatter format:

```mermaid
flowchart TD
    INPUT[Raw file content] --> DETECT{Starts with?}
    DETECT -->|"+++"| TOML[Parse TOML frontmatter]
    DETECT -->|"---"| YAML[Parse YAML frontmatter]
    DETECT -->|Other| ERR[Error: Unknown format]

    TOML --> STRUCT[Pea struct]
    YAML --> STRUCT

    STRUCT --> BODY[Extract body after second delimiter]
```

When writing, the configured format (`config.toml` → `frontmatter = "toml"` or `"yaml"`) determines the output format.

## ID Generation

Two modes are available, configured in `config.toml`:

### Random Mode (default)
- Uses `nanoid` with alphanumeric charset
- Format: `{prefix}{random}` (e.g., `peas-a1b2c`)
- Configurable prefix (default: `peas-`) and length (default: 5)
- Collision-resistant for typical project sizes

### Sequential Mode
- Format: `{prefix}{padded_number}` (e.g., `peas-00001`)
- Counter stored in `.peas/.id`
- Padded to `id_length` digits
- Monotonically increasing

## Caching Strategy

```mermaid
sequenceDiagram
    participant Client
    participant Repo as PeaRepository
    participant Cache
    participant Disk

    Client->>Repo: list()
    Repo->>Cache: Check list cache
    alt Cache hit
        Cache-->>Repo: Return cached list
    else Cache miss
        Repo->>Disk: Read all .md files
        Disk-->>Repo: Raw file contents
        Repo->>Repo: Parse all files
        Repo->>Cache: Store in list + map
        Cache-->>Repo: Return list
    end
    Repo-->>Client: Vec of Pea

    Client->>Repo: get(id)
    Repo->>Cache: Check map cache
    alt Cache hit
        Cache-->>Repo: Return cached pea
    else Cache miss
        Repo->>Disk: Read single file
        Disk-->>Repo: Raw content
        Repo->>Repo: Parse file
        Repo->>Cache: Store in map
        Cache-->>Repo: Return pea
    end
    Repo-->>Client: Option of Pea
```

Cache invalidation occurs on:
- Any write operation (create, update, delete, archive)
- External file changes detected by the TUI file watcher

## Undo System

```mermaid
flowchart LR
    subgraph "Undo Stack (LIFO, max 50)"
        OP1[Operation 1]
        OP2[Operation 2]
        OP3[Operation 3]
        OPN[...]
    end

    subgraph "Operation Types"
        CREATE[Create → undo deletes file]
        UPDATE[Update → undo restores old content]
        DELETE[Delete → undo recreates file]
        ARCHIVE[Archive → undo moves back]
    end

    OP1 --> CREATE
    OP2 --> UPDATE
    OP3 --> DELETE
```

Each operation records:
- **Type**: create, update, delete, or archive
- **File path**: Location of the affected file
- **Previous content**: For updates, the file content before the change
- **Timestamp**: When the operation occurred

The stack is persisted as JSON in `.peas/.undo`.

## Asset Management

```mermaid
flowchart TD
    INPUT[Source file] --> VAL{Validate}
    VAL -->|Size > 50MB| REJECT1[Reject: too large]
    VAL -->|Blocked ext| REJECT2[Reject: blocked type]
    VAL -->|OK| COPY[Copy to .peas/assets/ticket-id/]

    COPY --> REF[Add filename to pea.assets]
    REF --> SAVE[Save pea to disk]
```

**Blocked extensions**: `exe`, `dll`, `so`, `dylib`, `bat`, `cmd`, `sh`, `bash`, `ps1`, `js`

Assets are stored in `.peas/assets/{ticket-id}/{filename}` and referenced by filename in the pea's `assets` array.

## Search Engine

```mermaid
flowchart TD
    QUERY[Search query string] --> PARSE{Parse query type}

    PARSE -->|No prefix| SIMPLE[Simple: substring match]
    PARSE -->|"field:value"| FIELD[Field-specific match]
    PARSE -->|"regex:pattern"| REGEX[Regex match]
    PARSE -->|"field:regex:pattern"| COMBINED[Field + regex match]

    SIMPLE --> MATCH[Match against: title, body, ID, tags]
    FIELD --> MATCHF[Match against specified field]
    REGEX --> MATCHR[Regex match against all fields]
    COMBINED --> MATCHFR[Regex match against specified field]

    MATCH & MATCHF & MATCHR & MATCHFR --> RESULTS[Filtered results]
```

All searches are case-insensitive except regex (which follows the pattern's flags).

## Configuration Resolution

```mermaid
flowchart TD
    START[Load config] --> C1{.peas/config.toml?}
    C1 -->|Yes| LOAD1[Load TOML]
    C1 -->|No| C2{.peas/config.yml?}
    C2 -->|Yes| LOAD2[Load YAML]
    C2 -->|No| C3{.peas/config.yaml?}
    C3 -->|Yes| LOAD3[Load YAML]
    C3 -->|No| C4{.peas/config.json?}
    C4 -->|Yes| LOAD4[Load JSON]
    C4 -->|No| C5{Legacy locations?}
    C5 -->|Yes| LOAD5[Load + warn deprecated]
    C5 -->|No| DEFAULT[Use defaults]

    LOAD1 & LOAD2 & LOAD3 & LOAD4 & LOAD5 --> VALIDATE[Validate settings]
    DEFAULT --> VALIDATE
    VALIDATE --> CONFIG[PeasConfig]
```

Legacy locations (`.peas.toml`, `.peas.yml`, etc. in project root) are supported but deprecated. Run `peas doctor --fix` or `peas migrate` to move to the canonical location.

## Security Measures

### Path Traversal Prevention
All file paths are validated against:
- Directory traversal (`..`)
- Absolute paths (`/`, `\`)
- Null bytes
- URL-encoded variants (`%2f`, `%5c`, `%2e%2e`)
- Path containment (output must stay within `.peas/`)

### Input Sanitization
- Title: max 200 characters
- Body: max 50,000 characters
- ID: max 50 characters, restricted charset
- Tag: max 50 characters, restricted charset
- Memory: max 50KB content, max 10,000 entries

### Asset Safety
- Maximum file size: 50MB
- Blocked executable extensions
- Files copied (not linked) to prevent external mutation
