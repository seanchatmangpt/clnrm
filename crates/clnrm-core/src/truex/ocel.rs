use chrono::{DateTime, Utc};
use opentelemetry_sdk::trace::SpanData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── EventLog — monotonic append-only OCEL store ───────────────────────────────

/// Unique identifier for an event in the log.
pub type EventId = u64;

/// An OCEL 2.0-compatible event with monotonic sequencing and causal links.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcelEvent {
    /// Monotonically increasing ID assigned at append time.
    pub id: EventId,
    /// Human-readable activity label.
    pub activity: String,
    /// Event timestamp.
    pub timestamp: DateTime<Utc>,
    /// Object map: IDs of objects involved in this event.
    pub omap: Vec<String>,
    /// Attribute map: key-value pairs for event attributes.
    pub attributes: HashMap<String, String>,
    /// Causal predecessors: event IDs that must precede this event.
    pub causal_predecessors: Vec<EventId>,
}

impl OcelEvent {
    /// Create a new OcelEvent with the given activity and timestamp.
    pub fn new(activity: impl Into<String>, timestamp: DateTime<Utc>) -> Self {
        Self {
            id: 0,
            activity: activity.into(),
            timestamp,
            omap: Vec::new(),
            attributes: HashMap::new(),
            causal_predecessors: Vec::new(),
        }
    }

    /// Builder: add an object reference.
    pub fn with_object(mut self, obj_id: impl Into<String>) -> Self {
        self.omap.push(obj_id.into());
        self
    }

    /// Builder: add an attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Builder: add a causal predecessor.
    pub fn with_predecessor(mut self, pred_id: EventId) -> Self {
        self.causal_predecessors.push(pred_id);
        self
    }
}

/// Object-Centric Event Log with monotonic append, temporal/object queries, and DOT export.
#[derive(Debug, Default)]
pub struct EventLog {
    events: Vec<OcelEvent>,
    next_id: EventId,
}

impl EventLog {
    /// Create an empty EventLog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an event, assigning a monotonic EventId and returning it.
    ///
    /// The `id` field of the event is set by the log; any value in the input is overwritten.
    pub fn append(&mut self, mut event: OcelEvent) -> crate::error::Result<EventId> {
        let id = self.next_id;
        event.id = id;
        self.next_id += 1;
        self.events.push(event);
        Ok(id)
    }

    /// Query all events that reference a given object ID.
    pub fn query_by_object(&self, obj_id: &str) -> Vec<&OcelEvent> {
        self.events
            .iter()
            .filter(|e| e.omap.iter().any(|o| o == obj_id))
            .collect()
    }

    /// Query events whose timestamps fall within [start, end] (inclusive).
    pub fn query_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&OcelEvent> {
        self.events
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect()
    }

    /// Verify that all causal predecessor links are satisfied.
    ///
    /// For each event, every causal predecessor ID must:
    /// 1. Exist in the log.
    /// 2. Have a strictly earlier monotonic ID (guaranteed by construction, but verified).
    /// 3. Have a timestamp <= this event's timestamp.
    ///
    /// Returns `true` if all causal constraints are satisfied.
    pub fn verify_causal_order(&self) -> bool {
        let id_to_event: HashMap<EventId, &OcelEvent> =
            self.events.iter().map(|e| (e.id, e)).collect();

        for event in &self.events {
            for &pred_id in &event.causal_predecessors {
                match id_to_event.get(&pred_id) {
                    None => return false, // predecessor not found
                    Some(&pred) => {
                        if pred.id >= event.id {
                            return false; // predecessor must come before
                        }
                        if pred.timestamp > event.timestamp {
                            return false; // predecessor must not be later
                        }
                    }
                }
            }
        }
        true
    }

    /// Export the log as OCEL 2.0 JSON format.
    pub fn export_json(&self) -> crate::error::Result<String> {
        // Build OCEL 2.0 JSON structure
        let events_json: Vec<serde_json::Value> = self
            .events
            .iter()
            .map(|e| {
                serde_json::json!({
                    "ocel:id": e.id.to_string(),
                    "ocel:activity": e.activity,
                    "ocel:timestamp": e.timestamp.to_rfc3339(),
                    "ocel:omap": e.omap,
                    "ocel:vmap": e.attributes,
                    "ocel:predecessors": e.causal_predecessors,
                })
            })
            .collect();

        let root = serde_json::json!({
            "ocel:global-event-attributes": [],
            "ocel:global-object-attributes": [],
            "ocel:events": events_json,
        });

        serde_json::to_string_pretty(&root)
            .map_err(|e| crate::error::CleanroomError::serialization_error(e.to_string()))
    }

    /// Export the log as a DOT graph for visualization.
    ///
    /// Each event is a node; causal predecessor edges are drawn as arrows.
    pub fn to_dot_graph(&self) -> String {
        let mut dot = String::from("digraph ocel_event_log {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=box, style=filled, fillcolor=lightblue];\n\n");

        // Nodes
        for event in &self.events {
            let label = format!(
                "{}\\n{}\\n{}",
                event.id,
                event.activity,
                event.timestamp.format("%H:%M:%S")
            );
            dot.push_str(&format!(
                "    e{} [label=\"{}\"];\n",
                event.id, label
            ));
        }

        dot.push('\n');

        // Edges (causal links)
        for event in &self.events {
            for &pred_id in &event.causal_predecessors {
                dot.push_str(&format!(
                    "    e{} -> e{} [label=\"causes\"];\n",
                    pred_id, event.id
                ));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Returns the number of events in the log.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns true if the log is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Access all events (for iteration).
    pub fn events(&self) -> &[OcelEvent] {
        &self.events
    }

    /// Get an event by its ID.
    pub fn get_by_id(&self, id: EventId) -> Option<&OcelEvent> {
        self.events.iter().find(|e| e.id == id)
    }
}

#[cfg(test)]
mod tests_event_log {
    use super::*;

    fn make_ts(offset_secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000 + offset_secs, 0).unwrap()
    }

    #[test]
    fn test_append_and_query_by_object() {
        let mut log = EventLog::new();
        let e1 = OcelEvent::new("Create", make_ts(0)).with_object("order-1");
        let e2 = OcelEvent::new("Ship", make_ts(10)).with_object("order-1").with_object("item-2");
        let e3 = OcelEvent::new("Pay", make_ts(20)).with_object("item-2");

        let id1 = log.append(e1).unwrap();
        let id2 = log.append(e2).unwrap();
        let _id3 = log.append(e3).unwrap();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);

        let order_events = log.query_by_object("order-1");
        assert_eq!(order_events.len(), 2);
        assert_eq!(order_events[0].activity, "Create");

        let item_events = log.query_by_object("item-2");
        assert_eq!(item_events.len(), 2);
    }

    #[test]
    fn test_query_time_range() {
        let mut log = EventLog::new();
        log.append(OcelEvent::new("A", make_ts(0))).unwrap();
        log.append(OcelEvent::new("B", make_ts(100))).unwrap();
        log.append(OcelEvent::new("C", make_ts(200))).unwrap();

        let results = log.query_time_range(make_ts(50), make_ts(150));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].activity, "B");
    }

    #[test]
    fn test_causal_order_valid() {
        let mut log = EventLog::new();
        let id0 = log.append(OcelEvent::new("First", make_ts(0))).unwrap();
        let second = OcelEvent::new("Second", make_ts(10)).with_predecessor(id0);
        log.append(second).unwrap();

        assert!(log.verify_causal_order());
    }

    #[test]
    fn test_export_json() {
        let mut log = EventLog::new();
        log.append(OcelEvent::new("Test", make_ts(0)).with_object("obj-1")).unwrap();

        let json = log.export_json().unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("obj-1"));
    }

    #[test]
    fn test_dot_graph() {
        let mut log = EventLog::new();
        let id0 = log.append(OcelEvent::new("A", make_ts(0))).unwrap();
        log.append(OcelEvent::new("B", make_ts(5)).with_predecessor(id0)).unwrap();

        let dot = log.to_dot_graph();
        assert!(dot.contains("digraph ocel_event_log"));
        assert!(dot.contains("e0 -> e1"));
    }
}

// ── Vector clocks ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VectorClock {
    pub clock: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clock: HashMap::new(),
        }
    }

    pub fn increment(&mut self, node: &str) {
        let count = self.clock.entry(node.to_string()).or_insert(0);
        *count += 1;
    }

    pub fn to_string_repr(&self) -> String {
        let mut parts = Vec::new();
        for (node, count) in &self.clock {
            parts.push(format!("{}:{}", node, count));
        }
        parts.sort(); // Consistent ordering
        parts.join(",")
    }

    pub fn from_string_repr(repr: &str) -> std::result::Result<Self, String> {
        let mut clock = HashMap::new();
        if repr.is_empty() {
            return Ok(Self { clock });
        }
        for part in repr.split(',') {
            let sub_parts: Vec<&str> = part.split(':').collect();
            if sub_parts.len() != 2 {
                return Err(format!("Invalid vector clock part: {}", part));
            }
            let node = sub_parts[0].to_string();
            let count: u64 = sub_parts[1]
                .parse()
                .map_err(|e| format!("Failed to parse count: {}", e))?;
            clock.insert(node, count);
        }
        Ok(Self { clock })
    }
}

#[derive(Debug, Clone)]
pub struct OCELEvent {
    pub id: String,
    pub activity: String,
    pub timestamp: String,
    pub omap: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OCELObject {
    pub id: String,
    pub otype: String,
}

#[derive(Debug, Clone)]
pub struct OcelRdfEventLog {
    pub events: Vec<OCELEvent>,
    pub objects: Vec<OCELObject>,
}

use crate::error::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

// ... existing structs ...

impl OcelRdfEventLog {
    // ... existing implementation ...

    pub fn record_state_transition(event: &OCELEvent, from: String, to: String) -> OCELEvent {
        let mut new_event = event.clone();
        new_event.activity = format!("{} [{} -> {}]", event.activity, from, to);
        new_event
    }
}

pub fn append_to_log(log_path: PathBuf, event: OCELEvent) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .map_err(|e| crate::error::CleanroomError::io_error(e.to_string()))?;

    writeln!(
        file,
        "ocel:event_{} rdf:type ocel:Event ; ocel:activity \"{}\" .",
        event.id, event.activity
    )
    .map_err(|e| crate::error::CleanroomError::io_error(e.to_string()))?;
    Ok(())
}
impl OcelRdfEventLog {
    pub fn from_spans(spans: &[SpanData]) -> Self {
        let mut events = Vec::new();
        let mut objects = Vec::new();

        for (idx, span) in spans.iter().enumerate() {
            let event_id = format!("event_{}", idx);
            let activity = span.name.to_string();

            // Extract container id or other attributes to form objects
            let mut container_id = None;
            for kv in &span.attributes {
                if kv.key.as_str() == "container.id" {
                    container_id = Some(kv.value.to_string());
                }
            }

            let mut omap = Vec::new();
            if let Some(c_id) = container_id {
                let obj_id = format!("obj_{}", c_id);
                omap.push(obj_id.clone());

                // Avoid duplicate objects
                if !objects.iter().any(|o: &OCELObject| o.id == obj_id) {
                    objects.push(OCELObject {
                        id: obj_id,
                        otype: "container".to_string(),
                    });
                }
            }

            events.push(OCELEvent {
                id: event_id,
                activity,
                timestamp: "2026-05-29T17:40:00Z".to_string(),
                omap,
            });
        }

        Self { events, objects }
    }

    pub fn to_rdf_turtle(&self) -> String {
        let mut turtle = String::new();
        turtle.push_str("@prefix ocel: <http://ocel-standard.org/> .\n");
        turtle.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\n");

        for event in &self.events {
            turtle.push_str(&format!("ocel:{} rdf:type ocel:Event ;\n", event.id));
            turtle.push_str(&format!("    ocel:activity \"{}\" ;\n", event.activity));
            turtle.push_str(&format!("    ocel:timestamp \"{}\"", event.timestamp));
            for obj in &event.omap {
                turtle.push_str(&format!(" ;\n    ocel:hasObject ocel:{}", obj));
            }
            turtle.push_str(" .\n\n");
        }

        for obj in &self.objects {
            turtle.push_str(&format!("ocel:{} rdf:type ocel:Object ;\n", obj.id));
            turtle.push_str(&format!("    ocel:type \"{}\" .\n\n", obj.otype));
        }

        turtle
    }
}
