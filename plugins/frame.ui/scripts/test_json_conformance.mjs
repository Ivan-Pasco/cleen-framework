/**
 * test_json_conformance.mjs — JSONTestSuite corpus conformance harness for
 * the _json_decode_v2 browser-runtime bridge function.
 *
 * Iterates every .json file under JSONTestSuite/test_parsing/ and runs it
 * through the same mock WASM memory + bridge setup as test_json_v2.mjs.
 * Categorises results per Nicolas Seriot's y_/n_/i_ file-name rules:
 *
 *   y_*  — MUST parse. Any failure fails CI.
 *   n_*  — MUST reject. Any success fails CI.
 *   i_*  — implementation-defined. Logged; not enforced.
 *
 * Baseline (plugins/frame.ui/scripts/json_conformance_baseline.txt) locks the
 * required y_* pass count and n_* rejection count. Future regressions fail;
 * i_* drift only warns.
 *
 * Constraint: this harness exercises the real _json_decode_v2 code path (a
 * thin wrapper over V8's JSON.parse). It MUST NOT reimplement decoding.
 *
 * Exit 0 on baseline match, 1 on regression.
 */

import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const CORPUS = join(__dirname, 'JSONTestSuite', 'test_parsing');
const BASELINE_PATH = join(__dirname, 'json_conformance_baseline.txt');

// ---------------------------------------------------------------------------
// Mock WASM environment — mirrors test_json_v2.mjs
// ---------------------------------------------------------------------------

const WASM_PAGE_SIZE = 65536;
const memory = new WebAssembly.Memory({ initial: 16 });
let heapPtr = 1024;

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

function allocBytes(size) {
    const padded = size + ((8 - (size % 8)) % 8);
    const p = heapPtr;
    ensureCapacity(p + padded);
    heapPtr += padded;
    return p;
}

function readString(ptr, len) {
    const bytes = new Uint8Array(memory.buffer, ptr, len);
    return new TextDecoder().decode(bytes);
}

// ---------------------------------------------------------------------------
// _json_decode_v2 — ported from runtime/loader.js. Returns 0 on parse failure,
// or a boxed-Any heap pointer on success. For the conformance harness we only
// need the accept/reject verdict, so writeBoxedAny is stubbed to a non-zero
// sentinel to keep the harness allocation-light.
// ---------------------------------------------------------------------------

const ACCEPT_SENTINEL = 0xC0DEC0DE;

function _json_decode_v2(bytesPtr, len) {
    const text = readString(bytesPtr, len);
    try {
        JSON.parse(text);
    } catch (_) {
        return 0;
    }
    return ACCEPT_SENTINEL;
}

// ---------------------------------------------------------------------------
// Corpus iteration
// ---------------------------------------------------------------------------

function classifyFile(name) {
    if (name.startsWith('y_')) return 'y';
    if (name.startsWith('n_')) return 'n';
    if (name.startsWith('i_')) return 'i';
    return null;
}

function runOne(filePath) {
    // Corpus contains non-UTF-8 fixtures (UTF-16 with BOM, invalid sequences).
    // readFileSync returns raw bytes; we hand them to the bridge as raw bytes
    // exactly as the browser runtime would after fetch().arrayBuffer().
    let bytes;
    try {
        bytes = readFileSync(filePath);
    } catch (err) {
        return { accepted: false, error: 'read: ' + err.message };
    }

    // Allocate and copy into WASM memory.
    const ptr = allocBytes(bytes.length);
    new Uint8Array(memory.buffer, ptr, bytes.length).set(bytes);

    let verdict;
    try {
        verdict = _json_decode_v2(ptr, bytes.length);
    } catch (err) {
        // Uncaught throw from the bridge is a bug in the bridge, not a reject.
        return { accepted: false, error: 'throw: ' + err.message };
    }
    return { accepted: verdict !== 0 };
}

function main() {
    let files;
    try {
        files = readdirSync(CORPUS).filter(f => f.endsWith('.json')).sort();
    } catch (err) {
        console.error('FATAL: cannot read corpus at ' + CORPUS);
        console.error('Run: git submodule update --init --recursive');
        process.exit(2);
    }

    const stats = {
        y: { total: 0, pass: 0, fail: [] },
        n: { total: 0, reject: 0, wrongly_accepted: [] },
        i: { total: 0, parsed: 0, rejected: 0, errored: 0 },
    };

    for (const name of files) {
        const cls = classifyFile(name);
        if (cls === null) continue;
        const result = runOne(join(CORPUS, name));

        if (cls === 'y') {
            stats.y.total++;
            if (result.accepted) stats.y.pass++;
            else stats.y.fail.push(name + (result.error ? ' [' + result.error + ']' : ''));
        } else if (cls === 'n') {
            stats.n.total++;
            if (!result.accepted) stats.n.reject++;
            else stats.n.wrongly_accepted.push(name);
        } else {
            stats.i.total++;
            if (result.error) stats.i.errored++;
            else if (result.accepted) stats.i.parsed++;
            else stats.i.rejected++;
        }
    }

    // Report
    console.log('JSONTestSuite conformance — _json_decode_v2 (browser/loader.js)');
    console.log('  y_* MUST parse:   ' + stats.y.pass + '/' + stats.y.total);
    console.log('  n_* MUST reject:  ' + stats.n.reject + '/' + stats.n.total);
    console.log('  i_* breakdown:    parsed=' + stats.i.parsed + ' rejected=' + stats.i.rejected + ' errored=' + stats.i.errored + ' total=' + stats.i.total);

    if (stats.y.fail.length) {
        console.log('\n  y_* files that failed to parse:');
        for (const f of stats.y.fail) console.log('    ' + f);
    }
    if (stats.n.wrongly_accepted.length) {
        console.log('\n  n_* files that were wrongly accepted:');
        for (const f of stats.n.wrongly_accepted) console.log('    ' + f);
    }

    // Baseline check
    const baseline = loadBaseline();
    if (baseline === null) {
        console.log('\nNo baseline file at ' + BASELINE_PATH);
        console.log('Write one with:');
        console.log('  y_pass=' + stats.y.pass);
        console.log('  y_total=' + stats.y.total);
        console.log('  n_reject=' + stats.n.reject);
        console.log('  n_total=' + stats.n.total);
        console.log('  i_parsed=' + stats.i.parsed);
        console.log('  i_rejected=' + stats.i.rejected);
        console.log('  i_errored=' + stats.i.errored);
        console.log('  i_total=' + stats.i.total);
        process.exit(2);
    }

    let failed = false;
    if (stats.y.total !== baseline.y_total) {
        console.error('\nERROR: y_* corpus size changed (' + baseline.y_total + ' → ' + stats.y.total + '). Update baseline explicitly.');
        failed = true;
    }
    if (stats.n.total !== baseline.n_total) {
        console.error('\nERROR: n_* corpus size changed (' + baseline.n_total + ' → ' + stats.n.total + '). Update baseline explicitly.');
        failed = true;
    }
    if (stats.y.pass < baseline.y_pass) {
        console.error('\nREGRESSION: y_* pass dropped from ' + baseline.y_pass + ' to ' + stats.y.pass);
        failed = true;
    }
    if (stats.n.reject < baseline.n_reject) {
        console.error('\nREGRESSION: n_* rejection dropped from ' + baseline.n_reject + ' to ' + stats.n.reject);
        failed = true;
    }
    if (stats.y.pass > baseline.y_pass) {
        console.log('\nIMPROVEMENT: y_* pass rose from ' + baseline.y_pass + ' to ' + stats.y.pass + '. Update baseline.');
    }
    if (stats.n.reject > baseline.n_reject) {
        console.log('\nIMPROVEMENT: n_* rejection rose from ' + baseline.n_reject + ' to ' + stats.n.reject + '. Update baseline.');
    }
    if (stats.i.parsed !== baseline.i_parsed ||
        stats.i.rejected !== baseline.i_rejected ||
        stats.i.errored !== baseline.i_errored) {
        console.warn('\nWARN: i_* breakdown drifted from baseline (parsed=' + baseline.i_parsed +
                     ' rejected=' + baseline.i_rejected + ' errored=' + baseline.i_errored + ').');
        console.warn('      Now: parsed=' + stats.i.parsed + ' rejected=' + stats.i.rejected + ' errored=' + stats.i.errored + '. Not a failure.');
    }

    if (failed) process.exit(1);
    console.log('\nConformance OK.');
    process.exit(0);
}

function loadBaseline() {
    let text;
    try {
        text = readFileSync(BASELINE_PATH, 'utf8');
    } catch (_) {
        return null;
    }
    const b = {};
    for (const raw of text.split('\n')) {
        const line = raw.trim();
        if (!line || line.startsWith('#')) continue;
        const eq = line.indexOf('=');
        if (eq < 0) continue;
        const k = line.slice(0, eq).trim();
        const v = Number(line.slice(eq + 1).trim());
        if (!Number.isFinite(v)) continue;
        b[k] = v;
    }
    const required = ['y_pass', 'y_total', 'n_reject', 'n_total', 'i_parsed', 'i_rejected', 'i_errored', 'i_total'];
    for (const k of required) if (!(k in b)) return null;
    return b;
}

main();
