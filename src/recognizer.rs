
use std::collections::HashMap;
use std::iter::Map;
use std::sync::{Arc, RwLock};
use crate::atn::ATN;
use crate::rule_context::RuleContext;
use better_any::TidAble;
use crate::atn_config_set::ATNConfigSet;
use crate::atn_deserializer::ATNDeserializer;
use crate::dfa::DFA;
use crate::errors::BaseRecognitionError;
use crate::int_stream::IntStream;
use crate::token::{TOKEN_EOF, TOKEN_INVALID_TYPE};
use crate
use crate::atn_simulator::IATNSimulator;
use crate::error_listener::ErrorListener; bit_set;
use crate::token_factory::{TokenAware, TokenFactory};
use crate::token_stream::TokenStream;
use crate::tree::NodeImpl;
use crate::vocabulary::{Vocabulary, VocabularyImpl};

/// Major version of this runtime.
/// Used by generated parser to verify that it is compatible with current version of runtime
pub const VERSION_MAJOR: &'static str = env!("CARGO_PKG_VERSION_MAJOR");
/// Major version of this runtime.
/// Used by generated parser to verify that it is compatible with current version of runtime
pub const VERSION_MINOR: &'static str = env!("CARGO_PKG_VERSION_MINOR");

// todo move to compile time check when it will be possible to compare strings in constants
/// Used by generated parser to verify that it is compatible with current version of runtime
pub fn check_version(major: &str, minor: &str) {
    assert!(major == VERSION_MAJOR && minor == VERSION_MINOR,
            "parser is not compatible with current runtime version, please generate parser with the latest version of ANTLR")
}
//todo just a reminder to update version to be inserted in generated parser,
//const _:[();0-!(VERSION_MAJOR == "0" && VERSION_MINOR == "2") as usize] = [];
pub struct LookaheadEventInfo {
    predicted_alt : usize,
}

pub struct DecisionEventInfo<'input> {
    decision : usize,
    configs : ATNConfigSet,
    input : dyn TokenStream<'input>,
    start_index : usize,
    stop_index : usize,
    full_context : bool,
}

type ContextSensitivityInfo<'input> = DecisionEventInfo<'input>;
type ErrorInfo<'input> = DecisionEventInfo<'input>;

pub struct DecisionInfo<'input> {
    /**
     * The decision number, which is an index into {@link ATN#decisionToState}.
     */
    decision : i32,

    /**
     * The total number of times {@link ParserATNSimulator#adaptivePredict} was
     * invoked for this decision.
     */
    invocations : i64,

    /**
     * The total time spent in {@link ParserATNSimulator#adaptivePredict} for
     * this decision, in nanoseconds.
     *
     * <p>
     * The value of this field contains the sum of differential results obtained
     * by {@link System#nanoTime()}, and is not adjusted to compensate for JIT
     * and/or garbage collection overhead. For best accuracy, use a modern JVM
     * implementation that provides precise results from
     * {@link System#nanoTime()}, and perform profiling in a separate process
     * which is warmed up by parsing the input prior to profiling. If desired,
     * call {@link ATNSimulator#clearDFA} to reset the DFA cache to its initial
     * state before starting the profiling measurement pass.</p>
     */
    timeInPrediction : i64,

    /**
     * The sum of the lookahead required for SLL prediction for this decision.
     * Note that SLL prediction is used before LL prediction for performance
     * reasons even when {@link PredictionMode#LL} or
     * {@link PredictionMode#LL_EXACT_AMBIG_DETECTION} is used.
     */
    SLL_TotalLook : i64,

    /**
     * Gets the minimum lookahead required for any single SLL prediction to
     * complete for this decision, by reaching a unique prediction, reaching an
     * SLL conflict state, or encountering a syntax error.
     */
    SLL_MinLook : i64,

    /**
     * Gets the maximum lookahead required for any single SLL prediction to
     * complete for this decision, by reaching a unique prediction, reaching an
     * SLL conflict state, or encountering a syntax error.
     */
    pub SLL_MaxLook : i64,

    /**
     * Gets the {@link LookaheadEventInfo} associated with the event where the
     * {@link #SLL_MaxLook} value was set.
     */
    pub SLL_MaxLookEvent : LookaheadEventInfo ,

    /**
     * The sum of the lookahead required for LL prediction for this decision.
     * Note that LL prediction is only used when SLL prediction reaches a
     * conflict state.
     */
    pub LL_TotalLook : i64,

    /**
     * Gets the minimum lookahead required for any single LL prediction to
     * complete for this decision. An LL prediction completes when the algorithm
     * reaches a unique prediction, a conflict state (for
     * {@link PredictionMode#LL}, an ambiguity state (for
     * {@link PredictionMode#LL_EXACT_AMBIG_DETECTION}, or a syntax error.
     */
    pub LL_MinLook : i64,

    /**
     * Gets the maximum lookahead required for any single LL prediction to
     * complete for this decision. An LL prediction completes when the algorithm
     * reaches a unique prediction, a conflict state (for
     * {@link PredictionMode#LL}, an ambiguity state (for
     * {@link PredictionMode#LL_EXACT_AMBIG_DETECTION}, or a syntax error.
     */
    pub LL_MaxLook : i64,

    /**
     * Gets the {@link LookaheadEventInfo} associated with the event where the
     * {@link #LL_MaxLook} value was set.
     */
    pub  LL_MaxLookEvent : LookaheadEventInfo,

    /**
     * A collection of {@link ContextSensitivityInfo} instances describing the
     * context sensitivities encountered during LL prediction for this decision.
     *
     * @see ContextSensitivityInfo
     */
    pub contextSensitivities : Vec<ContextSensitivityInfo<'input>>,

    /**
     * A collection of {@link ErrorInfo} instances describing the parse errors
     * identified during calls to {@link ParserATNSimulator#adaptivePredict} for
     * this decision.
     *
     * @see ErrorInfo
     */
    pub  errors : Vec<ErrorInfo<'input>>,

    /**
     * A collection of {@link AmbiguityInfo} instances describing the
     * ambiguities encountered during LL prediction for this decision.
     *
     * @see AmbiguityInfo
     */
    pub ambiguities : Vec<AmbiguityInfo<'input>>,

    /**
     * A collection of {@link PredicateEvalInfo} instances describing the
     * results of evaluating individual predicates during prediction for this
     * decision.
     *
     * @see PredicateEvalInfo
     */
    pub predicateEvals : Vec<PredicateEvalInfo>,

    /**
     * The total number of ATN transitions required during SLL prediction for
     * this decision. An ATN transition is determined by the number of times the
     * DFA does not contain an edge that is required for prediction, resulting
     * in on-the-fly computation of that edge.
     *
     * <p>
     * If DFA caching of SLL transitions is employed by the implementation, ATN
     * computation may cache the computed edge for efficient lookup during
     * future parsing of this decision. Otherwise, the SLL parsing algorithm
     * will use ATN transitions exclusively.</p>
     *
     * @see #SLL_ATNTransitions
     * @see ParserATNSimulator#computeTargetState
     * @see LexerATNSimulator#computeTargetState
     */
    pub SLL_ATNTransitions : i64,

    /**
     * The total number of DFA transitions required during SLL prediction for
     * this decision.
     *
     * <p>If the ATN simulator implementation does not use DFA caching for SLL
     * transitions, this value will be 0.</p>
     *
     * @see ParserATNSimulator#getExistingTargetState
     * @see LexerATNSimulator#getExistingTargetState
     */
    pub SLL_DFATransitions : i64,

    /**
     * Gets the total number of times SLL prediction completed in a conflict
     * state, resulting in fallback to LL prediction.
     *
     * <p>Note that this value is not related to whether or not
     * {@link PredictionMode#SLL} may be used successfully with a particular
     * grammar. If the ambiguity resolution algorithm applied to the SLL
     * conflicts for this decision produce the same result as LL prediction for
     * this decision, {@link PredictionMode#SLL} would produce the same overall
     * parsing result as {@link PredictionMode#LL}.</p>
     */
    pub LL_Fallback : i64,

    /**
     * The total number of ATN transitions required during LL prediction for
     * this decision. An ATN transition is determined by the number of times the
     * DFA does not contain an edge that is required for prediction, resulting
     * in on-the-fly computation of that edge.
     *
     * <p>
     * If DFA caching of LL transitions is employed by the implementation, ATN
     * computation may cache the computed edge for efficient lookup during
     * future parsing of this decision. Otherwise, the LL parsing algorithm will
     * use ATN transitions exclusively.</p>
     *
     * @see #LL_DFATransitions
     * @see ParserATNSimulator#computeTargetState
     * @see LexerATNSimulator#computeTargetState
     */
    pub LL_ATNTransitions : i64,

    /**
     * The total number of DFA transitions required during LL prediction for
     * this decision.
     *
     * <p>If the ATN simulator implementation does not use DFA caching for LL
     * transitions, this value will be 0.</p>
     *
     * @see ParserATNSimulator#getExistingTargetState
     * @see LexerATNSimulator#getExistingTargetState
     */
    pub LL_DFATransitions : i64,
}

pub struct AmbiguityInfo<'input> {
    abiguityAlts : bit_set,
    decision_info : DecisionEventInfo<'input>
}

pub trait ParseInfo {
    fn get_decision_info(&self) -> Vec<DecisionInfo>;
    fn get_ll_decisions(&self) -> Vec<i32>;
    fn get_total_time_in_prediction(&self) -> i64;
    fn get_total_sl_lookahead_ops(&self) -> i64;
    fn get_total_ll_lookahead_ops(&self) -> i64;
    fn get_total_sll_atn_lookahead_ops(&self) -> i64;
    fn get_total_ll_atn_lookahead_ops(&self) -> i64;
    fn get_total_atn_lookahead_ops(&self) -> i64;
    fn get_dfa_size(&self) -> i32;
    fn get_dfa_size_from_decision(&self, decision : i32) -> i32;
}

pub type TOKEN_TYPE_MAP_TYPE<'input> = &'input HashMap<&'input str, isize>;
pub type RULE_INDEX_MAP_TYPE<'input> = &'input HashMap<&'input str,usize>;

/// **! Usually generated by ANTLR !**
pub trait Recognizer<'input> {
    const EOF : int = -1;
    fn get_rule_names(&self) -> &[&str];
    fn get_vocabulary(&self) -> &VocabularyImpl;
    fn get_token_type_map(&self) -> TOKEN_TYPE_MAP_TYPE<'input>;
    fn get_rule_index_map(&self) -> RULE_INDEX_MAP_TYPE<'input>;
    fn get_token_type(&self, token_name : &str) -> isize;
    fn get_serialized_atn(&self) -> &str;
    fn get_grammar_file_name(&self) -> &str;
    fn get_atn(&self) -> &ATN;
    fn get_interpreter(&self) -> &ATNInterpreter;
    fn get_parse_info(&self) -> Option<dyn ParseInfo>;
    fn set_interpreter(&mut self, interpreter:ATNInterpreter);
    fn get_error_header(&self, err: BaseRecognitionError) -> String;
    fn add_error_listener(&mut self, listener : ANTLRErrorListener);
    fn get_error_listeners(&self) -> Vec<ANTLRErrorListener>;
    fn sempred(&mut self, local_ctxt : Box<dyn RuleContext<'input>>, rule_index: isize, action_index: isize) -> bool;
    fn precpred(&mut self, local_ctxt : Box<dyn RuleContext<'input>>, precedence: isize) -> bool;
    fn action(&mut self, local_ctxt : Box<dyn RuleContext<'input>>, rule_index: isize, action_index: isize);
    fn get_state(&self) -> isize;
    fn set_state(&mut self, state : isize);
    fn get_input_stream(&self) -> Option<&'input mut dyn IntStream>;
    fn set_input_stream(&mut self, input : Option<&'input mut dyn IntStream>);
    fn get_token_factory(&self) -> Option<dyn TokenFactory<'input>>;
    fn set_token_factory(&mut self, factory : Option<dyn TokenFactory<'input>>);
    fn reset(&mut self);
}

pub struct RecognizerImpl<'input, T : Recognizer<'input>> {
    grammar_file_name: &'static str,
    rule_names: &'static [&'static str],
    channel_names: &'static [&'static str],
    mode_names: &'static [&'static str],
    vocabulary: VocabularyImpl,
    serialized_atn: &'static [&'static str],
    interpreter : Option<ATNInterpreter>,
    input : Option<&'input mut dyn IntStream>,
    token_factory : Option<dyn TokenFactory<'input>>,
    error_listeners : Vec<Box<ErrorListener<'input, T>>>,
    parse_info : Option<dyn ParseInfo>,
    atn : Arc<ATN>,
    rule_index_map : RULE_INDEX_MAP_TYPE<'input>,
    token_type_map : TOKEN_TYPE_MAP_TYPE<'input>,
    state_number : isize,
}

impl<'input> RecognizerImpl<'input, T> {
    pub fn new(
        grammar_file_name: &'static str,
        rule_names: &'static [&'static str],
        channel_names: &'static [&'static str],
        mode_names: &'static [&'static str],
        vocabulary: VocabularyImpl,
        serialized_atn: &'static [&'static str],

    ) -> Self {

        let mut rule_index_map :RULE_TYPE_INDEX_MAP_TYPE<'input> = HashMap::new();
        for i in 0..rule_names.len() {
            rule_index_map.insert(rule_names[i], i);
        }
        let atn : Arc<ATN> = Arc::new(ATNDeserializer::new(None).deserialize(serialized_atn.chars()));
        let mut token_type_map : TOKEN_TYPE_MAP_TYPE<'input> = HashMap::new();
        for i  in 0..ATN.maxTokenType {
            let literalName = vocabulary.getLiteralName(i);
            if literalName.isSome() {
                token_type_map.insert(literalName, i);
            }
            let  symbolicName = vocabulary.getSymbolicName(i);
            if symbolicName.isSome() {
                token_type_map.insert(symbolicName, i);
            }
        }
        token_type_map.insert( "EOF", TOKEN_EOF);

        RecognizerImpl {
            grammar_file_name,
            rule_names,
            channel_names,
            mode_names,
            vocabulary,
            serialized_atn,
            interpreter: None,
            input: None,
            token_factory: None,
            error_listeners: vec![],
            parse_info: None,
            atn,
            rule_index_map,
            token_type_map,
            state_number : -1,
        }
    }
}

pub type ATNInterpreter = dyn IATNSimulator;

impl<'input> Recognizer<'input> for RecognizerImpl<'input, T> {
    fn get_rule_names(&self) -> &[&str] {
        self.rule_names
    }

    fn get_vocabulary(&self) -> &VocabularyImpl {
        &self.vocabulary
    }

    fn get_token_type_map(&self) -> TOKEN_TYPE_MAP_TYPE<'input> {
        &self.token_type_map
    }

    fn get_rule_index_map(&self) -> RULE_INDEX_MAP_TYPE<'input> {
        &self.rule_index_map;
    }

    fn get_token_type(&self, token_name : &str) -> isize {
        match self.token_type_map.get( token_name ) {
            Some(t) => *t,
            _ => TOKEN_INVALID_TYPE
        }
    }

    fn get_serialized_atn(&self) -> &str {
        self.serialized_atn.join("").as_str()
    }

    fn get_grammar_file_name(&self) -> &str {
        self.grammar_file_name
    }

    fn get_atn(&self) -> Arc<ATN> {
        self.atn.clone()
    }

    fn get_interpreter(&self) -> &Option<ATNInterpreter> {
        &self.interpreter
    }

    fn get_parse_info(&self) -> &Option<dyn ParseInfo> {
        &self.parse_info
    }

    fn set_interpreter(&mut self, interpreter: Option<ATNInterpreter>) {
        self.interpreter = interpreter
    }

    fn get_error_header(&self, err: BaseRecognitionError) -> String {
        todo!()
    }

    fn add_error_listener(&mut self, listener: Box<ErrorListener<T>>) {
        self.error_listeners.push( listener );
    }

    fn get_error_listeners(& self) -> Vec<Box<ErrorListener<T>>> {
        self.error_listeners.clone()
    }

    fn sempred(&mut self, local_ctxt: Box<dyn RuleContext<'input>>, rule_index: isize, action_index: isize) -> bool {
        todo!();
        true
    }

    fn precpred(&mut self, local_ctxt: Box<dyn RuleContext<'input>>, precedence: isize) -> bool {
        todo!();
        true
    }

    fn action(&mut self, local_ctxt: Box<dyn RuleContext<'input>>, rule_index: isize, action_index: isize) {
        todo!()
    }

    fn get_state(&self) -> isize {
        return self.state_number
    }

    fn set_state(&mut self, state: isize) {
        self.state_number = state;
    }

    fn get_input_stream(&self) -> &Option<&mut dyn IntStream> {
        &self.input
    }

    fn set_input_stream(&mut self, input: Option<&mut dyn IntStream>) {
        self.input = input;
    }

    fn get_token_factory(&self) -> &Option<dyn TokenFactory<'input>> {
        &self.token_factory
    }

    fn set_token_factory(&mut self, factory: Option<dyn TokenFactory<'input>>) {
        self.token_factory = factory;
    }

    fn reset(&mut self) {
        if self.input.is_some() {
            self.input.unwrap().seek(0);
        }
        if self.interpreter.is_some() {
            self.interpreter.unwrap().reset();
        }
    }
}
