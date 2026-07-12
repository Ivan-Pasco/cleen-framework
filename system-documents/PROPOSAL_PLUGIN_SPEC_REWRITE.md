# Proposal — Plugin Spec Rewrite for `frame-data`

**Status:** Draft proposal awaiting developer approval per Principle 25.
**Purpose:** Present the target content for `foundation/spec/plugins/frame-data.ebnf` and `foundation/spec/plugins/frame-data-semantics.md` for review before any spec file is modified.
**Companion documents:**
- `clean-framework/system-documents/SPEC_DATA_PERSISTENCE_MODEL.md` — the design this rewrite implements.
- `clean-framework/system-documents/DRAFT_MIGRATION_PLAN_DATA_PERSISTENCE_MODEL.md` — the migration plan Phase 1 of which produces this rewrite.
- `foundation/management/cross-component-prompts/framework-data-persistence-migration-phase-1-spec.md` — the cross-component prompt for this work.

---

## 1. Executive summary of changes

### 1.1 `frame-data.ebnf` — from v1.2.0 to v2.0.0

**Removed productions:**

- `data_block` in its current bare-field form (`data User\n integer id : pk, auto`) → replaced by a sub-block form.
- `data_field_declaration`, `data_field_constraint_list`, `data_field_constraint`, `data_referential_action`, `data_composite_constraint`, `data_type` (bare-field field declarations) → replaced by structured `fields_sub_block` inside the new `data_block`.
- `mutation_expression` and all descendants (`insert_expression`, `insert_id_expression`, `update_expression`, `upsert_expression`, `delete_expression`, `assignment_pair`, `update_clause`, `set_clause`, `upsert_clause`, `match_clause`, `match_condition`, `on_insert_clause`) → mutations move to the `Database` service (top-level Clean code), out of DSL grammar.
- `data_config_block` and descendants → superseded by `main.cln` `frame.data:` config block, which is a separate concern documented in `04_frame_data.md` §7 (updated).
- `raw_query_expression` and descendants → retained but reclassified under a separate `raw_query` section (escape hatch, not primary API).

**Added productions:**

- `entity_class_declaration` — signals that a plain `class T` in `app/entity/<name>.cln` becomes an entity. No new syntax at the class level; the entity-ness comes from the folder + pairing convention.
- `data_block` in new sub-block form: `data <T>:` followed by required and optional sub-blocks.
- `table_sub_block` — declares the SQL table name.
- `fields_sub_block` — declares which domain fields are persisted, plus per-field constraints and column-name overrides.
- `field_declaration` (new form) — replaces the old `data_field_declaration` with sub-block-scoped syntax.
- `field_constraint` (new form) — includes `primary`, `generated`, `required`, `unique`, `default: <expr>`, `as "column_name"`.
- `indexes_sub_block` — declares indexes; single-field or tuple-form entries; optional `unique` suffix.
- `index_entry` — one entry per index.
- `relations_sub_block` — declares foreign-key relationships.
- `relation_entry` — `<name>: <cardinality> <TargetClass> on <fk_column>`.
- `queries_sub_block` — contains query methods that return the paired domain type or collections/nullables of it.
- `query_method` — one query method declaration.
- `data_accessor_expression` — the `<Entity>.data.<method>(<args>)` dispatch path.
- `database_service_expression` — `Database.save(entity)`, `Database.delete(entity)`, `Database.deleteOrFail(entity)`, `Database.saveAll(list)`, `Database.deleteAll(list)`.

**Retained productions (unchanged):**

- `query_expression`, `query_verb`, `query_argument`, `query_body`, `query_clause` and all descendants — the query DSL is unchanged. Includes `where_clause`, `order_clause`, `limit_clause`, `offset_clause`, `include_clause`, `select_clause`, `join_clause`, `link_clause`, `group_clause`, `having_clause`, `paginate_clause`, `cursor_clause`.
- `path_identifier`, `path_segment`, `comparison_operator`.
- `transaction_block`.
- `migrate_block` and all descendants (schema migrations).
- `raw_query_expression` and descendants (moved to a labeled "escape hatch" section but grammar unchanged).
- `data_plugin_type` (`Model`, `Query`, `Transaction`, `PagedResult<T>`, `CursorResult<T>`).

**Integration with core grammar (unchanged):**

- Data blocks integrate with `top_level_declaration` per Clean's `framework_block` mechanism (opaque to core grammar per `foundation/spec/grammar.ebnf` §6.13).
- `.data` accessor and `Database` service expressions integrate as ordinary expressions per plugin-generated code.

### 1.2 `frame-data-semantics.md` — from ~60 rules to ~70 rules

**Removed rule categories:**

- **DAT-M001..M013 (Model semantics — old form).** Bare-field `data:` model semantics are removed. Replaced by new DAT-E* (entity) and DAT-M* (data-block) rules.
- **DAT-C001, DAT-C002.** Mutation semantics for `delete:` and `update:` block forms — these forms no longer exist. Replaced by `Database.save` / `Database.delete` semantics.
- **DAT-D004 (`insert:` returns the newly created model instance).** No longer applies; `Database.save` mutates in place.
- **DAT-I004 (config file location `app/data/config.cln`).** Replaced by `main.cln` `frame.data:` block; config file location moves.

**Added rule categories:**

- **DAT-E001..E010 — Entity semantics.** How `class T` in `app/entity/` pairs with `data T:` in `app/data/models/`; nullable `id` field; entity constructor semantics with named arguments.
- **DAT-M014..M020 — Data block semantics (new form).** How the sub-blocks (`table`, `fields:`, `indexes:`, `relations:`, `queries:`) relate to each other and to the paired entity.
- **DAT-A001..A005 — Accessor semantics.** How `<Entity>.data.<method>` resolves and dispatches.
- **DAT-S001..S010 — Database service semantics.** How `Database.save`, `Database.delete`, `Database.deleteOrFail`, `Database.saveAll`, `Database.deleteAll` behave; invariant enforcement on write; `id` mutation on INSERT.
- **DAT-P001..P005 — Pairing verification semantics.** Plugin-time checks: entity has a data block, data block has an entity, field alignment.
- **DAT-R001..R003 — Report class semantics.** How `app/data/reports/` classes work; capability-noun naming; return-type rule.

**Retained rule categories (unchanged):**

- **DAT-Q001..Q019 — Query DSL semantics.** All query verb, clause, and path-identifier rules retained.
- **DAT-T001..T005 — Transaction semantics.** All transaction rules retained.
- **DAT-C003..C010 — Constraint / correctness rules.** Business-logic-not-in-model, tenant scoping, migration rules, security rules retained.
- **DAT-D001..D003, D005 — Defaults.** Engine default, pool defaults, `order:` direction default, `upsert:` requirements. (D004 retired as noted above.)
- **DAT-I001..I003, I005..I006 — Integration rules.** Tenant isolation, raw query tenant bypass, seed file location, migration file location, env var access retained.

---

## 2. Target content for `foundation/spec/plugins/frame-data.ebnf`

```ebnf
(* ============================================================================ *)
(* frame-data Plugin Grammar — EBNF Specification                             *)
(* Plugin:  frame-data                                                        *)
(* Version: 2.0.0                                                             *)
(* Date:    2026-07-10                                                        *)
(* Extends: Clean Language core grammar (spec/grammar.ebnf)                  *)
(*                                                                            *)
(* This file specifies the syntax extensions introduced by the frame-data     *)
(* plugin under the v2 (entity/data-pairing) architecture. It defines:        *)
(*                                                                            *)
(*   - The data block sub-block form (table, fields, indexes, relations,      *)
(*     queries), replacing the bare-field form of v1.                         *)
(*   - The .data accessor for reads.                                          *)
(*   - The Database service surface for writes.                               *)
(*   - The retained query DSL (unchanged from v1).                            *)
(*   - The retained transaction and migration blocks (unchanged from v1).     *)
(*   - The retained raw query escape hatch (unchanged from v1).               *)
(*                                                                            *)
(* Removed from v1.2.0:                                                       *)
(*   - Bare-field data: declarations.                                         *)
(*   - Block-form mutations (Model.insert:, Model.update:, Model.delete:,     *)
(*     Model.upsert:, Model.insert_id:).                                      *)
(*   - The data: configuration block (superseded by main.cln frame.data:).    *)
(*                                                                            *)
(* All core grammar productions (expression, identifier, type, statement,     *)
(* string_literal, etc.) are inherited from spec/grammar.ebnf without        *)
(* redefinition.                                                              *)
(*                                                                            *)
(* The data block content itself is opaque to the core grammar per            *)
(* framework_block_content (§6.13 of grammar.ebnf); the productions below     *)
(* specify what the frame-data plugin's expand_block_typed function accepts   *)
(* inside the block body.                                                     *)
(* ============================================================================ *)


(* ======================================================================== *)
(* 1. ENTITY CLASS CONVENTION                                               *)
(* ======================================================================== *)

(* Entity classes in app/entity/<name>.cln are plain Clean class declarations *)
(* per core grammar §6.4. No new syntax is introduced. The entity-ness of a  *)
(* class comes from its folder location and its name-based pairing with a    *)
(* data: block in app/data/models/<name>.cln.                                *)

(* An entity class typically has this shape (all constructs are core Clean): *)

(*   class User                                                              *)
(*       integer? id                                                         *)
(*       string email                                                        *)
(*       private string passwordHash                                         *)
(*       string status                                                       *)
(*       datetime createdAt                                                  *)
(*                                                                            *)
(*       always:                                                             *)
(*           status in ["active", "pending", "suspended"]                    *)
(*                                                                            *)
(*       functions:                                                          *)
(*           public:                                                         *)
(*               boolean canPost()                                           *)
(*                   return status == "active"                               *)

(* Entity semantics (pairing, id nullability, invariant enforcement on save) *)
(* are described in frame-data-semantics.md categories DAT-E and DAT-S.      *)


(* ======================================================================== *)
(* 2. DATA BLOCK DECLARATION (sub-block form)                               *)
(* ======================================================================== *)

(* A data_block declares the persistence view of a paired entity class.     *)
(* Introduced by the "data" keyword followed by the paired class name and   *)
(* a colon. The block body contains ordered sub-blocks.                     *)

data_block          = "data" , class_name , ":" , NEWLINE
                    , INDENT , { data_sub_block } ;

(* Sub-blocks may appear in any order. Each is optional except fields:.     *)
(* Duplicate sub-blocks of the same kind are a compile-time error.          *)
data_sub_block      = table_sub_block
                    | fields_sub_block            (* required *)
                    | indexes_sub_block
                    | relations_sub_block
                    | queries_sub_block ;


(* --- 2.1 TABLE Sub-block --- *)

(* table declares the SQL table name. If omitted, defaults to the           *)
(* snake_case pluralization of the entity class name (User -> users).       *)

table_sub_block     = "table" , string_literal , NEWLINE ;


(* --- 2.2 FIELDS Sub-block --- *)

(* fields declares which domain fields are persisted, plus per-field         *)
(* constraints and column-name overrides. Field types are read from the     *)
(* paired entity class; the data block does not restate them.               *)

fields_sub_block    = "fields" , ":" , NEWLINE
                    , INDENT , INDENT , { field_declaration , NEWLINE } ;

field_declaration   = identifier , { field_constraint } ;

field_constraint    = "primary"                                (* primary key *)
                    | "generated"                              (* DB-generated (typically for id) *)
                    | "required"                               (* non-null *)
                    | "unique"                                 (* uniqueness *)
                    | "default" , ":" , expression             (* default value *)
                    | "as" , string_literal ;                  (* column name override *)


(* --- 2.3 INDEXES Sub-block --- *)

(* indexes declares indexes on the table. Each entry is either a single      *)
(* field name or a parenthesized tuple. Suffix "unique" for a unique index. *)

indexes_sub_block   = "indexes" , ":" , NEWLINE
                    , INDENT , INDENT , { index_entry , NEWLINE } ;

index_entry         = ( identifier | index_tuple ) , [ "unique" ] ;

index_tuple         = "(" , identifier , { "," , identifier } , ")" ;


(* --- 2.4 RELATIONS Sub-block --- *)

(* relations declares foreign-key relationships. Each entry has a name       *)
(* (used for eager loading via include: and for filtering), a cardinality,  *)
(* the target class, and the foreign-key column.                            *)

relations_sub_block = "relations" , ":" , NEWLINE
                    , INDENT , INDENT , { relation_entry , NEWLINE } ;

relation_entry      = identifier , ":" , relation_cardinality
                    , class_name , "on" , identifier ;

relation_cardinality
                    = "has_many"
                    | "has_one"
                    | "belongs_to" ;


(* --- 2.5 QUERIES Sub-block --- *)

(* queries declares read-only methods that return the paired domain type,   *)
(* nullable of it, or a list of it. Called via .data accessor from          *)
(* application code: <Entity>.data.<methodName>(<args>).                    *)

queries_sub_block   = "queries" , ":" , NEWLINE
                    , INDENT , INDENT , { query_method } ;

query_method        = query_return_type , identifier
                    , "(" , [ parameter_list ] , ")"
                    , NEWLINE
                    , INDENT , INDENT , INDENT , query_method_body ;

(* Query methods return the paired entity type T, nullable T?, or list<T>.  *)
(* No other return types are permitted; use a report class in               *)
(* app/data/reports/ for aggregate or cross-entity return types.            *)
query_return_type   = class_name                    (* T — requires findOrFail *)
                    | class_name , "?"              (* T? — requires first *)
                    | "list" , "<" , class_name , ">" ;  (* list<T> — requires find *)

(* The method body is a return statement invoking the query DSL.            *)
query_method_body   = "return" , query_expression ;


(* ======================================================================== *)
(* 3. .data ACCESSOR                                                        *)
(* ======================================================================== *)

(* The .data accessor is a syntactic namespace on entity classes that       *)
(* resolves at compile time (plugin-time) to a query method declared on the *)
(* paired data block. It cannot be captured as a runtime value.             *)

data_accessor_expression
                    = class_name , "." , "data" , "." , identifier
                    , "(" , [ argument_list ] , ")" ;

(* Semantics: see frame-data-semantics.md DAT-A001..A005.                   *)


(* ======================================================================== *)
(* 4. DATABASE SERVICE                                                      *)
(* ======================================================================== *)

(* The Database service is the top-level object providing mutation and     *)
(* bulk operations. Its methods are exposed as static calls on Database.    *)

database_service_expression
                    = "Database" , "." , database_service_method
                    , "(" , [ argument_list ] , ")" ;

database_service_method
                    = "save"                          (* insert if id null, update otherwise *)
                    | "delete"                        (* lenient: no-op if row missing *)
                    | "deleteOrFail"                  (* strict: raises NOT_FOUND if row missing *)
                    | "saveAll"                       (* bulk save *)
                    | "deleteAll" ;                   (* bulk delete *)

(* Semantics: see frame-data-semantics.md DAT-S001..S010.                   *)


(* ======================================================================== *)
(* 5. QUERY EXPRESSIONS (retained unchanged from v1.2.0)                    *)
(* ======================================================================== *)

(* Query operations are invoked on an entity class name using a dot         *)
(* accessor and a query verb, followed by an indented query_body. The       *)
(* verb-to-return-type correspondence is enforced by the plugin:            *)
(*                                                                            *)
(*   find       -> list<T>                                                  *)
(*   first      -> T?                                                       *)
(*   findOrFail -> T                                                        *)
(*   count      -> integer                                                  *)
(*   exists     -> boolean                                                  *)

query_expression    = class_name , "." , query_verb , query_argument ;

query_verb          = "find"
                    | "first"
                    | "count"
                    | "findOrFail"
                    | "exists" ;

query_argument      = ":" , NEWLINE , INDENT , query_body           (* block form   *)
                    | "(" , [ expression ] , ")"                    (* call form    *)
                    | (* empty — bare model.find with no filters *) ;

query_body          = { query_clause } ;

query_clause        = where_clause
                    | order_clause
                    | limit_clause
                    | offset_clause
                    | include_clause
                    | join_clause
                    | link_clause
                    | group_clause
                    | having_clause
                    | select_clause
                    | paginate_clause
                    | cursor_clause ;


(* --- 5.1 WHERE Clause --- *)

where_clause        = "where" , ":" , NEWLINE
                    , INDENT , INDENT , { where_condition , NEWLINE } ;

where_condition     = path_identifier , comparison_operator , expression
                    | expression ;                       (* bare expression fallback *)

(* path_identifier: leading identifier followed by dotted keys and/or       *)
(* bracketed integer indices. Resolution rules per DAT-C010:                *)
(*   leading_id is a join alias  -> dotted segments name columns on the    *)
(*                                  aliased model (no bracketed indices)   *)
(*   leading_id is a json column -> segments form a JSON path (both dotted *)
(*                                  and bracketed indices permitted)       *)
(*   leading_id is an include:d  -> dotted segments name columns on the    *)
(*     relation                     related model (per DAT-Q020, new)      *)
(*   anything else               -> compile-time error                     *)

path_identifier     = identifier , { path_segment } ;

path_segment        = "." , identifier
                    | "[" , integer_literal , "]" ;

comparison_operator = "==" | "!=" | "<" | "<=" | ">" | ">=" | "like" | "in" ;


(* --- 5.2 ORDER Clause --- *)

order_clause        = "order" , ":" , order_field
                    , [ order_direction ]
                    , NEWLINE ;

order_field         = identifier ;

order_direction     = "asc" | "desc" ;


(* --- 5.3 LIMIT and OFFSET Clauses --- *)

limit_clause        = "limit"  , ":" , expression , NEWLINE ;

offset_clause       = "offset" , ":" , expression , NEWLINE ;

paginate_clause     = "paginate" , ":" , NEWLINE
                    , INDENT , INDENT , "page"    , ":" , expression , NEWLINE
                    , INDENT , INDENT , "perPage" , ":" , expression , NEWLINE ;

cursor_clause       = "cursor" , ":" , NEWLINE
                    , INDENT , INDENT , "after"   , ":" , expression , NEWLINE
                    , INDENT , INDENT , "perPage" , ":" , expression , NEWLINE
                    , INDENT , INDENT , "by"      , ":" , identifier , NEWLINE ;


(* --- 5.4 INCLUDE Clause (eager loading / joins with filter scope) --- *)

include_clause      = "include" , ":" , include_list , NEWLINE ;

include_list        = identifier , { "," , identifier } ;


(* --- 5.5 LINK Clause (many-to-many join via junction model) --- *)

link_clause         = "link" , ":" , NEWLINE
                    , INDENT , INDENT , { link_item , NEWLINE } ;

link_item           = class_name , identifier , "==" , identifier
                    | class_name , identifier , "==" , class_name , "." , identifier ;


(* --- 5.6 JOIN Clause --- *)

join_clause         = "join" , ":" , NEWLINE
                    , INDENT , INDENT , { join_entry } ;

join_entry          = class_name , "as" , identifier , [ join_outer ] , ":" , NEWLINE
                    , INDENT , INDENT , INDENT , "on" , ":" , NEWLINE
                    , INDENT , INDENT , INDENT , INDENT , { where_condition , NEWLINE } ;

join_outer          = "left" | "right" ;


(* --- 5.7 GROUP and HAVING Clauses --- *)

group_clause        = "group" , ":" , identifier_list , NEWLINE ;

having_clause       = "having" , ":" , expression , NEWLINE ;

identifier_list     = identifier , { "," , identifier } ;


(* --- 5.8 SELECT Clause (column projection) --- *)

select_clause       = "select" , ":" , projection_list , NEWLINE ;

projection_list     = projection_item , { "," , projection_item } ;

projection_item     = path_identifier
                    | path_identifier , "as" , identifier ;


(* ======================================================================== *)
(* 6. TRANSACTION BLOCK (retained unchanged from v1.2.0)                    *)
(* ======================================================================== *)

(* transaction: wraps its body in a database transaction. If any statement  *)
(* inside raises an error, the entire transaction is rolled back.           *)
(* Nested transaction: blocks are a compile-time error (DAT-T003).          *)

transaction_block   = "transaction" , ":" , NEWLINE
                    , INDENT , { statement } ;


(* ======================================================================== *)
(* 7. MIGRATION BLOCK (retained unchanged from v1.2.0)                      *)
(* ======================================================================== *)

migrate_block       = "migrate" , string_literal , ":" , NEWLINE
                    , INDENT , { migration_direction } ;

migration_direction = up_block
                    | down_block ;

up_block            = "up" , ":" , NEWLINE
                    , INDENT , INDENT , { migration_statement , NEWLINE } ;

down_block          = "down" , ":" , NEWLINE
                    , INDENT , INDENT , { migration_statement , NEWLINE } ;

migration_statement = string_literal
                    | schema_helper_call ;

schema_helper_call  = schema_helper_name , "(" , { expression , "," } , ")" ;

schema_helper_name  = "createTable"
                    | "dropTable"
                    | "addColumn"
                    | "dropColumn"
                    | "renameColumn"
                    | "createIndex"
                    | "dropIndex"
                    | "addForeignKey"
                    | "dropForeignKey"
                    | "execute" ;


(* ======================================================================== *)
(* 8. RAW QUERY (escape hatch, retained unchanged from v1.2.0)              *)
(* ======================================================================== *)

(* db.queryAs(Type) and db.query provide raw-SQL escape hatches for queries *)
(* the DSL doesn't express (deep joins, aggregations, window functions,     *)
(* vendor-specific SQL). Report classes in app/data/reports/ are the        *)
(* natural home for callers of these.                                      *)

raw_query_expression
                    = typed_raw_query
                    | untyped_raw_query ;

typed_raw_query     = "db" , "." , "queryAs" , "(" , type , ")" , ":" , NEWLINE
                    , INDENT , { raw_query_clause } ;

untyped_raw_query   = "db" , "." , "query" , ":" , NEWLINE
                    , INDENT , { raw_query_clause } ;

raw_query_clause    = raw_sql_clause
                    | raw_params_clause ;

raw_sql_clause      = "sql" , ":" , string_literal , NEWLINE ;

raw_params_clause   = "params" , ":" , list_literal , NEWLINE ;


(* ======================================================================== *)
(* 9. BUILT-IN TYPES (retained unchanged from v1.2.0)                       *)
(* ======================================================================== *)

data_plugin_type    = "Model"
                    | "Query"
                    | "Transaction"
                    | paged_result_type
                    | cursor_result_type ;

paged_result_type   = "PagedResult" , "<" , type_name , ">" ;

cursor_result_type  = "CursorResult" , "<" , type_name , ">" ;

type_name           = class_name | type ;


(* ======================================================================== *)
(* 10. INTEGRATION WITH CORE GRAMMAR                                        *)
(* ======================================================================== *)

(* top_level_declaration (core) is extended to also accept:                 *)
(*   data_block | migrate_block                                             *)
(*   (data_config_block from v1.2.0 is removed; connection config moves to  *)
(*    main.cln's frame.data: block per 04_frame_data.md v2 §7.)                *)

(* expression (core) is extended to also accept:                            *)
(*   query_expression                                                       *)
(*   | data_accessor_expression                                             *)
(*   | database_service_expression                                          *)
(*   | raw_query_expression                                                 *)
(*   (mutation_expression from v1.2.0 is removed; mutations use Database.)  *)

(* type (core) is extended to also accept: data_plugin_type                *)

(* statement (core) is extended to also accept: transaction_block          *)
```

---

## 3. Target content for `foundation/spec/plugins/frame-data-semantics.md`

The semantics document is a large file (~430 lines). Rather than reproduce it in full here, this section describes the target structure and lists the specific rule changes. The full rewritten file will follow this structure.

### 3.1 Target structure

```
# frame-data Plugin Semantics
Version: 2.0.0
Date: 2026-07-10

## Categories

- DAT-E — Entity semantics (new)
- DAT-M — Data-block semantics (rewritten for sub-block form)
- DAT-A — .data accessor semantics (new)
- DAT-S — Database service semantics (new)
- DAT-P — Pairing verification semantics (new)
- DAT-R — Report class semantics (new)
- DAT-Q — Query DSL semantics (retained; one new rule DAT-Q020 for include:d relation path resolution)
- DAT-T — Transaction semantics (retained unchanged)
- DAT-C — Constraint/correctness rules (retained except DAT-C001 and DAT-C002 which are removed)
- DAT-D — Defaults (retained except DAT-D004 which is retired)
- DAT-I — Integration rules (retained except DAT-I004 which is updated for main.cln config location)
```

### 3.2 New rule category — DAT-E (Entity semantics)

- **DAT-E001: Entity classes live in `app/entity/<name>.cln`.** One entity class per file. File basename matches the class name in snake_case.
- **DAT-E002: The `id` field is nullable.** Every persistent entity has `integer? id` or `string? id`. `null` before persistence; DB-generated value after `Database.save`. Named-argument constructors omit `id` at construction.
- **DAT-E003: Entities are persistence-ignorant.** An entity class must be constructible, testable, and reasonable without any persistence machinery present. It cannot import from `app/data/`, `app/logic/`, or `app/server/`.
- **DAT-E004: `always:` invariants on the entity are enforced by `Database.save`.** Any invariant failure raises `RUN005 — Assertion Failure` and the save does not proceed.
- **DAT-E005: Entities may exist without a paired data block.** A class in `app/entity/` that never persists (session shape, computed view, value-carrying request) is legal. See DAT-P002 for the converse rule.
- **DAT-E006: Domain-only computed properties are methods.** Fields on the entity that are not stored (computed from other fields) are declared as methods in the class's `functions:` block, not as fields. Called with parentheses: `user.fullName()`. See §11.6 of the persistence spec.
- **DAT-E007: The entity `functions:` block may reference `private` fields freely.** Business methods have full access to the class's state, including `private` fields such as `passwordHash`.

### 3.3 Rewritten rule category — DAT-M (Data-block semantics, sub-block form)

Old DAT-M001..M013 covered bare-field data-block semantics. These are removed. The new rules cover the sub-block form:

- **DAT-M014: `data <T>:` declares the persistence view of paired entity `T`.** The identifier after `data` must exactly match a `class T` declaration in `app/entity/<basename>.cln`.
- **DAT-M015: The `fields:` sub-block is required.** A `data:` block without a `fields:` sub-block is a compile-time error. Every persisted field is declared here.
- **DAT-M016: Field types are inherited from the entity.** The data block declares field constraints, not types. A field declared in `fields:` must exist on the paired entity; its type comes from there.
- **DAT-M017: The `as "column_name"` constraint overrides the SQL column name.** Used for snake_case ↔ camelCase, historical naming, or renames. If omitted, the column name equals the field name.
- **DAT-M018: The `table` sub-block overrides the SQL table name.** If omitted, the table name defaults to the snake_case pluralization of the entity class name (`User` → `users`).
- **DAT-M019: The `indexes:` sub-block declares indexes.** Single-field entries create single-column indexes; parenthesized tuples create composite indexes. Suffix `unique` for uniqueness.
- **DAT-M020: The `relations:` sub-block declares foreign-key relationships.** Each entry has form `<name>: <cardinality> <TargetClass> on <fk_column>`. Cardinalities: `has_many`, `has_one`, `belongs_to`. The `<name>` is used in `include:` and in filter-scope references inside `where:`.

### 3.4 New rule category — DAT-A (`.data` accessor semantics)

- **DAT-A001: `<Entity>.data.<method>` is a syntactic namespace, not a runtime value.** The expression `E.data.m(args)` resolves at plugin-compile time to a `queries:` method on the paired data block. `var facade = E.data` is a compile-time error.
- **DAT-A002: Only methods declared in the paired data block's `queries:` sub-block are dispatchable.** Any other name after `.data.` is a compile-time error.
- **DAT-A003: Query method return types are constrained.** A query method in `queries:` must return `T`, `T?`, or `list<T>` where `T` is the paired entity type. Any other return type is a compile-time error. Aggregate and cross-entity queries move to report classes (see DAT-R).
- **DAT-A004: Query method bodies must be a single `return` statement invoking the query DSL.** No local variables, no arithmetic, no conditionals — the body is a direct DSL call.
- **DAT-A005: Query methods have implicit `public` visibility.** The `public:` sub-section that is required for methods on other classes (per grammar.ebnf §6.4b) is not required inside `queries:`.

### 3.5 New rule category — DAT-S (Database service semantics)

- **DAT-S001: `Database.save(entity)` inserts if `entity.id == null`, updates otherwise.** After INSERT, `entity.id` is mutated in place from `null` to the DB-generated value.
- **DAT-S002: `Database.save` enforces `always:` invariants before persisting.** Any invariant failure raises `RUN005` and the save does not proceed. See DAT-E004.
- **DAT-S003: `Database.delete(entity)` is lenient.** If no row with the entity's primary key exists, the operation is a no-op. Retry-safe, idempotent. Matches SQL's default `DELETE ... WHERE id = ?` returning zero affected rows without error.
- **DAT-S004: `Database.deleteOrFail(entity)` is strict.** If no row with the entity's primary key exists, the operation raises `NOT_FOUND`. See §11.3 of the persistence spec.
- **DAT-S005: `Database.saveAll(list)` is atomic within a transaction if called inside `transaction:`.** Otherwise, each save is individually committed.
- **DAT-S006: `Database.deleteAll(list)` follows `Database.delete` semantics per element.** Missing rows are silently skipped.
- **DAT-S007: The `Database` service is a compile-time namespace, not a runtime value.** `var db = Database` is a compile-time error.
- **DAT-S008: All Database service methods route through the plugin-configured driver.** Postgres, SQLite, and `examples` are the initial supported drivers (v3.0).
- **DAT-S009: Connection routing is single-connection in v3.0.** Read replicas and primary/replica splits are deferred to v3.1+ (see §11.2 of the persistence spec). `Database.readReplica.*` and `Database.primary.*` are reserved namespaces but not yet parsed.
- **DAT-S010: Bulk operations respect batch-size limits from driver configuration.** The plugin generates batched INSERTs/DELETEs when the list exceeds the driver's configured batch size (default: 1000 rows per batch).

### 3.6 New rule category — DAT-P (Pairing verification)

- **DAT-P001: Every `data <T>:` block requires a paired `class T` in `app/entity/<name>.cln`.** A data block without a matching entity is a compile-time error. Corresponds to E-STRUCT-012 in the MCP guidance.
- **DAT-P002: Entities may exist without a paired data block.** See DAT-E005. The reverse (data block without entity) is not permitted per DAT-P001.
- **DAT-P003: Every field named in `fields:` must exist on the paired entity.** A field in `fields:` that isn't declared on the entity is a compile-time error.
- **DAT-P004: Every non-optional field on the entity must appear in `fields:`.** An entity field that isn't in the data block's `fields:` sub-block is a compile-time error, unless the field is declared optional (`type?`) or is a method (per DAT-E006).
- **DAT-P005: Pairing verification runs at plugin-compile time.** The plugin reads both files (entity and data-block) during expansion; alignment failures are reported with the exact file path and field name.

### 3.7 New rule category — DAT-R (Report class semantics)

- **DAT-R001: Cross-entity or aggregate queries live in `app/data/reports/<name>.cln`.** Report classes are named with capability-noun style (`SalesByRegion`, `MonthlyActivity`, `ActivityCohort`) — not entity-scoped.
- **DAT-R002: Report class methods return non-entity types.** If the return type is a persistent entity (`T`, `T?`, `list<T>`), the query belongs in that entity's `queries:` sub-block, not in a report class.
- **DAT-R003: Report class methods may use raw SQL (`db.query` / `db.queryAs`).** Report classes are the natural home for queries that exceed the DSL's expressiveness (deep joins, aggregations, window functions, vendor-specific SQL).

### 3.8 Updated rule — DAT-Q020 (new)

- **DAT-Q020: `path_identifier` segments referring to `include:`d relations name columns on the related model.** Inside a `where:` clause, `orders.status` refers to the `status` column of the `Order` model when `include: orders` is present. Without the `include:`, the path is a compile-time error.

### 3.9 Retained rules (unchanged)

The following rules from v1.2.0 are retained without change:

- **DAT-Q001..Q019** — Query DSL semantics.
- **DAT-T001..T005** — Transaction semantics.
- **DAT-C003..C010** — Business-logic-not-in-model, tenant scoping, migration rules, security rules.
- **DAT-D001..D003, D005** — Engine default, pool defaults, `order:` direction default, `upsert:` requirements. (D004 retired — see §3.10 below.)
- **DAT-I001..I003, I005..I006** — Tenant isolation, raw query tenant bypass, seed file location, migration file location, env var access.

### 3.10 Retired rules

- **DAT-M001..M013** — Bare-field data-block semantics. Replaced by DAT-E and DAT-M014..M020.
- **DAT-C001** — `delete:` without `where:` deletes all rows. The `delete:` block form is removed; delete goes through `Database.delete`.
- **DAT-C002** — `update:` without `where:` updates all rows. The `update:` block form is removed; update goes through `Database.save` on a mutated entity.
- **DAT-D004** — `insert:` returns the newly created model instance. The `insert:` block form is removed; `Database.save` mutates the entity in place.
- **DAT-I004** — Config file location `app/data/config.cln`. Config moves to `main.cln`'s `frame.data:` block. Updated as: "**DAT-I004 (updated): Database connection configuration lives in `main.cln`'s `frame.data:` block, not in a separate config file.**"

---

## 4. Migration checklist for the spec files

Once this proposal is approved:

1. **Back up** the current `foundation/spec/plugins/frame-data.ebnf` and `frame-data-semantics.md` in git before modification.
2. **Replace `frame-data.ebnf`** with the content from §2 above. Bump version to `2.0.0`.
3. **Update `frame-data-semantics.md`** per §3 above:
   - Remove retired rules (§3.10).
   - Add new rule categories (§3.2 DAT-E, §3.3 DAT-M14..M20, §3.4 DAT-A, §3.5 DAT-S, §3.6 DAT-P, §3.7 DAT-R).
   - Add DAT-Q020.
   - Update DAT-I004 wording.
4. **Update `frame-data-semantics.md` version** to `2.0.0`.
5. **Commit both files together** with a message referencing this proposal document and the developer approval.
6. **Promote `SPEC_DATA_PERSISTENCE_MODEL.md`** to `SPEC_DATA_PERSISTENCE_MODEL.md` (rename, update Status §15).
7. **Update the framework CLAUDE.md** if any documentation-sync-protocol references need updating for `frame-data.ebnf` v2.

---

## 5. What this proposal deliberately does NOT change

- Core `foundation/spec/grammar.ebnf` — unchanged. The `data:` block body is opaque to core grammar per `framework_block_content` §6.13.
- Core `foundation/spec/semantic-rules.md` — unchanged. All new semantics are plugin-scoped.
- Core `foundation/spec/type-system.md` — unchanged.
- Core `foundation/spec/ast.md` — unchanged.
- Other plugin specs (`frame-auth.ebnf`, `frame-ui.ebnf`, `frame-server.ebnf`, etc.) — unchanged.
- `foundation/spec/plugins/plugin-contract.md` — unchanged. The `expand_block_typed` interface `frame.data` uses is preserved.

Per the architectural finding in the migration plan §1.1, the entire migration lives in the plugin surface. This proposal implements that scoping.

---

## 6. Approval

**Awaiting developer approval per Principle 25.**

Once approved, the changes described in §4 will be applied to:

- `foundation/spec/plugins/frame-data.ebnf`
- `foundation/spec/plugins/frame-data-semantics.md`
- `clean-framework/system-documents/SPEC_DATA_PERSISTENCE_MODEL.md` (rename + status update)

No spec file is modified before this proposal is approved.

**Reviewer questions to consider before approval:**

1. Is the sub-block set (`table`, `fields:`, `indexes:`, `relations:`, `queries:`) the right final set, or should any be split/merged/renamed?
2. Are the `Database` service methods (`save`, `delete`, `deleteOrFail`, `saveAll`, `deleteAll`) the right initial set?
3. Are the DAT-P pairing rules strict enough (rejecting missing fields) or too strict (would benefit from a "warn but proceed" mode for development)?
4. Should DAT-A003 (query method return type constrained to `T`, `T?`, `list<T>`) also allow `integer` for count-like queries, or is `Entity.data.countActive()` best delegated to a query returning `list<T>` + a `.count` accessor? (Current answer: query DSL already has `count` verb for direct integer returns; entity-scoped counts live in the entity's `queries:` sub-block with `count` verb.)
5. Any other reservations before this becomes committed spec?
