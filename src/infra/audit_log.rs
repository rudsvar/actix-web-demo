//! Logging configuration for storing audit data.

use derive_builder::Builder;
use std::{convert::TryInto, fmt::Debug};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{
    field::{Field, Visit},
    span, Event, Subscriber,
};
use tracing_subscriber::{layer::Context, registry::LookupSpan, Layer};

// Constants to match on
const PRINCIPAL: &str = "principal";
const AUDIT: &str = "audit";
const RETURN: &str = "return";
const ENTITY_ID: &str = "entity_id";
const INPUT: &str = "input";

/// Information about the user making the request.
#[derive(Clone, Debug, Builder)]
pub(crate) struct Principal {
    user_id: i32,
}

impl Principal {
    /// The user id of the principal.
    pub(crate) fn user_id(&self) -> i32 {
        self.user_id
    }
}

impl Visit for PrincipalBuilder {
    fn record_debug(&mut self, _: &Field, _: &dyn Debug) {}

    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == PRINCIPAL {
            let value = value.try_into().expect("user id must be a valid i32");
            self.user_id(value);
        }
    }
}

#[derive(Debug, Builder)]
struct ReturnValue {
    #[builder(default, setter(strip_option))]
    value: Option<String>,
}

impl Visit for ReturnValueBuilder {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == RETURN {
            self.value(format!("{:?}", value));
        };
    }
}

/// Information about a specific audit event.
#[derive(Clone, Debug, Builder)]
pub(crate) struct AuditEvent {
    module: String,
    function: String,
    #[builder(default, setter(strip_option))]
    input: Option<String>,
    #[builder(default, setter(strip_option))]
    entity_id: Option<i32>,
    #[builder(default, setter(strip_option))]
    output: Option<String>,
}

impl AuditEvent {
    pub(crate) fn module(&self) -> &str {
        self.module.as_ref()
    }

    pub(crate) fn function(&self) -> &str {
        self.function.as_ref()
    }

    pub(crate) fn input(&self) -> Option<&String> {
        self.input.as_ref()
    }

    pub(crate) fn entity_id(&self) -> Option<&i32> {
        self.entity_id.as_ref()
    }

    pub(crate) fn output(&self) -> Option<&String> {
        self.output.as_ref()
    }
}

impl Visit for AuditEventBuilder {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == INPUT {
            self.input(format!("{:?}", value));
        }
    }
    fn record_i64(&mut self, field: &Field, value: i64) {
        if field.name() == ENTITY_ID {
            let value = value.try_into().expect("entity id must be a valid i32");
            self.entity_id(value);
        }
    }
}

/// Information about who performed an action and what they did.
#[derive(Debug)]
pub(crate) struct AuditLogEntry {
    principal_data: Principal,
    audit_data: AuditEvent,
}

impl AuditLogEntry {
    pub(crate) fn principal_data(&self) -> &Principal {
        &self.principal_data
    }

    pub(crate) fn audit_data(&self) -> &AuditEvent {
        &self.audit_data
    }
}

pub(crate) struct AuditLayer {
    sender: UnboundedSender<AuditLogEntry>,
}

impl AuditLayer {
    pub(crate) fn new(sender: UnboundedSender<AuditLogEntry>) -> Self {
        Self { sender }
    }
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for AuditLayer {
    // Store information about span
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        if attrs.fields().field(PRINCIPAL).is_some() {
            let mut builder = PrincipalBuilder::default();
            attrs.record(&mut builder);
            let subject_data = builder.build().unwrap();

            let spanref = ctx.span(id).unwrap();
            let mut exts = spanref.extensions_mut();
            // Don't override
            if exts.get_mut::<Principal>().is_none() {
                exts.insert(subject_data);
            }
        }
        if attrs.fields().field(AUDIT).is_some() {
            let mut builder = AuditEventBuilder::default();
            attrs.record(&mut builder);
            builder.module(attrs.metadata().target().to_string());
            builder.function(attrs.metadata().name().to_string());
            let audit_data = builder.build().unwrap();

            let spanref = ctx.span(id).unwrap();
            let mut exts = spanref.extensions_mut();
            exts.insert(audit_data);
        }
    }

    fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>) {
        if let Some(id) = _ctx.current_span().id() {
            let spanref = _ctx.span(id).unwrap();
            // Latest span must be audit span
            if spanref.fields().field(AUDIT).is_none() {
                return;
            }
            // Event must contain return value
            if _event.metadata().fields().field(RETURN).is_none() {
                return;
            }

            // Find span that set the principal
            let subject_span = spanref
                .scope()
                .filter(|s| s.metadata().fields().field(PRINCIPAL).is_some())
                .last()
                .expect("principal not set");
            let extensions = subject_span.extensions();

            // Extract principal data from it
            let subject_data: &Principal = extensions.get().expect("missing principal data");

            // Get audit data from current span
            let mut extensions = spanref.extensions_mut();
            let audit_data: &mut AuditEvent = extensions.get_mut().expect("missing audit data");

            // Get return value if one is set
            let mut ret_data_visitor = ReturnValueBuilder::default();
            _event.record(&mut ret_data_visitor);
            let ret_data = ret_data_visitor.build().unwrap();
            audit_data.output = ret_data.value;

            // Send event to handler
            // TODO: Make sure receiver doesn't close too early
            self.sender
                .send(AuditLogEntry {
                    principal_data: subject_data.clone(),
                    audit_data: audit_data.clone(),
                })
                .unwrap_or_else(|e| tracing::warn!("Lost audit log entry: {}", e));
        }
    }
}
