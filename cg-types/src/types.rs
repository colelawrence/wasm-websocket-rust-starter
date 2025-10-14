use std::collections::HashMap;

use cg_protocol::{mdl, source, view::ViewRecord, SourceFilter, UID};
use i_cg_types_proc::protocol;
use serde::Serialize;

#[protocol("cg")]
pub struct DevString {
    pub message: String,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub records: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub records_dbg: HashMap<String, String>,
    /// Content of page
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub cause: Option<Box<DevString>>,
}

impl std::error::Error for DevString {}
impl std::fmt::Display for DevString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        for (key, value) in &self.records {
            write!(f, "\n  {}: {}", key, value)?;
        }
        for (key, value) in &self.records_dbg {
            write!(f, "\n  {}: {}", key, value)?;
        }
        if let Some(cause) = &self.cause {
            write!(f, "\nCaused by: {}", cause)?;
        }
        Ok(())
    }
}

impl DevString {
    pub fn new<T: Into<String>>(message: T) -> Self {
        Self {
            message: message.into(),
            records: HashMap::new(),
            records_dbg: HashMap::new(),
            cause: None,
        }
    }
    pub fn because(mut self, reason: DevString) -> Self {
        self.cause = Some(Box::new(reason));
        self
    }
    pub fn with<S: ToString, T: Serialize>(mut self, name: S, value: T) -> Self {
        self.records
            .insert(name.to_string(), serde_json::json!(value));
        self
    }
    pub fn with_dbg<S: ToString, T: std::fmt::Debug>(mut self, name: S, value: T) -> Self {
        self.records_dbg
            .insert(name.to_string(), format!("{value:?}"));
        self
    }
}

#[protocol("cg")]
#[derive(Default)]
pub struct WebsiteOpenGraphData {
    /// e.g. "Natural language processing"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// e.g. "Natural language processing (NLP) is a subfield of linguistics, computer science, and artificial intelligence concerned with the interactions between computers and human language."
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// e.g. <https://upload.wikimedia.org/wikipedia/commons/thumb/0/05/NLP_pipeline.png/440px-NLP_pipeline.png>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// e.g. <https://en.wikipedia.org/wiki/Natural_language_processing>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// e.g. "Wikipedia"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub site_name: Option<String>,
    /// og:type - e.g. "article"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og_type: Option<String>,
    /// e.g. "en-US"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// e.g. "Wikipedia contributors"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// e.g. "2001-01-15T12:00:00Z"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_time: Option<At>,
    /// e.g. "2024-01-20T15:32:11Z"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_time: Option<At>,
    /// e.g. "Computer Science"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

#[protocol("cg")]
pub enum WebsiteContent {
    Markdown(String),
    PlainText(String),
}

/// Retrieve items from the timeline.
#[protocol("cg")]
#[codegen(fn = "ping() -> ()")]
pub struct PingParams {}

/// Scrape a website
#[protocol("cg")]
#[codegen(fn = "scrape_website() -> ()")]
pub struct ScrapeWebsiteParams {
    pub url: String,
    pub title: String,
    pub content: WebsiteContent,
    pub open_graph: WebsiteOpenGraphData,
}

/// Retrieve items from the timeline.
#[protocol("cg")]
#[codegen(fn = "timeline() -> TimelineResult")]
pub struct TimelineParams {
    /// Defaults to a limit of 50
    pub limit: Option<usize>,
    /// Defaults to 30 days into the past
    pub ts_min: Option<At>,
    /// Defaults to 30 days from now
    pub ts_max: Option<At>,
    /// Only include items with a URL matching the regex.
    pub url_regex: Option<String>,
    /// Exclude any items with a URL matching the regex.
    pub url_exclude_regex: Option<String>,
    /// Only include items with scraped content.
    pub require_m: bool,
    /// Defaults to true, include scraped content.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub include_m: Option<bool>,
}

/// Search for things.
#[protocol("cg")]
#[codegen(cg_imports = "SourceFilter")]
#[codegen(fn = "search() -> SearchResult")]
pub struct SearchParams {
    pub reference_time: At,
    /// Limit the number of results for each account source API call (e.g. number of Google Drive results, number of Gmail results, etc)
    pub per_source_limit: Option<usize>,
    /// Defaults to "two weeks ago"
    pub acger: Option<String>,
    /// Defaults to "next week"
    pub before: Option<String>,
    /// Defaults to 180 seconds
    pub timeout_secs: Option<u64>,
    /// Query, which will be used to find records matching these as exact phrases.
    pub terms: Vec<String>,
    /// Filter to specific sources.
    pub source_filter: Option<SourceFilter>,
    #[serde(skip_serializing_if = "is_false", default)]
    /// Include statements (`.stm`) in the [ViewRecord] results.
    pub with_statements: bool,
    pub user_source_rankings:
        HashMap<String, cg_base::mvp_source_filtering::AccountUserSourceRankings>,
    /// Which sources to use for the search (default =`All` when `None`).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source_mode: Option<SearchSourceMode>,
    // For, now this is derived from `acger`, `before`, but in the future, maybe more?
    // /// Filter for specific integrations
    // pub account_custom_filter_set: AccountCustomFilterSet,
}

#[protocol("cg")]
pub enum SearchSourceMode {
    /// Search local index AND invoke account APIs (default behaviour).
    All,
    /// Search local index only – do NOT invoke account APIs.
    IndexOnly,
    /// Skip the local index – query account APIs only.
    ApisOnly,
}

fn is_false(value: &bool) -> bool {
    !value
}

/// Result from `search()` function.
#[protocol("cg")]
#[codegen(cg_imports = "ViewRecord")]
pub struct SearchResult {
    pub records: Vec<ViewRecord>,
    pub why: DevString,
}

/// Get current page's record
#[protocol("cg")]
#[codegen(fn = "find_record_for_url() -> FindRecordResult")]
pub struct FindRecordForURLParams {
    /// URL to get the record for
    pub url: String,
    // /// Include statements on the record
    // pub with_statements: bool,
    // /// Include full text on the record
    // pub with_full_text: bool,
}
#[protocol("cg")]
pub struct FindRecordResult {
    pub found: ViewRecord,
}

/// Get current page's record
#[protocol("cg")]
#[codegen(fn = "check_content_relevance() -> ContentRelevanceResult")]
pub struct CheckContentRelevanceParams {
    /// URL of the self
    pub self_url: Option<String>,
    /// URLs mention in content
    pub mentioned_urls: Vec<String>,
    /// Email addresses scraped from content
    pub mentioned_emails: Vec<String>,
    /// Article title / document title
    pub self_titles: Vec<String>,
    /// Text content (in markdown), which might have proper nouns in it
    pub prose: Vec<String>,
    // /// Posted dates etc
    // pub dates: Vec<At>,
    // pub self_posted_date: Vec<String>,
}

#[protocol("cg")]
#[codegen(cg_imports = "SourceFilterCriteria as FilterCriteria")]
pub struct ContentRelevanceResult {
    /// We found relevant things in index
    pub found_in_index: u64,
    /// Relevant content kind
    pub item: RelevantItem,
    /// Filters that can be used to find the relevant content
    pub filters: Vec<source::FilterCriteria>,
}

#[protocol("cg")]
pub enum RelevantItem {
    /// Email address passed in as mentioned
    Email(String),
    /// Proper noun (usually derived from the prose)
    Name(String),
    /// URL passed in as mentioned
    Record(RecordID),
    /// URL passed in as current page
    SelfRecord(RecordID),
}

/// Parse a relative time string like "2 weeks ago" and return a timestamp.
#[protocol("cg")]
#[codegen(fn = "parse_time() -> ParseTimeResult")]
pub struct ParseTimeParams {
    pub input: String,
    pub reference_time: At,
}

#[protocol("cg")]
pub struct ParseTimeResult {
    pub timestamp: At,
}

type RecordID = UID;

/// Retrieve record full text.
#[protocol("cg")]
#[codegen(fn = "record_info() -> RecordInfoResult")]
pub struct RecordInfoParams {
    pub record_ids: Vec<RecordID>,
    /// Only return full text if it's cached.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub only_cached_full_text: Option<bool>,
}

#[protocol("cg")]
pub struct RecordInfoResult(pub RecordID, pub Option<RecordInfoFound>);

#[protocol("cg")]
#[codegen(cg_imports = "ViewRecord")]
pub struct RecordInfoFound {
    pub record: ViewRecord,
    pub full_text: Option<String>,
}

/// Retrieve statement information.
/// Returns one [`ObjectLinkInfoResult`] with all targets resolved.
#[protocol("cg")]
#[codegen(fn = "object_link_info() -> ObjectLinkInfoResult")]
#[codegen(cg_imports = "Object")]
pub struct ObjectLinkInfoParams {
    pub targets: Vec<mdl::Object>,
}

/// See [ObjectLinkInfoParams].
#[protocol("cg")]
#[codegen(cg_imports = "Object")]
pub struct ObjectLinkInfoResult(pub Vec<(mdl::Object, Option<String>)>);

/// WIP Implementing a search to learn more about how records currently work.
#[protocol("cg")]
#[codegen(fn = "wip_search() -> WIPSearchResult")]
pub struct WIPSearchParams {
    pub query: String,
}

/// See [WIPSearchParams].
#[protocol("cg")]
pub struct WIPSearchResult {
    pub hmm: DevString,
}

/// Retrieve items from the extension storage on this device
#[protocol("cg")]
#[codegen(fn = "in_device() -> DeviceStorageValue")]
#[serde(transparent)]
pub struct DeviceStorageCommand(StorageCommand);

impl DeviceStorageCommand {
    pub fn into_storage_command(self) -> StorageCommand {
        self.0
    }
}

/// Retrieve items from the extension storage which is synchronized across devices
#[protocol("cg")]
#[codegen(fn = "in_synced() -> SyncStorageValue")]
#[serde(transparent)]
pub struct SyncStorageCommand(StorageCommand);

impl SyncStorageCommand {
    pub fn into_storage_command(self) -> StorageCommand {
        self.0
    }
}

#[protocol("cg")]
pub enum StorageCommand {
    Get {
        key: String,
    },
    Set {
        key: String,
        value: Option<serde_json::Value>,
    },
    Watch {
        key: String,
    },
}

#[protocol("cg")]
pub struct DeviceStorageValue(pub Option<serde_json::Value>);

impl From<Option<serde_json::Value>> for DeviceStorageValue {
    fn from(value: Option<serde_json::Value>) -> Self {
        Self(value)
    }
}

#[protocol("cg")]
pub struct SyncStorageValue(pub Option<serde_json::Value>);

impl From<Option<serde_json::Value>> for SyncStorageValue {
    fn from(value: Option<serde_json::Value>) -> Self {
        Self(value)
    }
}
/// Timestamp referring to a date time, and is usually used to interact with the User's timeline.
/// Consider adding a [DevString] to track provenance of the timestamp?
#[protocol("cg")]
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct At {
    pub UNIX_SECS: i64,
}

impl From<cg_protocol::TimeStamp> for At {
    fn from(ts: cg_protocol::TimeStamp) -> Self {
        let secs = ts.to_unix_seconds();
        Self { UNIX_SECS: secs }
    }
}
impl From<chrono::DateTime<chrono::Utc>> for At {
    fn from(ts: chrono::DateTime<chrono::Utc>) -> Self {
        let secs = ts.timestamp();
        Self { UNIX_SECS: secs }
    }
}

impl At {
    pub fn to_timestamp(&self) -> cg_protocol::TimeStamp {
        cg_protocol::TimeStamp::from_unix_seconds(self.UNIX_SECS)
    }
    pub fn minus_days(&self, days: i64) -> Self {
        let secs = self.UNIX_SECS - days * 24 * 60 * 60;
        Self { UNIX_SECS: secs }
    }
    pub fn plus_days(&self, days: i64) -> Self {
        let secs = self.UNIX_SECS + days * 24 * 60 * 60;
        Self { UNIX_SECS: secs }
    }
}

#[protocol("cg")]
#[codegen(cg_imports = "Source")]
pub struct TimelineResult {
    /// Records
    pub r: Vec<RecordID>,
    /// Sources
    pub s: Vec<cg_protocol::view::Source>,
    /// Visited at
    pub v: At,
    /// Content of page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub m: Option<String>,
    /// Content of open graph
    #[serde(skip_serializing_if = "Option::is_none")]
    pub og: Option<WebsiteOpenGraphData>,
    /// URL
    pub u: String,
}

/// Store a highlight in IndexedDB
#[protocol("cg")]
#[codegen(fn = "store_highlight() -> StoreHighlightResult")]
pub struct StoreHighlightParams {
    /// URL of the page where this highlight was created
    pub page_url: String,
    /// The highlighted text
    pub text: String,
    /// Priority level (higher = more important)
    pub priority: i32,
    /// Related record IDs
    pub records: Vec<RecordID>,
    /// Optional summary text
    pub summary: Option<String>,
    /// Relevance flags
    pub personally_relevant: bool,
    pub contextually_relevant: bool,
    pub contextually_important: bool,
}

/// Result from storing a highlight
#[protocol("cg")]
pub struct StoreHighlightResult {
    /// The ID of the stored highlight
    pub highlight_id: String,
    /// Success message
    pub message: DevString,
}

/// Get highlights for a specific page URL
#[protocol("cg")]
#[codegen(fn = "get_highlights_by_url() -> GetHighlightsByUrlResult")]
pub struct GetHighlightsByUrlParams {
    /// URL of the page to get highlights for
    pub page_url: String,
}

/// A stored highlight returned from the database
#[protocol("cg")]
pub struct StoredHighlightData {
    /// The ID of the highlight
    pub highlight_id: String,
    /// The highlighted text
    pub text: String,
    /// Priority level (higher = more important)
    pub priority: i32,
    /// Related record IDs
    pub records: Vec<RecordID>,
    /// Optional summary text
    pub summary: Option<String>,
    /// Whether this highlight is personally relevant
    pub personally_relevant: bool,
    /// Whether this highlight is contextually relevant
    pub contextually_relevant: bool,
    /// Whether this highlight is contextually important
    pub contextually_important: bool,
    /// When this highlight was created
    pub created_at: Option<At>,
    /// When this highlight was last updated
    pub updated_at: Option<At>,
    /// When this highlight was archived (if archived)
    pub archived_at: Option<At>,
}

/// Result from getting highlights by URL
#[protocol("cg")]
pub struct GetHighlightsByUrlResult {
    /// List of highlights for the requested page
    pub highlights: Vec<StoredHighlightData>,
}
/// Expand the neighborhood around highlight records
#[protocol("cg")]
#[codegen(fn = "expand_highlight_neighborhood() -> HighlightNeighborhoodResult")]
pub struct ExpandHighlightNeighborhoodParams {
    /// The core highlight record IDs to expand around
    pub highlight_record_ids: Vec<RecordID>,
    /// The actual highlighted text for relevance scoring
    pub highlight_text: Option<String>,
    /// Maximum number of neighbor records to include (default: 33)
    pub max_neighbors: Option<usize>,
    /// Whether to include bridge atom information (default: true)
    pub include_bridge_atoms: Option<bool>,
    /// Whether to apply temporal weighting for recent activity (default: true)
    pub temporal_weighting: Option<bool>,
}

/// Result from expanding highlight neighborhood
#[protocol("cg")]
pub struct HighlightNeighborhoodResult {
    /// Original core record IDs that were expanded
    pub core_records: Vec<RecordID>,
    /// Highly relevant neighbor records (top-ranked)
    pub relevant_neighbors: Vec<RecordID>,
    /// Additional related neighbor records
    pub related_neighbors: Vec<RecordID>,
    /// Key bridge atoms that connect the records
    pub bridge_atoms: Vec<BridgeAtomInfo>,
    /// Summary of the expansion analysis
    pub expansion_summary: String,
    /// Total number of records considered in the neighborhood
    pub total_records_analyzed: usize,
    /// Relevance scoring summary (if highlight text was provided)
    pub relevance_summary: Option<String>,
}

/// Information about a bridge atom that connects records
#[protocol("cg")]
pub struct BridgeAtomInfo {
    /// Atom ID
    pub atom_id: u64,
    /// Atom type (EmailAddress, PersonName, etc.)
    pub atom_type: String,
    /// Display name if available from the index
    pub display_name: Option<String>,
    /// Bridge score (connectivity measure)
    pub bridge_score: f64,
    /// Number of core records this atom connects to
    pub core_connections: usize,
    /// Number of neighbor records this atom connects to
    pub neighbor_connections: usize,
}

/// Refresh tokens
#[protocol("cg")]
#[codegen(fn = "refresh_tokens() -> ()")]
pub struct RefreshTokensParams {}

/// Analyze the network structure using network analytics
#[protocol("cg")]
#[codegen(fn = "analyze_network() -> NetworkAnalysisResult")]
pub struct AnalyzeNetworkParams {
    /// Whether to test highlight analytics as part of the analysis
    pub test_highlights: bool,
    /// Export mode - if true, returns detailed JSON export instead of logging
    pub export_mode: bool,
}

/// Store behavioral signals for machine learning and user behavior analysis
#[protocol("cg")]
#[codegen(fn = "store_behavioral_signals() -> StoreBehavioralSignalsResult")]
pub struct StoreBehavioralSignalsParams {
    /// The behavioral signals to store
    pub signals: Vec<BehavioralSignalData>,
    /// Whether behavioral learning is fully enabled on the client
    #[serde(default)]
    pub enable_behavioral_learning: bool,
    /// Whether predicate-aware multi-lens clustering should run
    #[serde(default)]
    pub enable_semantic_multilens: bool,
}

/// Result from storing behavioral signals
#[protocol("cg")]
pub struct StoreBehavioralSignalsResult {
    /// Number of signals successfully stored
    pub signals_stored: usize,
    /// Success message
    pub message: DevString,
}

/// Data structure for behavioral signals
#[protocol("cg")]
pub struct BehavioralSignalData {
    /// Unique identifier for this signal
    pub signal_id: String,
    /// Type of behavioral signal
    pub signal_type: String,
    /// When this signal was generated
    pub timestamp: i64,
    /// Context information as JSON string
    pub context: String,
    /// User outcome as JSON string
    pub outcome: String,
}

/// Get behavioral guidance for enhancing page analysis
#[protocol("cg")]
#[codegen(fn = "get_behavioral_guidance_for_page() -> BehavioralGuidanceResult")]
pub struct BehavioralGuidanceParams {
    /// The domain to get behavioral guidance for
    pub page_domain: String,
    /// Current page URL for context
    pub page_url: String,
    /// Current time context (hour of day)
    pub current_hour: u8,
    /// Recent page sequence for pattern matching
    pub recent_domains: Vec<String>,
}

/// Temporal engagement patterns derived from behavioral learning
#[protocol("cg")]
pub struct BehavioralTemporalPatterns {
    /// Hourly engagement scores keyed by hour of day (0-23)
    pub hourly_engagement: HashMap<u8, f64>,
    /// Day-of-week engagement scores keyed by weekday (Monday = 0)
    pub day_of_week_engagement: HashMap<u8, f64>,
    /// Peak engagement hours sorted by preference
    pub peak_hours: Vec<u8>,
}

/// Behavioral guidance for improving LLM analysis and highlighting
#[protocol("cg")]
pub struct BehavioralGuidanceResult {
    /// Search terms that correlate with user engagement
    pub engaging_search_terms: Vec<String>,
    /// Topics that users typically ignore
    pub ignored_search_terms: Vec<String>,
    /// Optimal bridge atom density range for this domain
    pub optimal_bridge_atom_density: BridgeAtomDensityRange,
    /// Peak engagement time patterns
    pub peak_engagement_hours: Vec<u8>,
    /// Domain-specific engagement preferences
    pub domain_preferences: DomainEngagementPreferences,
    /// Prominence scoring guidance
    pub prominence_guidance: ProminenceGuidance,
    /// Confidence score for these recommendations (0.0-1.0)
    pub confidence: f64,
    /// Summary of the behavioral patterns found
    pub guidance_summary: String,
    /// Timeline-highlight correlations for enhanced prominence
    pub timeline_correlations: Vec<TimelineHighlightCorrelation>,
    /// Temporal engagement patterns for the current domain
    pub temporal_patterns: BehavioralTemporalPatterns,
    /// Recently learned project/entity clusters contributing to guidance
    pub project_clusters: Vec<BehavioralProjectClusterSummary>,
}

/// Behavioral cluster snapshot surfaced for guidance consumers
#[protocol("cg")]
pub struct BehavioralProjectClusterSummary {
    /// Stable cluster identifier derived from atom membership
    pub cluster_id: String,
    /// Human-readable label (top entities)
    pub label: String,
    /// Normalized strength score (0.0-1.0)
    pub strength: f64,
    /// Most representative entities in the cluster
    pub top_entities: Vec<BehavioralClusterEntity>,
    /// Representative records that anchor the cluster
    pub representative_records: Vec<BehavioralClusterRecord>,
    /// Time window that the snapshot covers
    pub time_window: BehavioralClusterTimeWindow,
    /// Source describing how the cluster was inferred (e.g., content vs. engagement-driven)
    pub source: String,
    /// Human-readable summary derived from existing metadata
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub summary: Option<String>,
    /// Predicate-aware semantic summary if available
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub semantic_summary: Option<BehavioralClusterSemanticSummary>,
    /// Derived geometry features summarising cohesion and periphery
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub geometry_features: Option<BehavioralClusterGeometryFeatures>,
}

/// Request the latest cognition plan summary from the background service
#[protocol("cg")]
#[codegen(fn = "get_cognition_plan() -> CognitionPlanResult")]
pub struct GetCognitionPlanParams {}

/// Request the latest cognition telemetry summary
#[protocol("cg")]
#[codegen(fn = "get_cognition_telemetry() -> CognitionTelemetrySnapshot")]
pub struct GetCognitionTelemetryParams {}

/// Fetch introspection diagnostics for the most recent or specified plan
#[protocol("cg")]
#[codegen(fn = "get_cognition_introspection() -> CognitionIntrospectionResult")]
pub struct GetCognitionIntrospectionParams {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub plan_id: Option<UID>,
}

/// Introspection response wrapping an optional report payload
#[protocol("cg")]
pub struct CognitionIntrospectionResult {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub report: Option<CognitionIntrospectionReport>,
}

/// Serialized introspection report capturing planner reasoning context
#[protocol("cg")]
pub struct CognitionIntrospectionReport {
    pub plan_id: UID,
    pub decision_trace: CognitionIntrospectionTrace,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_digest: Option<String>,
    pub belief_state: CognitionBeliefState,
    pub confidence_breakdown: CognitionConfidenceBreakdown,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternative_explanations: Vec<CognitionAlternativeExplanation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watchdog: Option<CognitionWatchdogSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_comparison: Option<CognitionNetworkComparison>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mechanistic: Vec<CognitionMechanisticExplanation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub temporal_counterfactuals: Vec<CognitionTemporalCounterfactual>,
}

/// Detailed reasoning trace associated with a plan
#[protocol("cg")]
pub struct CognitionIntrospectionTrace {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<CognitionIntrospectionStep>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_points: Vec<CognitionDecisionPoint>,
}

/// Single reasoning step recorded by the introspection engine
#[protocol("cg")]
pub struct CognitionIntrospectionStep {
    pub stage: String,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<f32>,
}

/// Planner decision point recorded during introspection
#[protocol("cg")]
pub struct CognitionDecisionPoint {
    pub description: String,
    pub chosen_branch: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alternative_branches: Vec<String>,
    pub reason_for_choice: String,
}

/// Counterfactual watchdog accuracy summary surfaced alongside introspection
#[protocol("cg")]
pub struct CognitionWatchdogSummary {
    pub samples: u64,
    pub mean_absolute_error: f32,
    pub mean_signed_error: f32,
    pub mean_squared_error: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_observation: Option<CognitionWatchdogObservation>,
}

/// Most recent watchdog observation used to compute accuracy aggregates
#[protocol("cg")]
pub struct CognitionWatchdogObservation {
    pub predicted_uplicg: f32,
    pub observed_reward: f32,
    pub risk_floor: f32,
    pub timestamp_ms: i64,
    pub absolute_error: f32,
}

#[protocol("cg")]
pub struct CognitionTemporalCounterfactual {
    pub node: String,
    pub lag_ms: i64,
    pub mean: f32,
    pub std_dev: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard_confidence: Option<f32>,
}

/// Request the latest causal graph snapshot produced by cognition
#[protocol("cg")]
#[codegen(fn = "get_cognition_causal_graph() -> CognitionCausalGraphResult")]
pub struct GetCognitionCausalGraphParams {}

/// Causal graph response payload
#[protocol("cg")]
pub struct CognitionCausalGraphResult {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub graph: Option<CognitionCausalGraph>,
}

/// Retrieve the latest cognition option statistics
#[protocol("cg")]
#[codegen(fn = "get_cognition_options() -> CognitionOptionSetResult")]
pub struct GetCognitionOptionsParams {}

/// Snapshot of cognition options and execution metrics
#[protocol("cg")]
pub struct CognitionOptionSetResult {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<CognitionOptionEntry>,
}

/// Individual option entry with summary statistics
#[protocol("cg")]
pub struct CognitionOptionEntry {
    pub name: String,
    pub executions: u64,
    pub success_ratio: f32,
    pub average_reward: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_schema: Option<CognitionTemporalCausalSchema>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ltl_signatures: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub temporal_guards: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mechanistic_explanations: Vec<CognitionMechanisticExplanation>,
}

/// Retrieve the cognition Pareto frontier snapshot
#[protocol("cg")]
#[codegen(fn = "get_cognition_pareto() -> CognitionParetoResult")]
pub struct GetCognitionParetoParams {}

/// Pareto frontier for recent planner decisions
#[protocol("cg")]
pub struct CognitionParetoResult {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frontier: Vec<CognitionParetoPoint>,
}

/// Pareto frontier entry capturing action encoding and reward vector
#[protocol("cg")]
pub struct CognitionParetoPoint {
    pub action: Vec<f32>,
    pub reward: CognitionMultiObjectiveReward,
}

/// Multi-objective reward decomposition
#[protocol("cg")]
pub struct CognitionMultiObjectiveReward {
    pub information_gain: f32,
    pub time_cost: f32,
    pub cognitive_load: f32,
    pub success_probability: f32,
}

/// Serialized DAG describing causal dependencies between planner metrics
#[protocol("cg")]
pub struct CognitionCausalGraph {
    pub variables: Vec<String>,
    pub edges: Vec<CognitionCausalEdge>,
    pub samples_used: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iterations_to_converge: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub final_constraint_value: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at_ms: Option<i64>,
}

/// Weighted edge within the cognition causal graph
#[protocol("cg")]
pub struct CognitionCausalEdge {
    pub from: usize,
    pub to: usize,
    pub weight: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lag_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ltl_guard: Option<String>,
}

/// Aggregated belief state derived from perception and personalization
#[protocol("cg")]
pub struct CognitionBeliefState {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub intent_beliefs: Vec<CognitionIntentBelief>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub world_beliefs: Vec<CognitionWorldBelief>,
    pub meta_beliefs: CognitionMetaBeliefs,
}

/// Confidence assigned to an individual intent hypothesis
#[protocol("cg")]
pub struct CognitionIntentBelief {
    pub label: String,
    pub strength: f32,
    pub confidence: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

/// Summary of perceived environmental context
#[protocol("cg")]
pub struct CognitionWorldBelief {
    pub belief: String,
    pub confidence: f32,
    pub source: String,
}

/// Meta-learning posture relating to epistemic certainty and historical accuracy
#[protocol("cg")]
pub struct CognitionMetaBeliefs {
    pub epistemic_uncertainty: f32,
    pub domain_coverage: f32,
    pub historical_accuracy: f32,
}

/// Confidence breakdown across planning stages
#[protocol("cg")]
pub struct CognitionConfidenceBreakdown {
    pub overall_confidence: f32,
    pub intent_confidence: f32,
    pub planning_confidence: f32,
    pub execution_confidence: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uncertainty_sources: Vec<CognitionUncertaintySource>,
}

/// Contribution of a component to the residual uncertainty budget
#[protocol("cg")]
pub struct CognitionUncertaintySource {
    pub component: String,
    pub contribution: f32,
    pub reason: String,
}

/// Alternative interpretation considered during introspection
#[protocol("cg")]
pub struct CognitionAlternativeExplanation {
    pub explanation: String,
    pub likelihood: f32,
    pub why_rejected: String,
}

/// Mechanistic explanation generated from the causal schemas
#[protocol("cg")]
pub struct CognitionMechanisticExplanation {
    pub target: String,
    pub headline: String,
    pub confidence: f32,
    pub schema_type: String,
    pub likelihood: f32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causal_path: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub influences: Vec<CognitionMechanisticInfluence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_support: Option<f32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub temporal_intents: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curvature_factor: Option<f32>,
}

/// Individual influence contributing to a mechanistic explanation
#[protocol("cg")]
pub struct CognitionMechanisticInfluence {
    pub parent: String,
    pub weight: f64,
    pub contribution: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lag_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ltl_guard: Option<String>,
}

/// Cognition plan response containing optional plan details
#[protocol("cg")]
pub struct CognitionPlanResult {
    pub plan: Option<CognitionPlanSummary>,
}

/// High-level summary of the current cognition plan
#[protocol("cg")]
pub struct CognitionPlanSummary {
    pub plan_id: u64,
    pub context: CognitionPlanContext,
    pub guardrails: CognitionPlanGuardrails,
    #[serde(default)]
    pub uncertainty_budget: CognitionUncertaintyBudget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<CognitionPlanValidation>,
    #[serde(default)]
    pub policy_actions: Vec<CognitionPolicyAction>,
    pub steps: Vec<CognitionPlanStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_model: Option<CognitionWorldModelSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_comparison: Option<CognitionNetworkComparison>,
}

/// Context metadata describing when and how the plan was produced
#[protocol("cg")]
pub struct CognitionPlanContext {
    pub captured_at_millis: i64,
    pub config: CognitionPlanConfig,
}

/// Snapshot of cognition feature flags used when the plan was generated
#[protocol("cg")]
pub struct CognitionPlanConfig {
    pub enable_cognition: bool,
    pub enable_behavioral_learning: bool,
    pub enable_semantic_multilens: bool,
    pub enable_idle_trainer: bool,
    #[serde(default)]
    pub prefer_unified_network: bool,
    #[serde(default)]
    pub tuning: CognitionPlanTuning,
    #[serde(default)]
    pub idle_trainer: CognitionIdleTrainerConfig,
    #[serde(default)]
    pub multi_objective_profile: CognitionMultiObjectiveProfile,
    #[serde(default)]
    pub unified_rollout: CognitionUnifiedRolloutConfig,
    #[serde(default)]
    pub enable_curiosity: bool,
    #[serde(default)]
    pub curiosity_weight: f32,
    #[serde(default)]
    pub curiosity_budget_ms: u32,
}

#[protocol("cg")]
#[derive(Default)]
pub struct CognitionUnifiedRolloutConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub unified_percentage: f32,
    #[serde(default)]
    pub evaluation_window: u32,
    #[serde(default)]
    pub min_sessions: u32,
    #[serde(default)]
    pub promote_threshold: f32,
    #[serde(default)]
    pub demote_threshold: f32,
}

/// Planner idle training telemetry exposed to clients
#[protocol("cg")]
#[derive(Default)]
pub struct CognitionPlannerIdleTelemetry {
    pub total_runs: u64,
    pub successful_runs: u64,
    pub last_batches_requested: u64,
    pub last_statistics_queued: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_trained_at_ms: Option<i64>,
    pub last_run_succeeded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_value_stddev_mean: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_value_stddev_max: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_policy_stddev_mean: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_policy_stddev_max: Option<f32>,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub window_sample_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_value_stddev_mean: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_value_stddev_max: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_policy_stddev_mean: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_policy_stddev_max: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_network_value_delta: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_network_policy_l1: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_network_policy_kl: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_network_value_delta_mean: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_network_policy_l1_mean: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_network_policy_kl_mean: Option<f32>,
}

/// Meta-learning telemetry metrics surfaced for dashboards
#[protocol("cg")]
#[derive(Default)]
pub struct CognitionMetaTelemetry {
    pub online_updates: u64,
    pub idle_batches: u64,
    pub avg_reward: f32,
    pub recent_dricg: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_trained_at_ms: Option<u64>,
    pub htn_priority_changes: u64,
    pub llm_budget_adjustments: u64,
}

/// Aggregated cognition telemetry snapshot (planner + meta)
#[protocol("cg")]
pub struct CognitionTelemetrySnapshot {
    pub planner: CognitionPlannerIdleTelemetry,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<CognitionMetaTelemetry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub self_improvement: Option<CognitionSelfImprovementSnapshot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metrics: Option<CognitionMetricsSnapshot>,
}

#[protocol("cg")]
#[derive(Default)]
pub struct CognitionMetricsSnapshot {
    #[serde(default)]
    pub ltl_evaluations: u64,
    #[serde(default)]
    pub ltl_satisfied: u64,
    pub ltl_satisfaction_rate: f64,
    #[serde(default)]
    pub curvature_anomalies: u64,
    #[serde(default)]
    pub self_improvement_proposals: u64,
    #[serde(default)]
    pub temporal_lag_violations: u64,
}

#[protocol("cg")]
pub struct CognitionSelfImprovementSnapshot {
    #[serde(default)]
    pub proposals: Vec<CognitionSelfImprovementProposal>,
    #[serde(default)]
    pub history: Vec<CognitionImprovementOutcome>,
}

#[protocol("cg")]
pub struct CognitionSelfImprovementProposal {
    pub id: u64,
    pub reason: String,
    pub architecture: String,
    pub created_at_ms: f64,
}

#[protocol("cg")]
pub struct CognitionImprovementOutcome {
    pub proposal_id: u64,
    pub accepted: bool,
    pub reward_delta: f32,
    pub timestamp_ms: f64,
}

#[protocol("cg")]
#[derive(Default)]
pub struct CognitionPlanTuning {
    #[serde(default)]
    pub scoring: CognitionScoringConfig,
    #[serde(default)]
    pub sequential: CognitionSequentialConfig,
    #[serde(default)]
    pub analytics: CognitionAnalyticsConfig,
}

#[protocol("cg")]
pub enum CognitionMultiObjectiveProfile {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "speed_focused")]
    SpeedFocused,
    #[serde(rename = "thoroughness_focused")]
    ThoroughnessFocused,
    #[serde(rename = "user_friendly")]
    UserFriendly,
}

impl Default for CognitionMultiObjectiveProfile {
    fn default() -> Self {
        Self::Default
    }
}

#[protocol("cg")]
pub struct CognitionScoringConfig {
    pub max_intents: u32,
    pub max_highlight_interactions: u32,
    pub highlight_keyword_bonus: f32,
    pub record_keyword_bonus: f32,
    pub record_prior_weight: f32,
    pub cluster_prior_weight: f32,
}

#[protocol("cg")]
pub struct CognitionSequentialConfig {
    pub predicted_next_events: u32,
    pub transition_motif_cap: u32,
}

#[protocol("cg")]
pub struct CognitionAnalyticsConfig {
    pub temporal_refresh_ratio: f32,
    pub temporal_refresh_interval_secs: u32,
    pub predicate_refresh_ratio: f32,
    pub predicate_refresh_interval_secs: u32,
    pub dricg_alert_threshold: f32,
}

#[protocol("cg")]
#[derive(Default)]
pub struct CognitionIdleTrainerConfig {
    pub idle_detection_secs: u32,
    pub alarm_period_minutes: f32,
    pub cooldown_minutes: f32,
    pub max_batches_per_run: u32,
    #[serde(default)]
    pub max_llm_per_idle: u32,
    #[serde(default)]
    pub meta_batches_per_run: u32,
}

impl Default for CognitionScoringConfig {
    fn default() -> Self {
        Self {
            max_intents: 24,
            max_highlight_interactions: 5,
            highlight_keyword_bonus: 0.05,
            record_keyword_bonus: 0.05,
            record_prior_weight: 1.0 / 3.0,
            cluster_prior_weight: 0.5,
        }
    }
}

impl Default for CognitionSequentialConfig {
    fn default() -> Self {
        Self {
            predicted_next_events: 16,
            transition_motif_cap: 32,
        }
    }
}

impl Default for CognitionAnalyticsConfig {
    fn default() -> Self {
        Self {
            temporal_refresh_ratio: 0.1,
            temporal_refresh_interval_secs: 30,
            predicate_refresh_ratio: 0.1,
            predicate_refresh_interval_secs: 30,
            dricg_alert_threshold: 0.3,
        }
    }
}

/// Guardrail summary attached to the plan
#[protocol("cg")]
pub struct CognitionPlanGuardrails {
    pub expected_uplicg: f32,
    pub risk_floor: f32,
    pub llm_budget: Option<u32>,
    #[serde(default)]
    pub temporal_satisfaction: Option<f32>,
    #[serde(default)]
    pub uncertainty_total: Option<f32>,
    #[serde(default)]
    pub uncertainty_epistemic: Option<f32>,
    #[serde(default)]
    pub uncertainty_aleatoric: Option<f32>,
    #[serde(default)]
    pub uncertainty_distributional: Option<f32>,
}

/// Summary statistics for world model rollouts captured during planning
#[protocol("cg")]
#[derive(Default)]
pub struct CognitionWorldModelSnapshot {
    pub evaluated_actions: u32,
    pub average_projection: f32,
    pub max_projection: f32,
}

/// Aggregated uncertainty metrics for the plan generation pipeline
#[protocol("cg")]
#[derive(Default)]
pub struct CognitionUncertaintyBudget {
    #[serde(default)]
    pub perception: f32,
    #[serde(default)]
    pub intent: f32,
    #[serde(default)]
    pub counterfactual: f32,
    #[serde(default)]
    pub planning: f32,
    #[serde(default)]
    pub total: f32,
    #[serde(default)]
    pub exploration_ratio: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epistemic: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aleatoric: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distributional: Option<f32>,
}

/// Result of adversarial validation for a plan
#[protocol("cg")]
pub struct CognitionPlanValidation {
    pub flagged: bool,
    pub risk_score: f32,
    pub uncertainty_score: f32,
    pub reasons: Vec<String>,
}

/// Individual planned step
#[protocol("cg")]
pub struct CognitionPlanStep {
    pub action: String,
    pub description: Option<String>,
    pub cost: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<CognitionPlanStepMetadata>,
}

#[protocol("cg")]
#[derive(Default)]
pub struct CognitionPlanStepMetadata {
    #[serde(default)]
    pub htn_priority: f32,
    #[serde(default)]
    pub attention_focus: f32,
    #[serde(default)]
    pub sequential_support: f32,
    #[serde(default)]
    pub epistemic: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_order: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub justification: Option<String>,
    #[serde(default)]
    pub curiosity: f32,
    #[serde(default)]
    pub personalization_bias: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_causal: Option<CognitionTemporalCausalSchema>,
}

#[protocol("cg")]
#[derive(Default)]
pub struct CognitionTemporalCausalSchema {
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ltl_precondition: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub causal_parents: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_support: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curvature_factor: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal_lag_ms: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_intents: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

#[protocol("cg")]
#[derive(Default)]
pub struct CognitionNetworkComparison {
    pub primary: String,
    pub tiny_value: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unified_value: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value_delta: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_l1: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_kl: Option<f32>,
}

#[protocol("cg")]
#[derive(Default)]
pub struct CognitionPolicyAction {
    pub adapter: String,
    pub kind: String,
    pub confidence: f32,
    pub payload: String,
    pub rationale: String,
}

/// Derived geometry features exposed alongside project cluster summaries
#[protocol("cg")]
pub struct BehavioralClusterGeometryFeatures {
    /// Density-weighted cohesion (0.0-1.0)
    pub cohesion_score: f64,
    /// Average radius of the core exemplars
    pub core_radius: f64,
    /// Ratio of periphery spread relative to overall span (0.0-1.0)
    pub periphery_ratio: f64,
    /// Number of exemplars captured for this geometry snapshot
    pub exemplar_count: u32,
    /// Up to three exemplar IDs closest to the core
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub core_exemplars: Vec<String>,
    /// Up to three exemplar IDs on the periphery
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub periphery_exemplars: Vec<String>,
}

/// Simplified entity summary for behavioral clusters
#[protocol("cg")]
pub struct BehavioralClusterEntity {
    /// Network atom identifier
    pub atom_id: UID,
    /// Display label for the entity
    pub display_name: String,
    /// Underlying entity type (person, company, project, ...)
    pub entity_type: String,
    /// Normalized importance within the cluster
    pub weight: f64,
}

/// Representative record summary for behavioral clusters
#[protocol("cg")]
pub struct BehavioralClusterRecord {
    /// Record identifier
    pub record_id: UID,
    /// Record relevance within the cluster (0.0-1.0)
    pub relevance: f64,
    // Removed title and view_url for storage efficiency - can be looked up by record_id
}

/// Time window associated with a behavioral cluster snapshot
#[protocol("cg")]
pub struct BehavioralClusterTimeWindow {
    /// Start timestamp for the cluster window
    pub start: cg_protocol::TimeStamp,
    /// End timestamp for the cluster window
    pub end: cg_protocol::TimeStamp,
}

/// Semantic summary for behavioral clusters
#[protocol("cg")]
pub struct BehavioralClusterSemanticSummary {
    pub cluster_type: BehavioralClusterSemanticType,
    pub strengths: BehavioralClusterSemanticStrengths,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lenses: Vec<BehavioralClusterLensSummary>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub dominant_lens: Option<BehavioralClusterSemanticLens>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub explanation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub geometry: Option<BehavioralClusterGeometrySummary>,
}

/// Strength components for semantic clustering
#[protocol("cg")]
pub struct BehavioralClusterSemanticStrengths {
    pub ownership: f64,
    pub hierarchy: f64,
    pub tagging: f64,
    pub temporal: f64,
    pub bridge: f64,
}

#[protocol("cg")]
pub struct BehavioralClusterLensSummary {
    pub lens: BehavioralClusterSemanticLens,
    pub strength: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exemplars: Vec<UID>,
}

/// Optional geometry metadata for a behavioral cluster.
#[protocol("cg")]
pub struct BehavioralClusterGeometrySummary {
    /// Largest distance between cluster centroid and member exemplars.
    pub max_radius: f64,
    /// Average radius from centroid across exemplars.
    pub mean_radius: f64,
    /// Normalised density metric derived from radius variance.
    pub density: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exemplar_coords: Vec<BehavioralClusterGeometryExemplar>,
}

/// Coordinates for exemplar atoms used in geometry summaries.
#[protocol("cg")]
pub struct BehavioralClusterGeometryExemplar {
    pub atom_id: UID,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[protocol("cg")]
#[serde(rename_all = "snake_case")]
pub enum BehavioralClusterSemanticLens {
    Work,
    Social,
    Hierarchical,
    Temporal,
    Meta,
}

/// Semantic cluster category
#[protocol("cg")]
#[serde(rename_all = "snake_case")]
pub enum BehavioralClusterSemanticType {
    Unknown,
    Work,
    Social,
    Hierarchical,
    Temporal,
    Meta,
}

/// Bridge atom density range preference
#[protocol("cg")]
pub struct BridgeAtomDensityRange {
    /// Minimum effective bridge atom density
    pub min_density: f64,
    /// Maximum effective bridge atom density
    pub max_density: f64,
    /// Optimal bridge atom density
    pub optimal_density: f64,
}

/// Domain-specific engagement preferences
#[protocol("cg")]
pub struct DomainEngagementPreferences {
    /// Average engagement score for this domain (0.0-1.0)
    pub avg_engagement_score: f64,
    /// Preferred content types that drive engagement
    pub preferred_content_types: Vec<String>,
    /// Network characteristics that work well for this domain
    pub successful_network_patterns: Vec<String>,
    /// Timing preferences (hours when users are most engaged)
    pub optimal_timing_hours: Vec<u8>,
}

/// Guidance for intelligent prominence calculation
#[protocol("cg")]
pub struct ProminenceGuidance {
    /// Thresholds for personal relevance scoring
    pub personal_relevance_threshold: f64,
    /// Thresholds for contextual relevance scoring
    pub contextual_relevance_threshold: f64,
    /// Thresholds for importance scoring
    pub importance_threshold: f64,
    /// Color preferences based on user engagement patterns
    pub color_preferences: Vec<ColorPreference>,
}

/// Color preference based on behavioral learning
#[protocol("cg")]
pub struct ColorPreference {
    /// Color identifier (e.g. "yellow", "blue", "green")
    pub color: String,
    /// Engagement score for this color (0.0-1.0)
    pub engagement_score: f64,
    /// Contexts where this color works best
    pub optimal_contexts: Vec<String>,
}

/// Parameters for calculating intelligent prominence based on network analysis and behavioral patterns
#[protocol("cg")]
#[codegen(fn = "calculate_intelligent_prominence() -> IntelligentProminenceResult")]
pub struct CalculateProminenceParams {
    /// Network analysis data for the highlight
    pub network_analysis: ProminenceNetworkContext,
    /// Behavioral context from user patterns
    pub behavioral_context: ProminenceBehavioralContext,
    /// Page and temporal context
    pub page_context: ProminencePageContext,
    /// Text analysis context
    pub text_context: ProminenceTextContext,
}

/// Network analysis context for prominence calculation
#[protocol("cg")]
pub struct ProminenceNetworkContext {
    /// Number of bridge atoms found in the neighborhood
    pub bridge_atoms_count: usize,
    /// Bridge atom density score (0.0-1.0)
    pub bridge_atom_density: f64,
    /// Citation relevance score based on records found
    pub citation_relevance_score: f64,
    /// Network connectivity strength
    pub connectivity_strength: f64,
    /// Community cohesion score if available
    pub community_cohesion: Option<f64>,
}

/// Behavioral learning context for prominence calculation
#[protocol("cg")]
pub struct ProminenceBehavioralContext {
    /// How engaged the user typically is with this domain (0.0-1.0)
    pub domain_engagement_score: f64,
    /// Whether this matches learned engagement patterns
    pub matches_engagement_patterns: bool,
    /// Peak engagement time adjustment factor
    pub temporal_engagement_factor: f64,
    /// User's color preferences with scores
    pub color_preferences: Vec<ColorPreference>,
    /// Historical prominence patterns that worked
    pub successful_prominence_patterns: Vec<String>,
}

/// Page and temporal context for prominence calculation
#[protocol("cg")]
pub struct ProminencePageContext {
    /// Domain of the current page
    pub page_domain: String,
    /// Type of content (article, email, document, etc.)
    pub content_type: String,
    /// Current hour for temporal weighting
    pub current_hour: u8,
    /// Day of week for pattern matching
    pub day_of_week: u8,
    /// Recent page navigation patterns
    pub recent_page_sequence: Vec<String>,
}

/// Text analysis context for prominence calculation
#[protocol("cg")]
pub struct ProminenceTextContext {
    /// Length of the highlighted text
    pub text_length: usize,
    /// Number of search terms that matched
    pub matched_search_terms: usize,
    /// Total search terms attempted
    pub total_search_terms: usize,
    /// Text complexity or importance score if available
    pub text_importance_score: Option<f64>,
    /// Whether the text contains key entities (names, organizations, etc.)
    pub contains_key_entities: bool,
}

/// Result of intelligent prominence calculation
#[protocol("cg")]
pub struct IntelligentProminenceResult {
    /// Whether this highlight is personally relevant to the user
    pub personally_relevant: bool,
    /// Personal relevance confidence score (0.0-1.0)
    pub personal_relevance_score: f64,
    /// Whether this highlight is contextually relevant to the current page/topic
    pub contextually_relevant: bool,
    /// Contextual relevance confidence score (0.0-1.0)
    pub contextual_relevance_score: f64,
    /// Whether this highlight is contextually important (significant/noteworthy)
    pub contextually_important: bool,
    /// Contextual importance confidence score (0.0-1.0)
    pub importance_score: f64,
    /// Recommended color for this highlight based on learned preferences
    pub suggested_color: ProminenceColor,
    /// Overall prominence confidence (how sure we are about these scores)
    pub overall_confidence: f64,
    /// Explanation of how this prominence was calculated
    pub calculation_explanation: String,
    /// Behavioral patterns that influenced this calculation
    pub influencing_patterns: Vec<String>,
}

/// Color information with scoring rationale
#[protocol("cg")]
pub struct ProminenceColor {
    /// The color identifier (e.g. "yellow", "blue", "green")
    pub color_id: String,
    /// RGB values for display
    pub rgb_values: ColorRGB,
    /// Why this color was chosen
    pub selection_reason: String,
    /// Confidence in this color choice (0.0-1.0)
    pub color_confidence: f64,
}

/// RGB color values
#[protocol("cg")]
pub struct ColorRGB {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha/opacity component (0.0-1.0)
    pub a: f64,
}

/// Result from network analysis
#[protocol("cg")]
pub struct NetworkAnalysisResult {
    /// Summary of the analysis
    pub summary: String,
    /// Bridge atoms (atom_id, score) - rare but highly connected atoms
    pub bridge_atoms: Vec<(u64, f64)>,
    /// Central records (record_id, score) - highly connected records
    pub central_records: Vec<(u64, f64)>,
    /// Influential atoms (atom_id, score) - globally important atoms
    pub influential_atoms: Vec<(u64, f64)>,
    /// Highlight analysis results if requested
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlight_analysis: Option<String>,
    /// Detailed export data (only populated in export mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_data: Option<NetworkAnalysisExportData>,
}

/// Archive a highlight in IndexedDB (socg delete)
#[protocol("cg")]
#[codegen(fn = "archive_highlight() -> ArchiveHighlightResult")]
pub struct ArchiveHighlightParams {
    /// The ID of the highlight to archive
    pub highlight_id: String,
}

/// Result from archiving a highlight
#[protocol("cg")]
pub struct ArchiveHighlightResult {
    /// Success message
    pub message: DevString,
}

/// Detailed network analysis data for export
#[protocol("cg")]
pub struct NetworkAnalysisExportData {
    /// Analysis metadata
    pub metadata: NetworkAnalysisMetadata,
    /// Analyzed records with details
    pub records: Vec<NetworkAnalysisRecord>,
    /// Analyzed atoms with details
    pub atoms: Vec<NetworkAnalysisAtom>,
    /// Community structure
    pub communities: Vec<NetworkCommunity>,
    /// Network statistics
    pub network_stats: NetworkStatistics,
    /// Temporal analysis results
    pub temporal_insights: TemporalAnalysisInsights,
    /// Advanced graph connectivity metrics
    pub graph_metrics: AdvancedGraphMetrics,
}

/// Analysis metadata
#[protocol("cg")]
pub struct NetworkAnalysisMetadata {
    /// When the analysis was performed
    pub timestamp: At,
    /// Total analysis duration in milliseconds
    pub duration_ms: u64,
    /// Data source (e.g., "real_data" or "demo_data")
    pub data_source: String,
    /// Analysis parameters used
    pub parameters: AnalyzeNetworkParams,
    /// Socgware version and build info
    pub version_info: AnalysisVersionInfo,
}

/// Version and build information for the analysis
#[protocol("cg")]
pub struct AnalysisVersionInfo {
    /// Extension version
    pub extension_version: String,
    /// Network analytics crate version
    pub analytics_version: String,
    /// Algorithms used in this analysis
    pub algorithms_used: Vec<String>,
}

/// Record in network analysis export
#[protocol("cg")]
pub struct NetworkAnalysisRecord {
    /// Record ID
    pub id: u64,
    /// Record title
    pub title: String,
    /// Record source class (Message, Document, etc.)
    pub source_class: String,
    /// Degree centrality (number of connected atoms)
    pub degree_centrality: f64,
    /// Centrality score
    pub centrality_score: f64,
    /// Community ID this record belongs to
    pub community_id: Option<u64>,
    /// Record creation timestamp if available
    pub created_at: Option<At>,
    /// Source domain (Gmail, Drive, etc.)
    pub source_domain: Option<String>,
}

/// Atom in network analysis export
#[protocol("cg")]
pub struct NetworkAnalysisAtom {
    /// Atom ID
    pub id: u64,
    /// Atom type (EmailAddress, PersonName, etc.)
    pub atom_type: String,
    /// Display name if available
    pub display_name: Option<String>,
    /// Bridge score (if it's a bridge atom)
    pub bridge_score: Option<f64>,
    /// Influence score (eigenvector centrality)
    pub influence_score: f64,
    /// PageRank centrality score
    pub pagerank_score: Option<f64>,
    /// Closeness centrality score
    pub closeness_score: Option<f64>,
    /// Degree centrality (simple connection count)
    pub degree_centrality: f64,
    /// Community ID this atom belongs to
    pub community_id: Option<u64>,
    /// Temporal trend scores across different time scales
    pub temporal_scores: TemporalScores,
}

/// Temporal analysis scores for an atom
#[protocol("cg")]
pub struct TemporalScores {
    /// Immediate trending score (last few hours)
    pub immediate_trend: Option<f64>,
    /// Recent trending score (last few days)
    pub recent_trend: Option<f64>,
    /// Background trending score (longer term patterns)
    pub background_trend: Option<f64>,
}

/// Community in network analysis
#[protocol("cg")]
pub struct NetworkCommunity {
    /// Community ID
    pub id: u64,
    /// Atom IDs in this community
    pub atom_ids: Vec<u64>,
    /// Record IDs connected to this community
    pub connected_record_ids: Vec<u64>,
    /// Community size (number of atoms)
    pub size: usize,
    /// Community description
    pub description: String,
    /// Modularity score for this community
    pub modularity_score: Option<f64>,
    /// Most central atoms in this community
    pub central_atoms: Vec<u64>,
    /// Representative atom types in this community
    pub dominant_atom_types: Vec<String>,
    /// Cross-community connection strength
    pub external_connectivity: f64,
}

/// Network statistics
#[protocol("cg")]
pub struct NetworkStatistics {
    /// Total number of records
    pub total_records: usize,
    /// Total number of atoms
    pub total_atoms: usize,
    /// Total number of edges
    pub total_edges: usize,
    /// Network density (0.0 to 1.0)
    pub density: f64,
    /// Average degree (connections per node)
    pub average_degree: f64,
    /// Clustering coefficient
    pub clustering_coefficient: f64,
    /// Atom type distribution
    pub atom_type_distribution: std::collections::HashMap<String, usize>,
    /// Source domain distribution
    pub source_domain_distribution: std::collections::HashMap<String, usize>,
}

/// Temporal analysis insights
#[protocol("cg")]
pub struct TemporalAnalysisInsights {
    /// Top trending atoms in immediate timeframe
    pub immediate_trending: Vec<TrendingAtomScore>,
    /// Top trending atoms in recent timeframe
    pub recent_trending: Vec<TrendingAtomScore>,
    /// Top trending atoms in background timeframe
    pub background_trending: Vec<TrendingAtomScore>,
    /// Temporal patterns detected
    pub patterns: TemporalPatternAnalysis,
}

/// A trending atom with its temporal score
#[protocol("cg")]
pub struct TrendingAtomScore {
    /// Atom ID
    pub atom_id: u64,
    /// Trending score
    pub score: f64,
    /// Display name if available
    pub display_name: Option<String>,
    /// Atom type
    pub atom_type: String,
    /// Growth rate compared to baseline
    pub growth_rate: Option<f64>,
    /// Recent mention count
    pub recent_mentions: u32,
}

/// Temporal pattern analysis results
#[protocol("cg")]
pub struct TemporalPatternAnalysis {
    /// Peak activity periods detected
    pub peak_periods: Vec<ActivityPeriod>,
    /// Seasonal patterns detected
    pub seasonal_patterns: Vec<String>,
    /// Trend direction indicators
    pub overall_trends: std::collections::HashMap<String, f64>,
}

/// A period of peak activity
#[protocol("cg")]
pub struct ActivityPeriod {
    /// Start of the period
    pub start_time: At,
    /// End of the period
    pub end_time: At,
    /// Activity intensity score
    pub intensity: f64,
    /// Description of the activity
    pub description: String,
}

/// Advanced graph connectivity metrics
#[protocol("cg")]
pub struct AdvancedGraphMetrics {
    /// Atom projection connectivity metrics
    pub atom_connectivity: f64,
    /// Record projection connectivity metrics
    pub record_connectivity: f64,
    /// Overall network density
    pub overall_density: f64,
    /// Number of edges in atom projection
    pub atom_projection_edges: usize,
    /// Number of edges in record projection
    pub record_projection_edges: usize,
    /// Network diameter (longest shortest path)
    pub network_diameter: Option<f64>,
    /// Average path length
    pub average_path_length: Option<f64>,
    /// Number of connected components
    pub connected_components: usize,
    /// Giant component size ratio
    pub giant_component_ratio: Option<f64>,
    /// Small-world characteristics
    pub small_world_metrics: SmallWorldMetrics,
}

/// Small-world network characteristics
#[protocol("cg")]
pub struct SmallWorldMetrics {
    /// Small-world coefficient
    pub small_world_coefficient: Option<f64>,
    /// Average clustering compared to random network
    pub clustering_ratio: Option<f64>,
    /// Average path length compared to random network
    pub path_length_ratio: Option<f64>,
    /// Whether the network exhibits small-world properties
    pub is_small_world: bool,
}

// === Step 4: Behavioral Network Analytics Types ===

/// Analyze network with behavioral intelligence for Step 4: Complete Learning Loop
#[protocol("cg")]
#[codegen(fn = "analyze_behavioral_network() -> BehavioralNetworkAnalysisResult")]
pub struct BehavioralNetworkAnalysisParams {
    /// Domain for domain-specific behavioral patterns
    pub domain: Option<String>,
    /// Include cross-system intelligence analysis
    pub include_cross_system_analysis: bool,
    /// Include adaptive recommendations
    pub include_recommendations: bool,
    /// Maximum number of behavioral insights to return
    pub max_insights: Option<usize>,
}

/// Result of behavioral network analysis integrating learning with network analysis
#[protocol("cg")]
pub struct BehavioralNetworkAnalysisResult {
    /// Behavioral-enhanced bridge atoms with engagement weights
    pub enhanced_bridge_atoms: Vec<BehavioralCentralityScore>,
    /// Network paths that correlate with user engagement
    pub engaging_network_paths: Vec<NetworkPath>,
    /// Cross-system intelligence insights
    pub cross_system_intelligence: CrossSystemIntelligence,
    /// Adaptive recommendations for improving analysis
    pub adaptive_recommendations: AdaptiveRecommendations,
    /// Analysis summary
    pub summary: String,
    /// Whether behavioral learning is active and effective
    pub learning_active: bool,
}

/// Network centrality score enhanced with behavioral intelligence
#[protocol("cg")]
pub struct BehavioralCentralityScore {
    /// Atom ID
    pub atom_id: String,
    /// Base network centrality score
    pub base_centrality: f64,
    /// Behavioral engagement weight (0.0-1.0)
    pub behavioral_weight: f64,
    /// Combined behavioral centrality score
    pub behavioral_centrality: f64,
    /// Explanation of why this atom is behaviorally significant
    pub behavioral_explanation: String,
    /// Engagement patterns that influenced this score
    pub influencing_patterns: Vec<String>,
}

/// Network path that correlates with user engagement
#[protocol("cg")]
pub struct NetworkPath {
    /// Atom IDs in the path
    pub path_atom_ids: Vec<String>,
    /// Record IDs connected by this path
    pub connected_record_ids: Vec<String>,
    /// Engagement correlation score (0.0-1.0)
    pub engagement_correlation: f64,
    /// Why this path is significant for user engagement
    pub significance_explanation: String,
}

/// Cross-system intelligence from behavioral + network + prominence integration
#[protocol("cg")]
pub struct CrossSystemIntelligence {
    /// Correlations between network analysis and behavioral patterns
    pub network_behavioral_correlations: HashMap<String, f64>,
    /// Prominence patterns that predict network importance
    pub prominence_network_predictions: Vec<ProminenceNetworkPrediction>,
    /// LLM search terms that correlate with network connectivity
    pub search_network_correlations: HashMap<String, f64>,
    /// Cross-domain intelligence transfer patterns
    pub domain_transfer_patterns: Vec<String>,
}

/// Prediction about network importance based on prominence patterns
#[protocol("cg")]
pub struct ProminenceNetworkPrediction {
    /// Predicted network centrality based on prominence patterns
    pub predicted_centrality: f64,
    /// Confidence in this prediction (0.0-1.0)
    pub prediction_confidence: f64,
    /// Behavioral patterns supporting this prediction
    pub supporting_patterns: Vec<String>,
    /// Network features that correlate with prominence
    pub correlating_features: Vec<String>,
}

/// Correlation between timeline activities and highlight engagement
#[protocol("cg")]
pub struct TimelineHighlightCorrelation {
    /// Topic from timeline activity
    pub timeline_topic: String,
    /// Topic from highlight engagement
    pub highlight_topic: String,
    /// Strength of correlation (0.0 - 1.0)
    pub correlation_strength: f64,
    /// Number of data points supporting this correlation
    pub sample_size: usize,
    /// Confidence in this correlation
    pub confidence: f64,
    /// Highlight ID associated with this correlation (if available)
    pub highlight_id: Option<String>,
}

/// Adaptive recommendations for improving network analysis and cross-system learning
#[protocol("cg")]
pub struct AdaptiveRecommendations {
    /// Recommended optimizations for user engagement
    pub engagement_optimizations: Vec<String>,
    /// Cross-system improvements for better integration
    pub cross_system_improvements: Vec<String>,
    /// Learning priorities for future behavioral collection
    pub learning_priorities: Vec<String>,
    /// Adaptive parameters for network analysis
    pub analysis_parameter_suggestions: Vec<String>,
}

/// Update behavioral network patterns for enhanced analysis
#[protocol("cg")]
#[codegen(fn = "update_behavioral_network_patterns() -> UpdateBehavioralNetworkPatternsResult")]
pub struct UpdateBehavioralNetworkPatternsParams {
    /// Domain for the patterns
    pub domain: String,
    /// Atoms that have shown high engagement
    pub engaging_atoms: Vec<String>,
    /// Atom types that correlate with engagement
    pub engaging_atom_types: Vec<String>,
    /// Successful bridge atom patterns
    pub successful_bridge_patterns: Vec<String>,
    /// Temporal engagement weights by hour (0-23)
    pub temporal_engagement_weights: HashMap<u8, f64>,
}

/// Result of updating behavioral network patterns
#[protocol("cg")]
pub struct UpdateBehavioralNetworkPatternsResult {
    /// Success status
    pub success: bool,
    /// Updated patterns count
    pub patterns_updated: usize,
    /// Cross-system correlations discovered
    pub new_correlations: Vec<String>,
    /// Learning effectiveness improvement
    pub learning_improvement: f64,
}

/// Session context for clustering analysis
#[protocol("cg")]
pub struct SessionContext {
    /// Time spent on page in seconds
    pub time_on_page_seconds: u64,
    /// Number of highlights clicked in this session
    pub highlights_clicked_count: u32,
    /// Number of timeline activities engaged with
    pub timeline_activities_engaged: u32,
    /// Whether network panel was opened
    pub network_panel_opened: bool,
    /// Recent page sequence for context
    pub recent_page_sequence: Vec<String>,
}

/// Apply clustering-based filtering to search terms
#[protocol("cg")]
#[codegen(fn = "apply_clustering_filter() -> ClusteringFilterResult")]
pub struct ClusteringFilterParams {
    /// Search terms to filter
    pub search_terms: Vec<String>,
    /// Domain for behavioral context
    pub domain: String,
    /// Current session context
    pub session_context: SessionContext,
}

/// Result of clustering-based term filtering
#[protocol("cg")]
pub struct ClusteringFilterResult {
    /// Search terms acger clustering filter
    pub filtered_search_terms: Vec<String>,
    /// Original number of terms
    pub original_count: usize,
    /// Number of terms acger filtering
    pub filtered_count: usize,
    /// Ratio of terms kept (0.0-1.0)
    pub filter_ratio: f64,
    /// Whether clustering was successfully applied
    pub clustering_applied: bool,
    /// Explanation of filtering decision
    pub filter_reasoning: String,
}

#[inline]
pub fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}
