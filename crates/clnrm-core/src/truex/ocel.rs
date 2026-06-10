use opentelemetry_sdk::trace::SpanData;
use std::collections::HashMap;

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
