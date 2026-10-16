//! Conservative interval proofs for the native backend. Keys refer to the original AST;
//! callers must not clone or move its nodes between analysis and code generation.
use crate::compiler::{Block, Expr, Function, Instruction, Type};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(super) struct Proofs {
    arithmetic: BTreeSet<usize>,
    unsigned_division: BTreeSet<usize>,
    index: BTreeSet<usize>,
    assignment_index: BTreeSet<usize>,
    calls: BTreeMap<usize, Vec<Interval>>,
}
impl Proofs {
    pub(super) fn call_inputs(&self, expr: &Expr) -> Option<&[Interval]> {
        self.calls
            .get(&(expr as *const Expr as usize))
            .map(Vec::as_slice)
    }
    pub(super) fn unsigned_division(&self, expr: &Expr) -> bool {
        self.unsigned_division
            .contains(&(expr as *const Expr as usize))
    }
    pub(super) fn arithmetic(&self, expr: &Expr) -> bool {
        self.arithmetic.contains(&(expr as *const Expr as usize))
    }
    pub(super) fn index(&self, expr: &Expr) -> bool {
        self.index.contains(&(expr as *const Expr as usize))
    }
    pub(super) fn assignment_index(&self, instruction: &Instruction) -> bool {
        self.assignment_index
            .contains(&(instruction as *const Instruction as usize))
    }
}

const MIN: i128 = i64::MIN as i128;
const MAX: i128 = i64::MAX as i128;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Interval {
    lo: i128,
    hi: i128,
}
impl Interval {
    pub(super) const TOP: Self = Self { lo: MIN, hi: MAX };
    fn point(value: i64) -> Self {
        Self {
            lo: value as i128,
            hi: value as i128,
        }
    }
    fn join(self, other: Self) -> Self {
        Self {
            lo: self.lo.min(other.lo),
            hi: self.hi.max(other.hi),
        }
    }
    fn intersect(self, lo: i128, hi: i128) -> Option<Self> {
        let value = Self {
            lo: self.lo.max(lo),
            hi: self.hi.min(hi),
        };
        (value.lo <= value.hi).then_some(value)
    }
    fn contains(self, other: Self) -> bool {
        self.lo <= other.lo && self.hi >= other.hi
    }
    fn widen(self, next: Self) -> Self {
        Self {
            lo: if next.lo < self.lo { MIN } else { self.lo },
            hi: if next.hi > self.hi { MAX } else { self.hi },
        }
    }
    // The result interval describes only executions on which the checked operation succeeds.
    // Raw arithmetic is permitted only if *all* endpoint results fit, before intersection.
    fn arithmetic(self, op: &str, rhs: Self) -> (Self, bool) {
        let (lo, hi) = match op {
            "+" => (self.lo + rhs.lo, self.hi + rhs.hi),
            "-" => (self.lo - rhs.hi, self.hi - rhs.lo),
            "*" => {
                let products = [
                    self.lo * rhs.lo,
                    self.lo * rhs.hi,
                    self.hi * rhs.lo,
                    self.hi * rhs.hi,
                ];
                (
                    *products.iter().min().unwrap(),
                    *products.iter().max().unwrap(),
                )
            }
            "%" if rhs.lo == rhs.hi && rhs.lo > 0 => {
                let limit = rhs.lo - 1;
                let value = Self {
                    lo: if self.lo >= 0 { 0 } else { -limit },
                    hi: if self.hi <= 0 { 0 } else { limit },
                };
                // This is a successful-result fact, not permission to remove checks.
                return (value, false);
            }
            _ => return (Self::TOP, false),
        };
        (
            Self::TOP.intersect(lo, hi).unwrap_or(Self::TOP),
            lo >= MIN && hi <= MAX,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Fact {
    value: Interval,
    length: Option<usize>,
}
type Env = BTreeMap<String, Fact>;
type State = Option<Env>;
fn join(a: State, b: State) -> State {
    match (a, b) {
        (None, value) | (value, None) => value,
        (Some(mut a), Some(b)) => {
            for (name, fact) in &mut a {
                if let Some(other) = b.get(name) {
                    fact.value = fact.value.join(other.value);
                    if fact.length != other.length {
                        fact.length = None;
                    }
                } else {
                    fact.value = Interval::TOP;
                    fact.length = None;
                }
            }
            Some(a)
        }
    }
}
#[derive(Clone, Copy)]
enum Head<'a> {
    Condition(&'a Expr),
    // `for name in start..end`: the immutable counter and its whole range.
    Range(&'a str, Interval),
}
#[derive(Default)]
struct Flow {
    normal: State,
    breaks: State,
    continues: State,
    returns: State,
}
impl Flow {
    fn merge(self, other: Self) -> Self {
        Self {
            normal: join(self.normal, other.normal),
            breaks: join(self.breaks, other.breaks),
            continues: join(self.continues, other.continues),
            returns: join(self.returns, other.returns),
        }
    }
}
#[derive(Clone, Default)]
struct Analysis {
    // AND repeated visits: one proof key must be safe in every reachable context.
    arithmetic: BTreeMap<usize, bool>,
    unsigned_division: BTreeMap<usize, bool>,
    index: BTreeMap<usize, bool>,
    assignment_index: BTreeMap<usize, bool>,
    calls: BTreeMap<usize, Vec<Interval>>,
}
fn record(map: &mut BTreeMap<usize, bool>, key: usize, safe: bool, collect: bool) {
    if collect {
        map.entry(key)
            .and_modify(|old| *old &= safe)
            .or_insert(safe);
    }
}
fn safe_index(env: &Env, name: &str, range: Interval) -> bool {
    env.get(name)
        .and_then(|f| f.length)
        .is_some_and(|length| range.lo >= 0 && range.hi < length as i128)
}
impl Analysis {
    fn expr(&mut self, expr: &Expr, env: &Env, collect: bool) -> Interval {
        let key = expr as *const Expr as usize;
        match expr {
            Expr::Int(value) => Interval::point(*value),
            // 匿名函数体在别处执行，不参与这里的区间推理；函数值的调用结果未知。
            Expr::Lambda(..) | Expr::FnRef(_) => Interval::TOP,
            Expr::Apply(_, values) => {
                for value in values {
                    self.expr(value, env, collect);
                }
                Interval::TOP
            }
            Expr::Variable(name) => env.get(name).map_or(Interval::TOP, |f| f.value),
            Expr::Unary('-', inner) => {
                let range = self.expr(inner, env, collect);
                let (value, safe) = Interval::point(0).arithmetic("-", range);
                record(&mut self.arithmetic, key, safe, collect);
                value
            }
            Expr::Binary(op, _, _) if op == "&&" || op == "||" => {
                self.condition(expr, env.clone(), collect);
                Interval::TOP
            }
            Expr::Binary(op, left, right) => {
                let a = self.expr(left, env, collect);
                let b = self.expr(right, env, collect);
                let (value, safe) = a.arithmetic(op, b);
                if matches!(op.as_str(), "+" | "-" | "*") {
                    record(&mut self.arithmetic, key, safe, collect);
                }
                if matches!(op.as_str(), "/" | "%") {
                    // These intervals describe successful operand evaluation. Checks
                    // inside the operands remain independent of this division proof.
                    record(
                        &mut self.unsigned_division,
                        key,
                        a.lo >= 0 && b.lo > 0,
                        collect,
                    );
                }
                value
            }
            Expr::Index(name, index) => {
                let range = self.expr(index, env, collect);
                record(&mut self.index, key, safe_index(env, name, range), collect);
                Interval::TOP
            }
            Expr::Catch(inner, binding, block) => {
                self.expr(inner, env, collect);
                let mut handler = env.clone();
                handler.insert(
                    binding.clone(),
                    Fact {
                        value: Interval::TOP,
                        length: None,
                    },
                );
                // Checked catch handlers always return; none of their assignments flow out.
                self.block(block, handler, collect);
                Interval::TOP
            }
            Expr::Unary(_, inner)
            | Expr::Try(inner)
            | Expr::ByteLen(inner)
            | Expr::CharLen(inner) => {
                self.expr(inner, env, collect);
                Interval::TOP
            }
            Expr::StringEqual(a, b) => {
                self.expr(a, env, collect);
                self.expr(b, env, collect);
                Interval::TOP
            }
            Expr::Record(_, fields) => {
                for (_, value) in fields {
                    self.expr(value, env, collect);
                }
                Interval::TOP
            }
            Expr::Call(_, values) => {
                let inputs: Vec<_> = values
                    .iter()
                    .map(|value| self.expr(value, env, collect))
                    .collect();
                if collect {
                    self.calls
                        .entry(key)
                        .and_modify(|old| {
                            // The original node has fixed arity. Fail closed on malformed capture.
                            if old.len() != inputs.len() {
                                old.clear();
                            } else {
                                for (old, input) in old.iter_mut().zip(&inputs) {
                                    *old = old.join(*input);
                                }
                            }
                        })
                        .or_insert(inputs);
                }
                Interval::TOP
            }
            Expr::Array(values) | Expr::Format(values) | Expr::Tuple(values) => {
                for value in values {
                    self.expr(value, env, collect);
                }
                Interval::TOP
            }
            Expr::Method(_, method, values)
            | Expr::ElementMethod(_, _, _, method, values)
            | Expr::Chain(_, method, values) => {
                if let Expr::ElementMethod(_, index, _, _, _) | Expr::Chain(index, _, _) = expr {
                    self.expr(index, env, collect);
                }
                for value in values {
                    self.expr(value, env, collect);
                }
                // A list length is a size_t that fits i64; it says nothing about indices.
                if method == "len" {
                    Interval { lo: 0, hi: MAX }
                } else {
                    Interval::TOP
                }
            }
            Expr::BorrowIndex(_, index, _, _) | Expr::IndexField(_, index, _) => {
                self.expr(index, env, collect);
                Interval::TOP
            }
            Expr::MapLit(entries) => {
                for (key, value) in entries {
                    self.expr(key, env, collect);
                    self.expr(value, env, collect);
                }
                Interval::TOP
            }
            Expr::If(condition, yes, no) => {
                // Each branch sees the facts its condition establishes.
                let (yes_env, no_env) = self.condition(condition, env.clone(), collect);
                let a = yes_env.map(|env| self.expr(yes, &env, collect));
                let b = no_env.map(|env| self.expr(no, &env, collect));
                match (a, b) {
                    (Some(a), Some(b)) => a.join(b),
                    (Some(value), None) | (None, Some(value)) => value,
                    (None, None) => Interval::TOP,
                }
            }
            Expr::Variant(_, _, values) => {
                for value in values {
                    self.expr(value, env, collect);
                }
                Interval::TOP
            }
            Expr::Match(scrutinee, arms) => {
                self.expr(scrutinee, env, collect);
                for ((_, bindings), value) in arms {
                    self.expr(value, &opaque(env, bindings), collect);
                }
                Interval::TOP
            }
            // f64 不参与整数区间推理。
            Expr::Field(_, _)
            | Expr::Text(_)
            | Expr::Borrow(_, _)
            | Expr::Bool(_)
            | Expr::Float(_) => Interval::TOP,
        }
    }
    fn condition(&mut self, expr: &Expr, env: Env, collect: bool) -> (State, State) {
        match expr {
            Expr::Bool(true) => (Some(env), None),
            Expr::Bool(false) => (None, Some(env)),
            Expr::Unary('!', inner) => {
                let (yes, no) = self.condition(inner, env, collect);
                (no, yes)
            }
            Expr::Binary(op, left, right) if op == "&&" => {
                let (yes, no) = self.condition(left, env, collect);
                let (right_yes, right_no) =
                    yes.map_or((None, None), |env| self.condition(right, env, collect));
                (right_yes, join(no, right_no))
            }
            Expr::Binary(op, left, right) if op == "||" => {
                let (yes, no) = self.condition(left, env, collect);
                let (right_yes, right_no) =
                    no.map_or((None, None), |env| self.condition(right, env, collect));
                (join(yes, right_yes), right_no)
            }
            Expr::Binary(op, left, right)
                if matches!(op.as_str(), "<" | "<=")
                    && matches!(left.as_ref(), Expr::Variable(_))
                    && scalar_variables(right).is_some() =>
            {
                // Evaluate first: even a zero-trip loop must retain condition errors.
                self.expr(left, &env, collect);
                let bound = self.expr(right, &env, collect);
                let Expr::Variable(name) = left.as_ref() else {
                    unreachable!()
                };
                let inclusive = op == "<=";
                (
                    refine_bound(&env, name, MIN, bound.hi - i128::from(!inclusive)),
                    refine_bound(&env, name, bound.lo + i128::from(inclusive), MAX),
                )
            }
            _ => {
                // Prove evaluation under incoming facts, never facts from its own result.
                self.expr(expr, &env, collect);
                (refine(expr, env.clone(), true), refine(expr, env, false))
            }
        }
    }
    fn block(&mut self, block: &Block, env: Env, collect: bool) -> Flow {
        let outer: BTreeSet<_> = env.keys().cloned().collect();
        let mut result = Flow {
            normal: Some(env),
            ..Flow::default()
        };
        for (instruction, _) in block {
            let Some(env) = result.normal.take() else {
                break;
            };
            let next = self.instruction(instruction, env, collect);
            result.normal = next.normal;
            result.breaks = join(result.breaks, next.breaks);
            result.continues = join(result.continues, next.continues);
            result.returns = join(result.returns, next.returns);
        }
        for env in [
            &mut result.normal,
            &mut result.breaks,
            &mut result.continues,
            &mut result.returns,
        ]
        .into_iter()
        .flatten()
        {
            env.retain(|name, _| outer.contains(name));
        }
        result
    }
    fn instruction(&mut self, instruction: &Instruction, mut env: Env, collect: bool) -> Flow {
        match instruction {
            Instruction::Let(name, _, ty, expr) => {
                let value = self.expr(expr, &env, collect);
                let length = match (ty, expr) {
                    (Some(Type::ArrayInt(length)), _) => Some(*length),
                    (None, Expr::Array(values)) => Some(values.len()),
                    (_, Expr::Variable(name)) => env.get(name).and_then(|f| f.length),
                    _ => None,
                };
                env.insert(name.clone(), Fact { value, length });
            }
            Instruction::Unpack(names, _, expr) => {
                self.expr(expr, &env, collect);
                for name in names {
                    env.insert(
                        name.clone(),
                        Fact {
                            value: Interval::TOP,
                            length: None,
                        },
                    );
                }
            }
            Instruction::Assign(name, expr) => {
                let value = self.expr(expr, &env, collect);
                if let Some(fact) = env.get_mut(name) {
                    fact.value = value;
                }
            }
            Instruction::AssignIndex(name, index, value) => {
                let range = self.expr(index, &env, collect);
                record(
                    &mut self.assignment_index,
                    instruction as *const Instruction as usize,
                    safe_index(&env, name, range),
                    collect,
                );
                self.expr(value, &env, collect);
            }
            Instruction::AssignIndexField(_, index, _, value) => {
                self.expr(index, &env, collect);
                self.expr(value, &env, collect);
            }
            Instruction::AssignField(_, _, expr)
            | Instruction::Print(expr)
            | Instruction::Call(expr) => {
                self.expr(expr, &env, collect);
            }
            Instruction::Return(expr) => {
                if let Some(expr) = expr {
                    self.expr(expr, &env, collect);
                }
                return Flow {
                    returns: Some(env),
                    ..Flow::default()
                };
            }
            Instruction::Fail(expr) => {
                self.expr(expr, &env, collect);
                return Flow {
                    returns: Some(env),
                    ..Flow::default()
                };
            }
            Instruction::Break => {
                return Flow {
                    breaks: Some(env),
                    ..Flow::default()
                }
            }
            Instruction::Continue => {
                return Flow {
                    continues: Some(env),
                    ..Flow::default()
                }
            }
            Instruction::If(condition, yes, no) => {
                let (yes_env, no_env) = self.condition(condition, env, collect);
                let a = yes_env.map_or_else(Flow::default, |env| self.block(yes, env, collect));
                let b = no_env.map_or_else(Flow::default, |env| self.block(no, env, collect));
                return a.merge(b);
            }
            Instruction::While(condition, body) => {
                return self.loop_flow(Head::Condition(condition), body, env, collect)
            }
            Instruction::Match(scrutinee, arms) => {
                // Arms bind payloads that carry no interval facts; checked matches are exhaustive.
                self.expr(scrutinee, &env, collect);
                return arms
                    .iter()
                    .fold(Flow::default(), |flow, ((_, bindings), body)| {
                        flow.merge(self.block(body, opaque(&env, bindings), collect))
                    });
            }
            Instruction::Each(name, second, _, _, body) => {
                // Elements are opaque here; the loop may run any number of times.
                let mut env = env;
                if let Some(second) = second {
                    env.remove(second);
                }
                return self.loop_flow(Head::Range(name, Interval::TOP), body, env, collect);
            }
            Instruction::For(name, start, end, body) => {
                let first = self.expr(start, &env, collect);
                let bound = self.expr(end, &env, collect);
                // The counter lies in [start, end - 1]; an empty interval means zero trips.
                if first.lo > bound.hi - 1 {
                    return Flow {
                        normal: Some(env),
                        ..Flow::default()
                    };
                }
                if first.lo == first.hi && bound.lo == bound.hi {
                    if let Some(flow) =
                        self.finite_range(name, first.lo, bound.lo, body, &env, collect)
                    {
                        return flow;
                    }
                }
                let counter = Interval {
                    lo: first.lo,
                    hi: bound.hi - 1,
                };
                return self.loop_flow(Head::Range(name, counter), body, env, collect);
            }
        }
        Flow {
            normal: Some(env),
            ..Flow::default()
        }
    }
    fn loop_step(
        &mut self,
        head: Head,
        body: &Block,
        header: &Env,
        collect: bool,
    ) -> (State, Flow) {
        let (inside, outside) = match head {
            Head::Condition(condition) => self.condition(condition, header.clone(), collect),
            Head::Range(name, counter) => {
                // The checker keeps the counter immutable, so every trip sees the full range.
                let mut inside = header.clone();
                inside.insert(
                    name.to_owned(),
                    Fact {
                        value: counter,
                        length: None,
                    },
                );
                (Some(inside), Some(header.clone()))
            }
        };
        (
            outside,
            inside.map_or_else(Flow::default, |env| self.block(body, env, collect)),
        )
    }
    fn finite_loop(
        &mut self,
        condition: &Expr,
        body: &Block,
        entry: &Env,
        iterations: usize,
        collect: bool,
    ) -> Option<Flow> {
        let mut header = Some(entry.clone());
        let mut exits = None;
        let mut returns = None;
        for iteration in 0..=iterations {
            let Some(env) = header.take() else {
                break;
            };
            let (inside, outside) = self.condition(condition, env, collect);
            exits = join(exits, outside);
            if iteration == iterations {
                // A successful checked counter step must have exhausted the bound.
                if inside.is_some() {
                    return None;
                }
                break;
            }
            let flow = inside.map_or_else(Flow::default, |env| self.block(body, env, collect));
            exits = join(exits, flow.breaks);
            returns = join(returns, flow.returns);
            if flow.continues.is_some() {
                return None;
            }
            header = flow.normal;
        }
        Some(Flow {
            normal: exits,
            returns,
            ..Flow::default()
        })
    }
    /// Constant `for` bounds: visit each trip with the exact counter, like `finite_loop`
    /// does for recognized `while` counters. The counter cannot change inside the body.
    fn finite_range(
        &mut self,
        name: &str,
        first: i128,
        end: i128,
        body: &Block,
        entry: &Env,
        collect: bool,
    ) -> Option<Flow> {
        let cost = body_cost(body, &mut BTreeMap::new())?;
        if cost * (end - first + 1) > FINITE_TRANSFER_BUDGET {
            return None;
        }
        let mut header = Some(entry.clone());
        let mut exits = None;
        let mut returns = None;
        for value in first..end {
            let Some(mut env) = header.take() else {
                break;
            };
            env.insert(
                name.to_owned(),
                Fact {
                    value: Interval {
                        lo: value,
                        hi: value,
                    },
                    length: None,
                },
            );
            let flow = self.block(body, env, collect);
            exits = join(exits, flow.breaks);
            returns = join(returns, flow.returns);
            header = join(flow.normal, flow.continues);
        }
        let mut normal = join(exits, header);
        for env in [&mut normal, &mut returns].into_iter().flatten() {
            env.remove(name);
        }
        Some(Flow {
            normal,
            returns,
            ..Flow::default()
        })
    }
    fn loop_flow(&mut self, head: Head, body: &Block, entry: Env, collect: bool) -> Flow {
        // Speculative visits must never leak proofs into a failed finite analysis.
        if let Head::Condition(condition) = head {
            if let Some(iterations) = finite_iterations(condition, body, &entry) {
                let mut provisional = self.clone();
                if let Some(flow) =
                    provisional.finite_loop(condition, body, &entry, iterations, collect)
                {
                    *self = provisional;
                    return flow;
                }
            }
        }
        let mut header = entry.clone();
        for iteration in 0..8 {
            let (_, flow) = self.loop_step(head, body, &header, false);
            let next = join(Some(entry.clone()), join(flow.normal, flow.continues)).unwrap();
            if contained(&header, &next) {
                break;
            }
            let old = header.clone();
            header = join(Some(header), Some(next)).unwrap();
            if iteration >= 2 {
                for (name, fact) in &mut header {
                    fact.value = old[name].value.widen(fact.value);
                }
            }
            if iteration == 7 {
                forget_mutations(body, &mut header);
            }
        }
        // Verify inductiveness independently of convergence/iteration limits.
        let (_, check) = self.loop_step(head, body, &header, false);
        let next = join(Some(entry.clone()), join(check.normal, check.continues)).unwrap();
        if !contained(&header, &next) {
            forget_mutations(body, &mut header);
            let (_, check) = self.loop_step(head, body, &header, false);
            let next = join(Some(entry), join(check.normal, check.continues)).unwrap();
            if !contained(&header, &next) {
                // Unsupported mutation patterns must never authorize narrow facts.
                for fact in header.values_mut() {
                    fact.value = Interval::TOP;
                }
            }
        }
        let (outside, flow) = self.loop_step(head, body, &header, collect);
        Flow {
            normal: join(outside, flow.breaks),
            returns: flow.returns,
            ..Flow::default()
        }
    }
}
// Bound expression/instruction transfers per recognized loop, including its final condition.
const FINITE_TRANSFER_BUDGET: i128 = 32768;

fn scalar_variables(expr: &Expr) -> Option<BTreeSet<String>> {
    match expr {
        Expr::Int(_) => Some(BTreeSet::new()),
        Expr::Variable(name) => Some(BTreeSet::from([name.clone()])),
        Expr::Unary('-', inner) => scalar_variables(inner),
        Expr::Binary(op, left, right) if matches!(op.as_str(), "+" | "-" | "*" | "/" | "%") => {
            let mut names = scalar_variables(left)?;
            names.extend(scalar_variables(right)?);
            Some(names)
        }
        _ => None,
    }
}

fn refine_bound(env: &Env, name: &str, lo: i128, hi: i128) -> State {
    let mut env = env.clone();
    if let Some(fact) = env.get_mut(name) {
        fact.value = fact.value.intersect(lo, hi)?;
    }
    Some(env)
}

fn expression_cost(expr: &Expr) -> Option<i128> {
    let children = match expr {
        // Hidden handler control flow and mutable borrows are outside this recognizer.
        Expr::Catch(_, _, _) | Expr::Borrow(_, true) | Expr::Match(_, _) => return None,
        Expr::Variant(_, _, values) => values
            .iter()
            .try_fold(0, |cost, value| Some(cost + expression_cost(value)?))?,
        Expr::Unary(_, inner)
        | Expr::Try(inner)
        | Expr::ByteLen(inner)
        | Expr::CharLen(inner)
        | Expr::Index(_, inner)
        | Expr::IndexField(_, inner, _)
        | Expr::BorrowIndex(_, inner, _, _) => expression_cost(inner)?,
        Expr::Binary(_, a, b) | Expr::StringEqual(a, b) => {
            expression_cost(a)? + expression_cost(b)?
        }
        Expr::Record(_, fields) => fields
            .iter()
            .try_fold(0, |cost, (_, value)| Some(cost + expression_cost(value)?))?,
        Expr::MapLit(entries) => entries.iter().try_fold(0, |cost, (key, value)| {
            Some(cost + expression_cost(key)? + expression_cost(value)?)
        })?,
        Expr::If(condition, yes, no) => {
            expression_cost(condition)? + expression_cost(yes)? + expression_cost(no)?
        }
        Expr::Call(_, values)
        | Expr::Method(_, _, values)
        | Expr::Array(values)
        | Expr::Tuple(values)
        | Expr::Format(values) => values
            .iter()
            .try_fold(0, |cost, value| Some(cost + expression_cost(value)?))?,
        Expr::ElementMethod(_, index, _, _, values) | Expr::Chain(index, _, values) => values
            .iter()
            .try_fold(expression_cost(index)?, |cost, value| {
                Some(cost + expression_cost(value)?)
            })?,
        _ => 0,
    };
    Some(1 + children)
}

fn body_cost(block: &Block, writes: &mut BTreeMap<String, usize>) -> Option<i128> {
    block.iter().try_fold(0, |cost, (instruction, _)| {
        let next = match instruction {
            Instruction::Continue
            | Instruction::While(_, _)
            | Instruction::For(_, _, _, _)
            | Instruction::Each(_, _, _, _, _)
            | Instruction::Unpack(_, _, _)
            | Instruction::Match(_, _) => return None,
            Instruction::Let(_, _, _, expr)
            | Instruction::Print(expr)
            | Instruction::Call(expr)
            | Instruction::Fail(expr) => expression_cost(expr)?,
            Instruction::Assign(name, expr) | Instruction::AssignField(name, _, expr) => {
                *writes.entry(name.clone()).or_default() += 1;
                expression_cost(expr)?
            }
            Instruction::AssignIndex(name, index, value)
            | Instruction::AssignIndexField(name, index, _, value) => {
                *writes.entry(name.clone()).or_default() += 1;
                expression_cost(index)? + expression_cost(value)?
            }
            Instruction::Return(expr) => expr.as_ref().map_or(Some(0), expression_cost)?,
            Instruction::If(condition, yes, no) => {
                expression_cost(condition)? + body_cost(yes, writes)? + body_cost(no, writes)?
            }
            Instruction::Break => 0,
        };
        Some(cost + 1 + next)
    })
}

fn finite_iterations(condition: &Expr, body: &Block, entry: &Env) -> Option<usize> {
    let Expr::Binary(op, left, bound) = condition else {
        return None;
    };
    if !matches!(op.as_str(), "<" | "<=") {
        return None;
    }
    let Expr::Variable(counter) = left.as_ref() else {
        return None;
    };
    let Instruction::Assign(name, Expr::Binary(step_op, step_left, step_right)) = &body.last()?.0
    else {
        return None;
    };
    if name != counter
        || step_op != "+"
        || !matches!(step_left.as_ref(), Expr::Variable(name) if name == counter)
    {
        return None;
    }
    let step = literal(step_right)?;
    if step <= 0 {
        return None;
    }
    let variables = scalar_variables(bound)?;
    let mut writes = BTreeMap::new();
    let cost = expression_cost(condition)? + body_cost(body, &mut writes)?;
    if writes.get(counter) != Some(&1) || variables.iter().any(|name| writes.contains_key(name)) {
        return None;
    }
    let initial = entry.get(counter)?.value;
    if initial == Interval::TOP || variables.iter().any(|name| !entry.contains_key(name)) {
        return None;
    }
    let upper = Analysis::default().expr(bound, entry, false).hi;
    let distance = (upper - initial.lo + i128::from(op == "<=")).max(0);
    let iterations = (distance + step - 1) / step;
    // Include the final condition evaluation, and cap all body traversal work.
    if cost * (iterations + 1) > FINITE_TRANSFER_BUDGET {
        return None;
    }
    usize::try_from(iterations).ok()
}

/// match 分支的环境：载荷绑定没有区间事实。
fn opaque(env: &Env, bindings: &[String]) -> Env {
    let mut env = env.clone();
    for binding in bindings {
        env.insert(
            binding.clone(),
            Fact {
                value: Interval::TOP,
                length: None,
            },
        );
    }
    env
}
fn contained(header: &Env, next: &Env) -> bool {
    next.iter().all(|(name, fact)| {
        header
            .get(name)
            .is_some_and(|old| old.value.contains(fact.value) && old.length == fact.length)
    })
}
fn forget_mutations(block: &Block, env: &mut Env) {
    for (instruction, _) in block {
        match instruction {
            Instruction::Assign(name, _) => {
                if let Some(fact) = env.get_mut(name) {
                    fact.value = Interval::TOP;
                }
            }
            Instruction::If(_, a, b) => {
                forget_mutations(a, env);
                forget_mutations(b, env);
            }
            Instruction::While(_, body)
            | Instruction::For(_, _, _, body)
            | Instruction::Each(_, _, _, _, body) => forget_mutations(body, env),
            Instruction::Match(_, arms) => {
                for (_, body) in arms {
                    forget_mutations(body, env);
                }
            }
            _ => {}
        }
    }
}
fn literal(expr: &Expr) -> Option<i128> {
    match expr {
        Expr::Int(value) => Some(*value as i128),
        Expr::Unary('-', inner) => literal(inner).and_then(|value| {
            let value = -value;
            (MIN..=MAX).contains(&value).then_some(value)
        }),
        _ => None,
    }
}
fn refine(expr: &Expr, mut env: Env, truth: bool) -> State {
    let Expr::Binary(op, left, right) = expr else {
        return Some(env);
    };
    let (name, value, reversed) = match (left.as_ref(), right.as_ref()) {
        (Expr::Variable(name), rhs) if literal(rhs).is_some() => {
            (name, literal(rhs).unwrap(), false)
        }
        (lhs, Expr::Variable(name)) if literal(lhs).is_some() => {
            (name, literal(lhs).unwrap(), true)
        }
        _ => return Some(env),
    };
    let op = if reversed {
        match op.as_str() {
            "<" => ">",
            "<=" => ">=",
            ">" => "<",
            ">=" => "<=",
            other => other,
        }
    } else {
        op.as_str()
    };
    let op = if truth {
        op
    } else {
        match op {
            "<" => ">=",
            "<=" => ">",
            ">" => "<=",
            ">=" => "<",
            "==" => "!=",
            "!=" => "==",
            other => other,
        }
    };
    let Some(fact) = env.get_mut(name) else {
        return Some(env);
    };
    let (lo, hi) = match op {
        "<" => (MIN, value - 1),
        "<=" => (MIN, value),
        ">" => (value + 1, MAX),
        ">=" => (value, MAX),
        "==" => (value, value),
        "!=" if fact.value.lo == value => (value + 1, MAX),
        "!=" if fact.value.hi == value => (MIN, value - 1),
        _ => return Some(env),
    };
    fact.value = fact.value.intersect(lo, hi)?;
    Some(env)
}
pub(super) fn analyze(function: &Function) -> Proofs {
    analyze_with_ranges(function, &[])
}
pub(super) fn analyze_with_ranges(function: &Function, inputs: &[Interval]) -> Proofs {
    let seeded = inputs.len() == function.params.len()
        && function.params.iter().all(|(_, ty)| *ty == Type::Int)
        && inputs
            .iter()
            .all(|input| input.lo <= input.hi && input.lo >= MIN && input.hi <= MAX);
    let env = function
        .params
        .iter()
        .enumerate()
        .map(|(i, (name, ty))| {
            (
                name.clone(),
                Fact {
                    value: if seeded { inputs[i] } else { Interval::TOP },
                    length: if let Type::ArrayInt(length) = ty {
                        Some(*length)
                    } else {
                        None
                    },
                },
            )
        })
        .collect();
    let mut analysis = Analysis::default();
    analysis.block(&function.body, env, true);
    let safe = |map: BTreeMap<usize, bool>| {
        map.into_iter()
            .filter_map(|(key, safe)| safe.then_some(key))
            .collect()
    };
    Proofs {
        arithmetic: safe(analysis.arithmetic),
        unsigned_division: safe(analysis.unsigned_division),
        index: safe(analysis.index),
        assignment_index: safe(analysis.assignment_index),
        calls: analysis.calls,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn function(body: Vec<Instruction>) -> Function {
        Function {
            public: false,
            params: vec![],
            result: Type::Unit,
            error: None,
            body: body.into_iter().map(|i| (i, (1, 1))).collect(),
            position: (1, 1),
            builtin: false,
            generics: Vec::new(),
            external: false,
        }
    }
    fn binary(op: &str, a: Expr, b: Expr) -> Expr {
        Expr::Binary(op.into(), Box::new(a), Box::new(b))
    }
    fn var(name: &str) -> Expr {
        Expr::Variable(name.into())
    }
    #[test]
    fn failed_finite_analysis_rolls_back_call_inputs_before_widening() {
        let module = crate::compiler::parse("fn main():\n    let mut i = 9223372036854775806\n    while i <= 9223372036854775807:\n        probe(i)\n        i = i + 1").unwrap();
        let f = &module.functions["main"];
        let Instruction::While(_, body) = &f.body[1].0 else {
            panic!()
        };
        let Instruction::Call(call) = &body[0].0 else {
            panic!()
        };
        let inputs = analyze(f);
        let range = inputs.call_inputs(call).unwrap()[0];
        assert_eq!(range.hi, MAX);
        assert!(range.lo < MAX);
    }
    #[test]
    fn for_ranges_bound_the_counter_but_not_outer_accumulators() {
        fn body(f: &Function, index: usize) -> &Block {
            let Instruction::For(_, _, _, body) = &f.body[index].0 else {
                panic!()
            };
            body
        }
        fn printed(body: &Block, index: usize) -> &Expr {
            let Instruction::Print(expr) = &body[index].0 else {
                panic!()
            };
            expr
        }
        fn assigned(body: &Block, index: usize) -> &Expr {
            let Instruction::Assign(_, expr) = &body[index].0 else {
                panic!()
            };
            expr
        }
        let module = crate::compiler::parse("fn main(n: i64):\n    let a = [3, 1, 4]\n    let mut total = 0\n    for i in 0..3:\n        println(a[i])\n        total = total + i\n    for j in 0..n:\n        println(a[j])\n        println(j + 1)\n        total = total + j\n    for k in 9223372036854775806..9223372036854775807:\n        println(k + 1)\n        println(k + 2)\n    for m in -3..n:\n        println(a[m])").unwrap();
        let f = &module.functions["main"];
        let proofs = analyze(f);
        // Constant bounds are visited trip by trip: indexing and the accumulator are exact.
        assert!(proofs.index(printed(body(f, 2), 0)));
        assert!(proofs.arithmetic(assigned(body(f, 2), 1)));
        // An unknown bound still caps the counter at i64::MAX - 1, but not indexing
        // or an accumulator that grows across trips.
        assert!(!proofs.index(printed(body(f, 3), 0)));
        assert!(proofs.arithmetic(printed(body(f, 3), 1)));
        assert!(!proofs.arithmetic(assigned(body(f, 3), 2)));
        // The end is exclusive: k + 1 reaches i64::MAX exactly, k + 2 overflows.
        assert!(proofs.arithmetic(printed(body(f, 4), 0)));
        assert!(!proofs.arithmetic(printed(body(f, 4), 1)));
        // A negative start rules out the index proof.
        assert!(!proofs.index(printed(body(f, 5), 0)));
    }
    #[test]
    fn malformed_seed_inputs_fail_closed() {
        let mut f = function(vec![Instruction::Print(binary(
            "+",
            var("n"),
            Expr::Int(1),
        ))]);
        f.params.push(("n".into(), Type::Int));
        let Instruction::Print(sum) = &f.body[0].0 else {
            panic!()
        };
        for inputs in [
            vec![],
            vec![Interval { lo: 3, hi: 2 }],
            vec![Interval { lo: MIN - 1, hi: 0 }],
        ] {
            assert!(!analyze_with_ranges(&f, &inputs).arithmetic(sum));
        }
    }
    #[test]
    fn nonnegative_division_proofs_and_codegen_keep_signed_fallback() {
        let module = crate::compiler::parse(
            "fn main(n: i64, d: i64):\n    println(n / d)\n    println(n % d)",
        )
        .unwrap();
        let f = &module.functions["main"];
        for (inputs, expected) in [
            (
                vec![Interval { lo: 0, hi: MAX }, Interval { lo: 1, hi: MAX }],
                true,
            ),
            (vec![Interval::TOP, Interval { lo: 1, hi: MAX }], false),
            (
                vec![Interval { lo: 0, hi: MAX }, Interval { lo: 0, hi: MAX }],
                false,
            ),
            (
                vec![Interval { lo: 0, hi: MAX }, Interval::point(-1)],
                false,
            ),
        ] {
            let proofs = analyze_with_ranges(f, &inputs);
            for (instruction, _) in &f.body {
                let Instruction::Print(expr) = instruction else {
                    panic!()
                };
                assert_eq!(proofs.unsigned_division(expr), expected);
            }
        }
        for (source, unsigned) in [
            (
                "fn main():\n    let n = 9223372036854775807\n    println(n / 4)\n    println(n % 4)",
                true,
            ),
            (
                "fn main(n: i64):\n    println(n / 3)\n    println(n % 3)",
                false,
            ),
        ] {
            let mut module = crate::compiler::parse(source).unwrap();
            let f = module.functions.remove("main").unwrap();
            module.functions.insert("case:main".into(), f);
            let generated = super::super::generate(&module, "case:main", "case");
            let body = generated
                .split_once(if unsigned {
                    "void f636173653a6d61696e(void) {\n"
                } else {
                    "void f636173653a6d61696e(int64_t p0) {\n"
                })
                .unwrap()
                .1
                .split_once("\n}\n")
                .unwrap()
                .0;
            assert_eq!(body.contains("(int64_t)((uint64_t)"), unsigned, "{body}");
            assert!(body.contains(" / "), "{body}");
            assert!(body.contains(" % "), "{body}");
        }
    }

    #[test]
    fn dynamic_positive_divisors_keep_runtime_width_dispatch() {
        let mut module = crate::compiler::parse(
            "fn main():\n    let n = 9223372036854775807\n    let d = 3\n    println(n / d)\n    println(n % d)",
        ).unwrap();
        let f = module.functions.remove("main").unwrap();
        module.functions.insert("case:main".into(), f);
        let generated = super::super::generate(&module, "case:main", "case");
        let body = generated
            .split_once("void f636173653a6d61696e(void) {\n")
            .unwrap()
            .1;
        assert!(body.contains("quotient("), "{body}");
        assert!(body.contains("remainder("), "{body}");
        assert!(!body.contains("(int64_t)((uint64_t)"), "{body}");
    }

    #[test]
    fn nonnegative_division_repeated_visits_and_rollback_fail_closed() {
        for source in [
            "fn main():\n    let mut i = 0\n    let mut n = 3\n    while i < 2:\n        println(n % 7)\n        n = -3\n        i = i + 1",
            "fn main():\n    let mut i = 0\n    let mut n = 3\n    while i < 2:\n        println(n / 7)\n        n = -3\n        i = i + 1",
            "fn main():\n    let mut i = 9223372036854775806\n    let mut n = 3\n    while i <= 9223372036854775807:\n        println(n % 7)\n        n = -3\n        i = i + 1",
        ] {
            let module = crate::compiler::parse(source).unwrap();
            let f = &module.functions["main"];
            let Instruction::While(_, body) = &f.body.last().unwrap().0 else { panic!() };
            let Instruction::Print(expr) = &body[0].0 else { panic!() };
            assert!(!analyze(f).unsigned_division(expr));
        }
    }

    #[test]
    fn successful_operand_interval_never_removes_inner_overflow_check() {
        let module =
            crate::compiler::parse("fn main(n: i64):\n    if n >= 0: println((n + 1) / 4)")
                .unwrap();
        let f = &module.functions["main"];
        let Instruction::If(_, body, _) = &f.body[0].0 else {
            panic!()
        };
        let Instruction::Print(expr) = &body[0].0 else {
            panic!()
        };
        let Expr::Binary(_, sum, _) = expr else {
            panic!()
        };
        let proofs = analyze(f);
        assert!(proofs.unsigned_division(expr));
        assert!(!proofs.arithmetic(sum));
    }

    #[test]
    fn specialized_nonnegative_recurrence_selects_unsigned_remainder() {
        let module = crate::compiler::parse(
            "fn accum(seed: i64) -> i64:\n    let mut acc = seed\n    let mut i = 0\n    while i < 512:\n        acc = (acc * 31 + i) % 1000003\n        i = i + 1\n    acc\n",
        ).unwrap();
        let f = &module.functions["accum"];
        let Instruction::While(_, body) = &f.body[2].0 else {
            panic!()
        };
        let Instruction::Assign(_, expr) = &body[0].0 else {
            panic!()
        };
        assert!(analyze_with_ranges(f, &[Interval { lo: 0, hi: 7 }]).unsigned_division(expr));
        assert!(!analyze(f).unsigned_division(expr));
    }

    #[test]
    fn endpoints_and_checked_success() {
        assert!(!Interval::TOP.arithmetic("+", Interval::point(1)).1);
        assert_eq!(
            Interval::TOP.arithmetic("+", Interval::point(1)).0.lo,
            MIN + 1
        );
        assert!(
            !Interval::point(i64::MIN)
                .arithmetic("*", Interval::point(-1))
                .1
        );
        assert!(Interval::TOP.arithmetic("*", Interval::point(0)).1);
        assert_eq!(
            Interval { lo: -3, hi: 2 }
                .arithmetic("*", Interval { lo: -4, hi: 5 })
                .0,
            Interval { lo: -15, hi: 12 }
        );
    }
    #[test]
    fn positive_remainder_describes_success_without_authorizing_checks() {
        for (input, expected) in [
            (Interval { lo: 0, hi: MAX }, Interval { lo: 0, hi: 39 }),
            (Interval { lo: MIN, hi: 0 }, Interval { lo: -39, hi: 0 }),
            (Interval::TOP, Interval { lo: -39, hi: 39 }),
        ] {
            assert_eq!(
                input.arithmetic("%", Interval::point(40)),
                (expected, false)
            );
        }
        assert_eq!(
            Interval::TOP.arithmetic("%", Interval::point(0)),
            (Interval::TOP, false)
        );
        assert_eq!(
            Interval::TOP.arithmetic("%", Interval::point(-1)),
            (Interval::TOP, false)
        );
    }

    #[test]
    fn finite_recurrence_generates_raw_arithmetic_for_unknown_remainder_bound() {
        let mut module = crate::compiler::parse(
            "fn main(seed: i64):\n    let bound = seed % 40\n    let mut i = 0\n    let mut a = 0\n    let mut b = 1\n    while i < bound:\n        let next = a + b\n        a = b\n        b = next\n        i = i + 1\n    println(a)",
        ).unwrap();
        let function = module.functions.remove("main").unwrap();
        module.functions.insert("case:main".into(), function);
        // Analysis must use the same, stable AST that generation traverses.
        let source = super::super::generate(&module, "case:main", "case");
        let body = source.rsplit_once("f636173653a6d61696e(int64_t").unwrap().1;
        let body = body.split_once("{\n").unwrap().1;
        assert!(!body.contains("add("), "{body}");
        assert!(body.contains(" + "), "{body}");
        assert!(body.contains(" % "), "{body}");
    }

    #[test]
    fn finite_visits_and_failed_speculation_never_keep_first_visit_proofs() {
        for source in [
            "fn main():\n    let mut i = 0\n    let mut x = 9223372036854775806\n    while i < 2:\n        x = x + 1\n        i = i + 1",
            "fn main():\n    let mut i = 9223372036854775806\n    while i <= 9223372036854775807:\n        i = i + 1",
        ] {
            let module = crate::compiler::parse(source).unwrap();
            let f = &module.functions["main"];
            let proofs = analyze(f);
            let Instruction::While(_, body) = &f.body.last().unwrap().0 else { panic!() };
            let Instruction::Assign(_, sum) = &body[0].0 else { panic!() };
            assert!(!proofs.arithmetic(sum));
        }
    }

    #[test]
    fn finite_recognizer_rejects_mutation_and_continue() {
        for source in [
            "fn main():\n    let mut bound = 2\n    let mut i = 0\n    while i < bound:\n        bound = bound + 1\n        i = i + 1",
            "fn main():\n    let mut i = 0\n    while i < 2:\n        if i == 0: continue\n        i = i + 1",
            "fn main():\n    let mut i = 0\n    while i < 2:\n        if i == 0: i = 1\n        i = i + 1",
            "fn main():\n    let mut i = 0\n    while i < 2:\n        while false: pass\n        i = i + 1",
        ] {
            let module = crate::compiler::parse(source).unwrap();
            let f = &module.functions["main"];
            let mut analysis = Analysis::default();
            let mut env = Env::new();
            for (instruction, _) in &f.body[..f.body.len() - 1] {
                env = analysis.instruction(instruction, env, false).normal.unwrap();
            }
            let Instruction::While(condition, body) = &f.body.last().unwrap().0 else { panic!() };
            assert!(finite_iterations(condition, body, &env).is_none());
        }
    }

    #[test]
    fn bounded_loop_and_array_proofs() {
        let f = function(vec![
            Instruction::Let("a".into(), true, None, Expr::Array(vec![Expr::Int(0); 4])),
            Instruction::Let("i".into(), true, None, Expr::Int(0)),
            Instruction::While(
                binary("<", var("i"), Expr::Int(4)),
                vec![
                    (
                        Instruction::AssignIndex(
                            "a".into(),
                            var("i"),
                            binary("*", var("i"), Expr::Int(2)),
                        ),
                        (1, 1),
                    ),
                    (
                        Instruction::Print(Expr::Index("a".into(), Box::new(var("i")))),
                        (1, 1),
                    ),
                    (
                        Instruction::Assign("i".into(), binary("+", var("i"), Expr::Int(1))),
                        (1, 1),
                    ),
                ],
            ),
        ]);
        let proofs = analyze(&f);
        let Instruction::While(_, body) = &f.body[2].0 else {
            panic!()
        };
        assert!(proofs.assignment_index(&body[0].0));
        let Instruction::Print(index) = &body[1].0 else {
            panic!()
        };
        assert!(proofs.index(index));
        let Instruction::Assign(_, sum) = &body[2].0 else {
            panic!()
        };
        assert!(proofs.arithmetic(sum));
    }
    #[test]
    fn short_circuit_refines_only_the_right_operand() {
        let guarded_sum = binary("+", var("n"), Expr::Int(1));
        let f = Function {
            params: vec![("n".into(), Type::Int)],
            ..function(vec![
                Instruction::If(
                    binary(
                        "&&",
                        binary("<", var("n"), Expr::Int(i64::MAX)),
                        binary(">", guarded_sum, Expr::Int(0)),
                    ),
                    vec![],
                    vec![],
                ),
                Instruction::If(
                    binary("<", binary("+", var("n"), Expr::Int(1)), Expr::Int(10)),
                    vec![],
                    vec![],
                ),
            ])
        };
        let proofs = analyze(&f);
        let Instruction::If(Expr::Binary(_, _, rhs), _, _) = &f.body[0].0 else {
            panic!()
        };
        let Expr::Binary(_, sum, _) = rhs.as_ref() else {
            panic!()
        };
        assert!(proofs.arithmetic(sum));
        let Instruction::If(Expr::Binary(_, sum, _), _, _) = &f.body[1].0 else {
            panic!()
        };
        assert!(!proofs.arithmetic(sum));
    }
    #[test]
    fn break_and_continue_carry_outer_assignments() {
        for jump in [Instruction::Break, Instruction::Continue] {
            let f = function(vec![
                Instruction::Let("n".into(), true, None, Expr::Int(0)),
                Instruction::While(
                    binary("<", var("n"), Expr::Int(2)),
                    vec![
                        (Instruction::Assign("n".into(), Expr::Int(i64::MAX)), (1, 1)),
                        (jump, (1, 1)),
                    ],
                ),
                Instruction::Print(binary("+", var("n"), Expr::Int(1))),
            ]);
            let Instruction::Print(sum) = &f.body[2].0 else {
                panic!()
            };
            assert!(!analyze(&f).arithmetic(sum));
        }
    }
    #[test]
    fn scope_retains_assignments_but_discards_locals() {
        let f = function(vec![
            Instruction::Let("n".into(), true, None, Expr::Int(0)),
            Instruction::If(
                Expr::Bool(true),
                vec![
                    (
                        Instruction::Let("temporary".into(), false, None, Expr::Int(i64::MAX)),
                        (1, 1),
                    ),
                    (Instruction::Assign("n".into(), var("temporary")), (1, 1)),
                ],
                vec![],
            ),
            Instruction::Print(binary("+", var("n"), Expr::Int(1))),
        ]);
        let Instruction::Print(sum) = &f.body[2].0 else {
            panic!()
        };
        assert!(!analyze(&f).arithmetic(sum));
    }
    #[test]
    fn params_and_loop_backedges_are_not_seeded() {
        let mut f = function(vec![Instruction::Print(binary(
            "+",
            var("n"),
            Expr::Int(1),
        ))]);
        f.params.push(("n".into(), Type::Int));
        let Instruction::Print(sum) = &f.body[0].0 else {
            panic!()
        };
        assert!(!analyze(&f).arithmetic(sum));
        let f = function(vec![
            Instruction::Let("n".into(), true, None, Expr::Int(0)),
            Instruction::While(
                Expr::Bool(true),
                vec![(
                    Instruction::Assign("n".into(), binary("+", var("n"), Expr::Int(1))),
                    (1, 1),
                )],
            ),
        ]);
        let Instruction::While(_, body) = &f.body[1].0 else {
            panic!()
        };
        let Instruction::Assign(_, sum) = &body[0].0 else {
            panic!()
        };
        assert!(!analyze(&f).arithmetic(sum));
    }
}
