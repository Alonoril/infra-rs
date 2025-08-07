# CB Infrastructure Modules (infra-rs)

A comprehensive collection of Rust infrastructure modules designed for building robust blockchain and distributed systems. This workspace provides foundational components that handle common infrastructure concerns including configuration management, caching, database operations, web services, and more.

## 🏗️ Architecture Overview

The `infra-rs` workspace follows a layered architecture approach where each module serves specific infrastructure concerns:

```
┌─────────────────┬─────────────────┬─────────────────┐
│   Application   │   Web Services  │   CLI Tools     │
├─────────────────┼─────────────────┼─────────────────┤
│   web-infra     │   cfg-nacos     │   cli-infra     │
├─────────────────┼─────────────────┼─────────────────┤
│   sql-infra     │   cache-infra   │   rksdb-infra   │
├─────────────────┴─────────────────┴─────────────────┤
│                base-infra (Foundation)               │
└─────────────────────────────────────────────────────┘
```

## 📦 Module Overview

### 🔧 Core Infrastructure

#### **base-infra** - Foundation Library
The cornerstone module providing essential utilities and patterns used across all other modules.

**Key Features:**
- **Error Handling**: Comprehensive error management with structured error codes and macros
- **Configuration**: Figment-based configuration management with environment variable support
- **Logging**: Structured logging with tracing support
- **Runtime Management**: Tokio and Rayon runtime abstractions
- **Codecs**: Binary serialization support (bincode)
- **Utilities**: Time, UUID, string, and vector utilities
- **Validation**: Input validation framework

**Dependencies:**
```toml
base-infra = { workspace = true, features = ["tokio-pool", "bincode"] }
```

### 💾 Data & Storage

#### **cache-infra** - In-Memory Caching System
High-performance caching solution with schema-based type safety and multiple codec support.

**Key Features:**
- **Type-Safe Caching**: Schema-based caching with compile-time type safety
- **Multiple Codecs**: Support for bincode and BCS serialization
- **TTL Support**: Time-to-live functionality for cache entries
- **Async/Sync**: Both synchronous and asynchronous cache operations
- **Memory Management**: Intelligent memory management with Moka

**Usage Example:**
```rust
use cache_infra::{Cacheable, init_cache};

// Initialize cache system
init_cache();

// Define schema and use caching
let cache = AsyncMemCache::new();
cache.store(key, value).await;
let result = cache.load(&key).await;
```

#### **rksdb-infra** - RocksDB Wrapper
Advanced RocksDB integration with schema-based operations and TTL support.

**Key Features:**
- **Schema System**: Type-safe database operations with schema definitions
- **TTL Support**: Built-in time-to-live functionality with automatic cleanup
- **Batch Operations**: Efficient batch read/write operations
- **Column Families**: Advanced column family management
- **Compression**: LZ4 compression support
- **Async Cleanup**: Background TTL cleanup scheduling

**Usage Example:**
```rust
use rksdb_infra::{OpenRocksDB, schemadb::ttl::*};

// Create database with TTL support
let db = MyDB::new(path, "mydb", &config, false, true)?;

// Store data with TTL (expires in 10 minutes)
let expire_at = timestamp_after_minutes(10);
db.put_with_ttl::<MySchema>(&key, &value, expire_at)?;

// Read with automatic TTL checking
let result = db.get_check_ttl::<MySchema>(&key)?;
```

#### **sql-infra** - SQL Database Abstraction
Comprehensive SQL database layer built on SeaORM with PostgreSQL and SQLite support.

**Key Features:**
- **SeaORM Integration**: Full SeaORM compatibility with extensions
- **Multi-Database**: PostgreSQL and SQLite support
- **Big Number Support**: Ethereum uint types (U64, U128, U256) mapping
- **Connection Pooling**: Advanced connection pool management
- **Pagination**: Built-in pagination utilities
- **Type Safety**: Compile-time type safety for database operations

**PostgreSQL Big Number Support:**
```rust
use sql_infra::sea_ext::uint_types::{DbU64, DbU128, DbU256};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "transactions")]
pub struct Model {
    #[sea_orm(column_type = "BigInteger")]
    pub gas_used: DbU64,
    
    #[sea_orm(column_type = "Decimal(Some((78, 0)))")]
    pub wei_amount: DbU256,
}
```

### 🌐 Service & Configuration

#### **web-infra** - Web Framework Infrastructure
Axum-based web framework foundation with error handling and tracing integration.

**Key Features:**
- **Axum Integration**: Built on Axum web framework
- **Error Handling**: Structured error responses with HTTP status codes
- **Request Tracing**: OpenTelemetry compatible request tracing
- **Pagination**: Built-in pagination support for APIs
- **Type Safety**: Strong typing for HTTP handlers

#### **cfg-nacos** - Nacos Configuration Client
Nacos service discovery and configuration management client.

**Key Features:**
- **Service Discovery**: Automatic service registration and discovery
- **Configuration Management**: Dynamic configuration updates
- **Health Checking**: Built-in health check mechanisms
- **Load Balancing**: Client-side load balancing support
- **Async Operations**: Full async/await support

### 🛠️ Development Tools

#### **cli-infra** - CLI Development Support
Command-line interface development utilities built on Clap.

**Key Features:**
- **Clap Integration**: Modern CLI argument parsing
- **Error Integration**: Seamless error handling with base-infra
- **Logging Support**: CLI-friendly logging configuration

## 🚀 Quick Start

### 1. Add to Dependencies

In your `Cargo.toml`:

```toml
[dependencies]
base-infra = { workspace = true, features = ["tokio-pool"] }
cache-infra = { workspace = true }
sql-infra = { workspace = true, features = ["pgsql"] }
web-infra = { workspace = true }
```

### 2. Basic Application Setup

```rust
use base_infra::{config::ConfigExt, logger};
use sql_infra::{DatabaseTrait, cfgs::PgSqlConfig};
use cache_infra::init_cache;

#[tokio::main]
async fn main() -> base_infra::result::AppResult<()> {
    // Initialize logging
    let _guard = logger::init_logger()?;
    
    // Load configuration
    let config = AppConfig::load("config.yml".into())?;
    
    // Initialize cache
    init_cache();
    
    // Setup database
    let db = GlobalDatabase::setup(&config.database).await?;
    
    // Start your application
    Ok(())
}
```

### 3. Error Handling Pattern

```rust
use base_infra::{result::AppResult, map_err, gen_impl_code_enum};

// Define custom error codes
gen_impl_code_enum! {
    MyServiceErr {
        ValidationFailed = ("SVC001", "Input validation failed"),
        DatabaseError = ("SVC002", "Database operation failed"),
    }
}

// Use in service methods
pub async fn process_data(&self, input: &str) -> AppResult<ProcessResult> {
    let validated = self.validate_input(input)
        .map_err(map_err!(&MyServiceErr::ValidationFailed))?;
        
    let result = self.database.save(&validated).await
        .map_err(map_err!(&MyServiceErr::DatabaseError))?;
        
    Ok(result)
}
```

## 🏭 Production Considerations

### Performance
- **Connection Pooling**: All database modules support configurable connection pooling
- **Async by Default**: Built with async/await patterns for high concurrency
- **Zero-Copy**: Minimal data copying with efficient serialization
- **Resource Management**: Proper resource cleanup and lifecycle management

### Reliability
- **Error Handling**: Comprehensive error handling with structured error codes
- **Graceful Shutdown**: Support for graceful service shutdown
- **Health Checks**: Built-in health checking for all external dependencies
- **Retry Logic**: Configurable retry mechanisms for transient failures

### Observability
- **Structured Logging**: OpenTelemetry compatible structured logging
- **Request Tracing**: Full request tracing across service boundaries
- **Metrics**: Built-in metrics collection for performance monitoring
- **Error Tracking**: Detailed error context for debugging

## 🔧 Configuration

### Workspace Dependencies

All modules follow workspace dependency management. Dependencies are centrally managed in the root `Cargo.toml`:

```toml
[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
# ... other dependencies
```

Modules reference dependencies using:

```toml
[dependencies]
tokio.workspace = true
serde.workspace = true
```

### Feature Flags

Each module provides optional feature flags for fine-grained control:

- **base-infra**: `tokio-pool`, `rayon-pool`, `bincode`, `http`
- **sql-infra**: `pgsql`, `sqlite`
- **rksdb-infra**: Inherits from `base-infra` features

## 📚 Documentation

- **API Documentation**: Run `cargo doc --open` for detailed API docs
- **Examples**: Each module includes examples in `examples/` directory
- **Tests**: Comprehensive test coverage with `cargo test`

## 🤝 Contributing

1. Follow the existing code patterns and error handling conventions
2. Add comprehensive unit tests for new functionality
3. Update documentation for public APIs
4. Ensure all modules compile with their default features
5. Follow the workspace dependency management rules

## 📄 License

This project is licensed under the terms specified in the workspace root.

---

**Note**: This infrastructure is designed for production use in blockchain and distributed systems. Each module is battle-tested and follows Rust best practices for performance, safety, and maintainability.