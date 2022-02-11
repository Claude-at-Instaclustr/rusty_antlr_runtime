//! General AST
use std::any::{Any, TypeId};
use std::borrow::Borrow;

use std::fmt::{Debug, Formatter};
use std::iter::from_fn;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use crate::atn::INVALID_ALT;
use crate::char_stream::InputData;
use crate::int_stream::EOF;
use crate::interval_set::Interval;
use crate::recogniser_rule_context::RecogniserRuleContext;
use crate::recognizer::{RecogniserNodeType, Recognizer};
use crate::rule_context::{CustomRuleContext, RuleContext};
use crate::token::Token;
use crate::token_factory::TokenFactory;
use crate::{interval_set, trees};
use better_any::{Tid, TidAble};

type ActionFunc<'input> = fn(isize, isize, dyn Recognizer<'input>);
type SempredFunc<'input> = fn(isize, isize, dyn Recognizer<'input>) -> bool;

pub enum NodeType {
    Leaf,
    Error,
    Rule,
    AST,
    Empty,
}
pub struct NodeImpl<'input> {
    node_type: NodeType,
    parent: Option<NodeImpl<'input>>,
    children: Vec<NodeImpl<'input>>,
    token: Option<dyn Token>,
    rule_context: Option<dyn RuleContext<'input>>,
    source_interval: Interval,
    sempred: Option<SempredFunc<'input>>,
    action: Option<ActionFunc<'input>>,
}

impl<'input> NodeImpl<'input> {
    pub fn new(node_type: NodeType, parent: Option<NodeImpl>) -> NodeImpl {
        NodeImpl {
            node_type,
            parent,
            children: vec![],
            token: None,
            rule_context: None,
            source_interval: interval_set::INVALID,
            sempred: None,
            action: None,
        }
    }

    pub fn set_action(&mut self, func: ActionFunc<'input>) {
        self.action = func
    }
    pub fn set_sempred(&mut self, state: bool) {
        self.sempred = state;
    }
    pub fn add_child(&mut self, child: NodeImpl) {
        self.children.append(child);
    }
    pub fn set_token(&mut self, token: Option<dyn Token>) {
        self.token = token;
    }
    pub fn set_rule_context(&mut self, rule_context: Option<dyn RuleContext>) {
        self.rule_context = rule_context;
    }
    pub fn set_source_interval(&mut self, source_interval: Interval) {
        self.source_interval = source_interval;
    }
    pub(crate) fn sempred(&mut self, rule_index: isize, action_index: isize) -> bool {
        match self.sempred {
            Some(f) => f(rule_index, action_index),
            _ => true,
        }
    }

    pub(crate) fn action(&mut self, rule_index: isize, action_index: isize) {
        match self.action {
            Some(f) => f(rule_index, action_index),
            _ => {}
        }
    }

}

impl<'input> Node<'input> for NodeImpl<'input> {
    fn get_node_type(&self) -> &NodeType {
        &self.node_type
    }
    fn is_node_type(&self, node_type : &NodeType) ->bool {
        self.node_type == node_type
    }
    fn get_parent(&self) -> Option<NodeImpl<'input>> {
        self.parent
    }
    fn has_parent(&self) -> bool {
        match &self.parent {
            Some(_x) => true,
            _ => false,
        }
    }
    fn get_payload(&self) -> Box<dyn Any> {
        let result = match &self.node_type {
            Leaf => &self.token,
            Error => &self.token,
            Rule => &self.rule_context,
            AST => &self.token,
            Empty => &self.token,
        };
        Box::new(result)
    }

    fn get_child(&self, i: usize) -> Option<&NodeImpl> {
        self.children.get(i)
    }
    fn get_child_count(&self) -> usize {
        self.children.len()
    }
    fn get_children<'a>(&'a self) -> Box<dyn Iterator<Item =NodeImpl> + 'a>
    where
        'input: 'a,
    {
        let mut index = 0;
        let iter = if index < self.get_child_count() {
            index += 1;
            self.get_child(index - 1)
        } else {
            None
        };

        Box::new(iter)
    }

    fn get_source_interval(&self) -> Interval {
        self.source_interval
    }

    /// Return combined text of this AST node.
    /// To create resulting string it does traverse whole subtree,
    /// also it includes only tokens added to the parse tree
    ///
    /// Since tokens on hidden channels (e.g. whitespace or comments) are not
    ///	added to the parse trees, they will not appear in the output of this
    ///	method.
    fn get_text(&self) -> String {
        match self.get_payload() {
            Some(t) => t.get_text(),
            _ => String::new(),
        }
    }
    fn is_token_type(&self, token_type : isize) -> bool {
        match &self.token {
            Some(x) => x.get_token_type() == token_type,
            _ => false,
        }
    }
    fn get_token(&self) -> &Option<dyn Token> {
        &self.token
    }
}

pub trait Node<'input> {
    fn get_node_type(&self) -> &NodeType;
    fn is_node_type(&self, node_type : &NodeType) ->bool;
    fn is_token_type(&self, token_type : isize) -> bool;
    fn get_token(&self) -> Option<dyn Token>;
    fn get_parent(&self) -> Option<NodeImpl>;
    fn has_parent(&self) -> bool;
    fn get_payload(&self) -> Box<dyn Any>;
    fn get_child(&self, i: usize) -> Option<&NodeImpl>;
    fn get_child_count(&self) -> usize;
    fn get_children<'input>(&'input self) -> Box<dyn Iterator<Item =NodeImpl> + 'input>;
    fn get_source_interval(&self) -> Interval;
    fn get_text(&self) -> String;
}
