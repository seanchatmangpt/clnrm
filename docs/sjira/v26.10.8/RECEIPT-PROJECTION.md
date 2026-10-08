# Receipt Projection: TestReceipt → sj:Receipt

Semantic Jira (sjira) projection for clnrm's test-execution receipts. Source of truth:
`crates/clnrm-core/src/receipts/receipt.rs`. Line anchors are to this branch
(`fix/port-allocator-hermetic-test` @ `122709c`).

## Why

Castle-gate courts (`courts/stop.rq`) adjudicate "may this milestone stop" over receipt
evidence. If the evidence is ad-hoc markdown, a court is adjudicating over prose. The
TestReceipt structure is already cryptographically verifiable, content-addressed, and
hash-chained — it is a lawful `μ(O*)` consequence record. Projecting it to `sj:Receipt`
lets the court run over real test-execution evidence instead of hand-written claims.

## Field Projection Table

| TestReceipt field | receipt.rs anchor | sj: property | Notes |
|---|---|---|---|
| `id` | L23 (`ReceiptId` = `ContentHash`, L17) | `sj:identity` | Content-addressed: SHA-256 over all fields except `id` and `signature` (`compute_id`, L247–291). A commit cannot contain its own hash — out-of-subject storage, see Chain → replayIdentity. |
| `scenario_id` | L26 | `sj:subject` | What was tested. |
| `capabilities` | L29 | `sj:authority` | Capability IDs exercised = the leased authority under which the run was admitted. |
| `effects` | L32 | `sj:consequence` | Effect set produced by the execution. |
| `sigma_hash` | L35 | `sj:oStar` | Hash of the environment ontology (O*) the run was manufactured against. |
| `image_digests` | L38, `ImageDigest` L67–76 | `sj:consequence` | Immutable container digests: pinned consequence surface. |
| `constraints` | L41 | `sj:bound` | ConstraintSet enforced (hermetic, latency band, determinism, resource limits). |
| `weaver_proof` | L44 (`WeaverProof` L80–98) | `sj:verification` | Weaver/OTEL validation evidence incl. `otel_graph` (L102–114). |
| `timing_footprint` | L47 (`TimingFootprint` L118–133) | `sj:consequence` | Hot/warm/cold paths + τ violations. |
| `hermeticity_witness` | L50 (`HermeticityWitness` L169–190) | `sj:standing` | Isolation proof — primary standing input, see below. |
| `previous_receipt` | L53 | `sj:replayIdentity` | Hash chain binding, see below. |
| `signature` | L56 (`ReceiptSignature` L194–206) | `sj:authority` | Ed25519 over `compute_id()` (L324–383). Optional; unsigned receipts carry standing only via the chain. |
| `timestamp` | L59 | `sj:when` | ISO 8601. |
| `metadata` | L62 | `sj:metadata` | Tags/labels. |

## Chain → sj:replayIdentity

`previous_receipt: Option<ReceiptId>` (L53) binds each receipt into a hash chain. In the
projection, the chain IS the replay path: `sj:replayIdentity` of receipt R<sub>n</sub> is the
pair (R<sub>n</sub>'s `id`, R<sub>n-1</sub>'s `id`). Verification of any receipt is:

1. `compute_id()` over the receipt body equals declared `id` (`validate`, L294–320).
2. `previous_receipt` resolves to a receipt whose `id` matches, recursively to a genesis
   receipt with `previous_receipt: None`.
3. If signed: Ed25519 verify over the id (L324–383), else chain membership is the authority.

This matches the C21 composition (out-of-subject receipts): because `id` is a SHA-256 of the
body, a receipt stored outside its subject's tree still certifies a replayable execution —
git notes / artifact store storage is first-class, not an exception.

## Standing Evaluation: ALIVE / BLOCKED

`sj:standing` is computed, not declared, from projected fields:

- **ALIVE** iff:
  - `validate()` passes (id matches computed hash; L294–302),
  - `constraints.hermetic == true` implies `hermeticity_witness.external_connections` empty
    and `network_isolated` (L312–317), and
  - `hermeticity_witness` fully clean: `filesystem_isolated`, `process_isolated`,
    `deterministic` all true with empty violation vectors (L169–190), and
  - chain link to `previous_receipt` resolves (or is genesis `None`).
- **BLOCKED** iff:
  - `validate()` fails (id mismatch or hermeticity violation), or
  - chain link unresolved (orphan receipt — no replay identity), or
  - `weaver_proof.validation_passed == false` (L88).

Everything else is UNVERIFIED, never rounded up.

## Castle-Gate Integration Path

Courts (`courts/stop.rq`) consume projected receipts as RDF, not markdown:

1. **Emission**: each test run emits a `TestReceipt` (already implemented, receipt.rs) →
   projection to `sj:Receipt` individuals (CONSTRUCT from the serialized receipt JSON).
2. **Projection**: deterministic CONSTRUCT mapping per the table above; no hand-editing of
   projected output (generated, not handwritten).
3. **Court**: `courts/stop.rq` queries over the projected graph:
   - milestone stop requires ≥1 ALIVE receipt per declared scenario/capability, resolved
     chain to genesis, and zero unresolved BLOCKED receipts in the closure;
   - ad-hoc markdown evidence carries no court weight — a claim without a receipt is O,
     not O*.
4. **Replay**: any court admission is replayable via the chain: fetch receipts by id from
   out-of-subject storage, recompute `compute_id()` per link, re-verify signatures.

## Implementation

Implemented: `crates/clnrm-core/src/receipts/sj_projection.rs` — `to_sj_receipt`
(TestReceipt → SjReceipt with computed standing) and `to_sj_ttl` (deterministic,
id-sorted Turtle emission). Standing law as specified above; a receipt viewed in
isolation cannot resolve a `Some` chain link, so un-resolved links project as
BLOCKED, never rounded up.

## See Also

- `crates/clnrm-core/src/receipts/receipt.rs`
- `crates/clnrm-core/src/receipts/store.rs` (receipt persistence)
- Composition C21/C27 (composition-space catalog): out-of-subject receipts, external
  authority courts.
