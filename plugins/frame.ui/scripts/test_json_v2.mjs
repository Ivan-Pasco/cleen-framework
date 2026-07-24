/**
 * test_json_v2.mjs — Unit tests for the _json_encode_v2, _json_encode_pretty_v2,
 * and _json_decode_v2 browser-runtime bridge functions.
 *
 * This file ports the algorithm from runtime/loader.js into a hermetic Node.js
 * harness backed by a real WebAssembly.Memory instance. No compiler, no WASM
 * binary, and no browser globals are required — the bridge logic is what's
 * under test here.
 *
 * IMPORTANT MAINTENANCE NOTE: Any change to the _json_encode_v2,
 * _json_encode_pretty_v2, or _json_decode_v2 implementations in
 * runtime/loader.js MUST be mirrored here. This duplication is intentional
 * and will be eliminated in [P4b] when we have a real WASM harness and the
 * canary runner (run_canaries.mjs) covers these bridges end-to-end.
 *
 * Spec refs:
 *   foundation/spec/platform/BOXED_ANY_ABI.md §3–§5
 *   foundation/spec/platform/MEMORY_MODEL.md §List / §Pairs / §String
 *   foundation/spec/platform/runtime-abi/v1.toml lines 1521–1546
 *
 * Exit 0 if all tests pass. Exit 1 with a failure summary otherwise.
 */

// ---------------------------------------------------------------------------
// Minimal fake WASM environment
// ---------------------------------------------------------------------------

const WASM_PAGE_SIZE = 65536;

// 4 initial pages = 256 KB. Enough for all test allocations; ensureCapacity
// grows it as needed.
const memory = new WebAssembly.Memory({ initial: 4 });

// heapPtr starts at 1024 (DATA_SECTION_START), safely above the null guard at 0.
// In the real loader this is initialised from __heap_ptr export after instantiation.
let heapPtr = 1024;

function checkMemoryGrowth() {
    // No-op in test harness (memory growth is observable but not logged).
}

function ensureCapacity(endOffset) {
    const currentBytes = memory.buffer.byteLength;
    if (endOffset <= currentBytes) return;
    const currentPages = currentBytes / WASM_PAGE_SIZE;
    const neededPages = Math.ceil(endOffset / WASM_PAGE_SIZE);
    const growBy = neededPages - currentPages;
    const growPages = Math.max(growBy * 2, 1);
    try {
        memory.grow(growPages);
    } catch (_) {
        memory.grow(growBy);
    }
}

function readString(ptr, len) {
    const bytes = new Uint8Array(memory.buffer, ptr, len);
    return new TextDecoder().decode(bytes);
}

function writeString(str) {
    const bytes = new TextEncoder().encode(str);
    const ptr = heapPtr;
    const padding = (4 - (bytes.length % 4)) % 4;
    const totalSize = bytes.length + 4 + padding;
    ensureCapacity(ptr + totalSize);
    new DataView(memory.buffer).setUint32(ptr, bytes.length, true);
    new Uint8Array(memory.buffer, ptr, bytes.length + 4).set(bytes, 4);
    heapPtr += totalSize;
    return ptr;
}

// allocBytes: raw bump allocator for fixed-size blocks (boxed-Any, list/pairs headers).
// Ported from the allocBytes helper added to loader.js.
function allocBytes(size) {
    const padded = size + ((8 - (size % 8)) % 8);
    const p = heapPtr;
    ensureCapacity(p + padded);
    heapPtr += padded;
    return p;
}

// dv(): always returns a fresh DataView — required because memory may grow
// on any allocBytes/writeString call, invalidating prior DataView snapshots.
function dv() {
    return new DataView(memory.buffer);
}

// ---------------------------------------------------------------------------
// Bridge implementations (mirrored from loader.js)
// ---------------------------------------------------------------------------

function readLPString(lpPtr) {
    const len = dv().getUint32(lpPtr, true);
    return readString(lpPtr + 4, len);
}

function readBoxedAny(ptr) {
    const tag = dv().getInt32(ptr, true);
    switch (tag) {
        case 0: // Null
            return null;
        case 1: // Integer (i64 little-endian split at ptr+4)
            return Number(dv().getBigInt64(ptr + 4, true));
        case 2: // Boolean
            return dv().getInt32(ptr + 4, true) !== 0;
        case 3: // Number (f64 at ptr+4)
            return dv().getFloat64(ptr + 4, true);
        case 4: { // String — LP pointer at ptr+4
            const lpPtr = dv().getInt32(ptr + 4, true);
            return readLPString(lpPtr);
        }
        case 5: { // Array — list<any> pointer at ptr+4
            const listPtr = dv().getInt32(ptr + 4, true);
            const size = dv().getInt32(listPtr, true);
            const result = [];
            for (let i = 0; i < size; i++) {
                const elemPtr = dv().getInt32(listPtr + 16 + i * 4, true);
                result.push(readBoxedAny(elemPtr));
            }
            return result;
        }
        case 6: { // Object — pairs<string,any> pointer at ptr+4
            const pairsPtr = dv().getInt32(ptr + 4, true);
            const count = dv().getInt32(pairsPtr, true);
            const result = {};
            for (let i = 0; i < count; i++) {
                const entryBase = pairsPtr + 8 + i * 8;
                const keyPtr = dv().getInt32(entryBase, true);
                const valPtr = dv().getInt32(entryBase + 4, true);
                result[readLPString(keyPtr)] = readBoxedAny(valPtr);
            }
            return result;
        }
        default:
            throw new Error('readBoxedAny: invalid boxed-Any tag ' + tag + ' at ptr ' + ptr);
    }
}

function _json_encode_v2(boxedAnyPtr) {
    const value = readBoxedAny(boxedAnyPtr);
    return writeString(JSON.stringify(value));
}

function _json_encode_pretty_v2(boxedAnyPtr) {
    const value = readBoxedAny(boxedAnyPtr);
    return writeString(JSON.stringify(value, null, 2));
}

function writeBoxedAny(value) {
    if (value === null) {
        const ptr = allocBytes(12);
        dv().setInt32(ptr,     0, true);
        dv().setInt32(ptr + 4, 0, true);
        dv().setInt32(ptr + 8, 0, true);
        return ptr;
    }
    if (typeof value === 'boolean') {
        const ptr = allocBytes(12);
        dv().setInt32(ptr,     2, true);
        dv().setInt32(ptr + 4, value ? 1 : 0, true);
        dv().setInt32(ptr + 8, 0, true);
        return ptr;
    }
    if (typeof value === 'number') {
        if (Number.isInteger(value) && Math.abs(value) <= Number.MAX_SAFE_INTEGER) {
            const ptr = allocBytes(12);
            dv().setInt32(ptr, 1, true);
            dv().setBigInt64(ptr + 4, BigInt(value), true);
            return ptr;
        } else {
            const ptr = allocBytes(12);
            dv().setInt32(ptr, 3, true);
            dv().setFloat64(ptr + 4, value, true);
            return ptr;
        }
    }
    if (typeof value === 'string') {
        const lpPtr = writeString(value);
        const ptr = allocBytes(12);
        dv().setInt32(ptr,     4, true);
        dv().setInt32(ptr + 4, lpPtr, true);
        dv().setInt32(ptr + 8, 0, true);
        return ptr;
    }
    if (Array.isArray(value)) {
        const elemPtrs = value.map(writeBoxedAny);
        const listPtr = allocBytes(16 + elemPtrs.length * 4);
        dv().setInt32(listPtr,      elemPtrs.length, true);
        dv().setInt32(listPtr +  4, elemPtrs.length, true);
        dv().setInt32(listPtr +  8, 0,               true); // type_id = 0 (any)
        dv().setInt32(listPtr + 12, 0,               true); // padding
        for (let i = 0; i < elemPtrs.length; i++) {
            dv().setInt32(listPtr + 16 + i * 4, elemPtrs[i], true);
        }
        const ptr = allocBytes(12);
        dv().setInt32(ptr,     5, true);
        dv().setInt32(ptr + 4, listPtr, true);
        dv().setInt32(ptr + 8, 0, true);
        return ptr;
    }
    // typeof "object" (non-null, non-array)
    const entries = Object.entries(value);
    const encodedEntries = entries.map(([k, v]) => [writeString(k), writeBoxedAny(v)]);
    const pairsPtr = allocBytes(8 + encodedEntries.length * 8);
    dv().setInt32(pairsPtr,     encodedEntries.length, true);
    dv().setInt32(pairsPtr + 4, encodedEntries.length, true);
    for (let i = 0; i < encodedEntries.length; i++) {
        const [keyLpPtr, valPtr] = encodedEntries[i];
        dv().setInt32(pairsPtr + 8 + i * 8,     keyLpPtr, true);
        dv().setInt32(pairsPtr + 8 + i * 8 + 4, valPtr,   true);
    }
    const ptr = allocBytes(12);
    dv().setInt32(ptr,     6, true);
    dv().setInt32(ptr + 4, pairsPtr, true);
    dv().setInt32(ptr + 8, 0, true);
    return ptr;
}

function _json_decode_v2(bytesPtr, len) {
    const text = readString(bytesPtr, len);
    let parsed;
    try {
        parsed = JSON.parse(text);
    } catch (_) {
        return 0; // parse-failure sentinel
    }
    return writeBoxedAny(parsed);
}

// ---------------------------------------------------------------------------
// Test helpers — read back a decoded LP string result from memory
// ---------------------------------------------------------------------------

function readResultString(lpPtr) {
    // lpPtr points to a length-prefixed string written by writeString.
    return readLPString(lpPtr);
}

// Read the tag of a decoded boxed-Any result.
function readTag(ptr) {
    return dv().getInt32(ptr, true);
}

// Read the i32 at ptr+4 (value low word).
function readValueI32(ptr) {
    return dv().getInt32(ptr + 4, true);
}

// Read the i64 at ptr+4 as a JS number.
function readValueI64(ptr) {
    return Number(dv().getBigInt64(ptr + 4, true));
}

// Read the f64 at ptr+4.
function readValueF64(ptr) {
    return dv().getFloat64(ptr + 4, true);
}

// Read the LP string via the i32 pointer stored at ptr+4.
function readValueString(ptr) {
    const lpPtr = dv().getInt32(ptr + 4, true);
    return readLPString(lpPtr);
}

// ---------------------------------------------------------------------------
// Test runner
// ---------------------------------------------------------------------------

let passed = 0;
let failed = 0;
const failures = [];

function assert(name, condition, extra) {
    if (condition) {
        console.log('PASS ' + name);
        passed++;
    } else {
        const msg = 'FAIL ' + name + (extra !== undefined ? ' — ' + extra : '');
        console.error(msg);
        failures.push(msg);
        failed++;
    }
}

function assertEq(name, actual, expected) {
    const ok = actual === expected;
    assert(name, ok, ok ? undefined : 'expected ' + JSON.stringify(expected) + ', got ' + JSON.stringify(actual));
}

// Helper: write a JSON string into WASM memory as a raw (non-LP) byte sequence
// and call _json_decode_v2(ptr, len). Returns the boxed-Any pointer.
function decode(jsonText) {
    const bytes = new TextEncoder().encode(jsonText);
    // Allocate raw bytes (no LP prefix — _json_decode_v2 takes (ptr, len)).
    const ptr = allocBytes(bytes.length);
    new Uint8Array(memory.buffer, ptr, bytes.length).set(bytes);
    return _json_decode_v2(ptr, bytes.length);
}

// Helper: build a boxed-Any in memory directly and encode it.
function encode(value) {
    const ptr = writeBoxedAny(value);
    const resultLp = _json_encode_v2(ptr);
    return readResultString(resultLp);
}

function encodePretty(value) {
    const ptr = writeBoxedAny(value);
    const resultLp = _json_encode_pretty_v2(ptr);
    return readResultString(resultLp);
}

// ---------------------------------------------------------------------------
// Tests — encode
// ---------------------------------------------------------------------------

assertEq('encode null',    encode(null),    'null');
assertEq('encode true',    encode(true),    'true');
assertEq('encode false',   encode(false),   'false');
assertEq('encode integer 42', encode(42), '42');
assertEq('encode Number.MAX_SAFE_INTEGER', encode(Number.MAX_SAFE_INTEGER), '9007199254740991');
assertEq('encode number 3.14', encode(3.14), '3.14');
assertEq('encode string "hello"', encode('hello'), '"hello"');

// String with embedded quotes and newline
const strWithSpecials = 'a"b\nc';
assertEq('encode string with quotes and newline', encode(strWithSpecials), JSON.stringify(strWithSpecials));

assertEq('encode empty array', encode([]), '[]');
assertEq('encode empty object', encode({}), '{}');

assertEq('encode array [1,"two",true,null]', encode([1, 'two', true, null]),
    '[1,"two",true,null]');

assertEq('encode object {a:1,b:"x"}', encode({ a: 1, b: 'x' }),
    '{"a":1,"b":"x"}');

assertEq('encode nested {a:[1,{b:2}]}', encode({ a: [1, { b: 2 }] }),
    '{"a":[1,{"b":2}]}');

// Pretty encode
assertEq('encode pretty {a:1}', encodePretty({ a: 1 }),
    JSON.stringify({ a: 1 }, null, 2));

// ---------------------------------------------------------------------------
// Tests — decode
// ---------------------------------------------------------------------------

{
    const ptr = decode('null');
    assert('decode "null" returns non-zero', ptr !== 0);
    assertEq('decode "null" tag', readTag(ptr), 0);
}

{
    const ptr = decode('true');
    assert('decode "true" returns non-zero', ptr !== 0);
    assertEq('decode "true" tag', readTag(ptr), 2);
    assertEq('decode "true" value', readValueI32(ptr), 1);
}

{
    const ptr = decode('false');
    assert('decode "false" returns non-zero', ptr !== 0);
    assertEq('decode "false" tag', readTag(ptr), 2);
    assertEq('decode "false" value', readValueI32(ptr), 0);
}

{
    const ptr = decode('42');
    assert('decode "42" returns non-zero', ptr !== 0);
    assertEq('decode "42" tag', readTag(ptr), 1); // integer path
    assertEq('decode "42" value', readValueI64(ptr), 42);
}

{
    const ptr = decode('3.14');
    assert('decode "3.14" returns non-zero', ptr !== 0);
    assertEq('decode "3.14" tag', readTag(ptr), 3); // float path
    // Floating-point round-trip comparison with tolerance
    const actual = readValueF64(ptr);
    assert('decode "3.14" value approx', Math.abs(actual - 3.14) < 1e-10,
        'got ' + actual);
}

{
    const ptr = decode('"hello"');
    assert('decode string "hello" returns non-zero', ptr !== 0);
    assertEq('decode string "hello" tag', readTag(ptr), 4);
    assertEq('decode string "hello" value', readValueString(ptr), 'hello');
}

{
    const ptr = decode('[1,2,3]');
    assert('decode array [1,2,3] returns non-zero', ptr !== 0);
    assertEq('decode array [1,2,3] tag', readTag(ptr), 5);
    const listPtr = dv().getInt32(ptr + 4, true);
    assertEq('decode array [1,2,3] size', dv().getInt32(listPtr, true), 3);
    // Verify each element is tag=1 (integer)
    for (let i = 0; i < 3; i++) {
        const elemPtr = dv().getInt32(listPtr + 16 + i * 4, true);
        assertEq('decode array [1,2,3] element ' + i + ' tag', readTag(elemPtr), 1);
        assertEq('decode array [1,2,3] element ' + i + ' value', readValueI64(elemPtr), i + 1);
    }
}

{
    const ptr = decode('{"a":1}');
    assert('decode object {"a":1} returns non-zero', ptr !== 0);
    assertEq('decode object {"a":1} tag', readTag(ptr), 6);
    const pairsPtr = dv().getInt32(ptr + 4, true);
    assertEq('decode object {"a":1} count', dv().getInt32(pairsPtr, true), 1);
    // Read the first entry
    const keyLpPtr = dv().getInt32(pairsPtr + 8, true);
    const valPtr   = dv().getInt32(pairsPtr + 8 + 4, true);
    assertEq('decode object {"a":1} key', readLPString(keyLpPtr), 'a');
    assertEq('decode object {"a":1} value tag', readTag(valPtr), 1);
    assertEq('decode object {"a":1} value', readValueI64(valPtr), 1);
}

// Failure cases — _json_decode_v2 must return 0
assertEq('decode malformed "[1,2," returns 0', decode('[1,2,'), 0);
assertEq('decode "not json" returns 0',        decode('not json'), 0);
assertEq('decode empty string returns 0',      decode(''), 0);

// ---------------------------------------------------------------------------
// Roundtrip test
// ---------------------------------------------------------------------------

{
    const original = { nested: { list: [1, 2, 3], nil: null, flag: true, pi: 3.14, str: 'hi' } };
    const originalJson = JSON.stringify(original);

    // Decode into WASM memory, then encode back to a JSON string.
    const decodedPtr = decode(originalJson);
    assert('roundtrip decode returns non-zero', decodedPtr !== 0);
    const encodedLp = _json_encode_v2(decodedPtr);
    const roundtripped = readResultString(encodedLp);

    // Compare stringified forms — structural equality, not pointer equality.
    assertEq('decode+encode roundtrip preserves structure',
        roundtripped,
        JSON.stringify(JSON.parse(originalJson)));
}

// ---------------------------------------------------------------------------
// Hand-built tag-5 boxed-Any encoding correctness
// ---------------------------------------------------------------------------

{
    // Build [42, "x", true, null] by hand in memory, then encode and compare.
    const elems = [42, 'x', true, null];
    const elemPtrs = elems.map(writeBoxedAny);
    const listPtr = allocBytes(16 + elemPtrs.length * 4);
    dv().setInt32(listPtr,      elemPtrs.length, true);
    dv().setInt32(listPtr +  4, elemPtrs.length, true);
    dv().setInt32(listPtr +  8, 0,               true);
    dv().setInt32(listPtr + 12, 0,               true);
    for (let i = 0; i < elemPtrs.length; i++) {
        dv().setInt32(listPtr + 16 + i * 4, elemPtrs[i], true);
    }
    const boxedPtr = allocBytes(12);
    dv().setInt32(boxedPtr,     5, true);
    dv().setInt32(boxedPtr + 4, listPtr, true);
    dv().setInt32(boxedPtr + 8, 0, true);

    const resultLp = _json_encode_v2(boxedPtr);
    const result = readResultString(resultLp);
    assertEq('hand-built tag-5 array encodes correctly', result,
        JSON.stringify([42, 'x', true, null]));
}

// ---------------------------------------------------------------------------
// Hand-built tag-6 boxed-Any encoding preserves insertion order
// ---------------------------------------------------------------------------

{
    // Build {z: 1, a: 2} by hand — insertion order is z then a.
    // JSON.stringify must emit keys in the same order.
    const entries = [['z', 1], ['a', 2]];
    const encodedEntries = entries.map(([k, v]) => [writeString(k), writeBoxedAny(v)]);
    const pairsPtr = allocBytes(8 + encodedEntries.length * 8);
    dv().setInt32(pairsPtr,     encodedEntries.length, true);
    dv().setInt32(pairsPtr + 4, encodedEntries.length, true);
    for (let i = 0; i < encodedEntries.length; i++) {
        const [kp, vp] = encodedEntries[i];
        dv().setInt32(pairsPtr + 8 + i * 8,     kp, true);
        dv().setInt32(pairsPtr + 8 + i * 8 + 4, vp, true);
    }
    const boxedPtr = allocBytes(12);
    dv().setInt32(boxedPtr,     6, true);
    dv().setInt32(boxedPtr + 4, pairsPtr, true);
    dv().setInt32(boxedPtr + 8, 0, true);

    const resultLp = _json_encode_v2(boxedPtr);
    const result = readResultString(resultLp);
    // Must be z before a (insertion order, D3).
    assertEq('hand-built tag-6 object preserves insertion order (z before a)',
        result, '{"z":1,"a":2}');
}

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------

const total = passed + failed;
console.log('\n' + passed + '/' + total + ' tests passed' +
    (failed > 0 ? ' (' + failed + ' failed)' : '') + '.');

if (failed > 0) {
    console.error('\nFailed tests:');
    for (const f of failures) console.error('  ' + f);
    process.exit(1);
}
process.exit(0);
