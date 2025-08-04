use crate::util::{HashMap, HashSet};
use crate::{parser::*, util::*};
use ark_r1cs_std::{GR1CSVar, alloc::AllocVar, boolean::Boolean, fields::fp::FpVar};
use ark_relations::gr1cs::ConstraintSystemRef;
use ark_relations::gr1cs::SynthesisError;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::vec::Vec;
use petgraph::{Direction, visit::EdgeRef};
use segmented_circuit_memory::memory::mem_type::MemType;
use segmented_circuit_memory::memory::nebula::{MemBuilder, RunningMem};
use sha2::{Digest, Sha256};
use std::{cmp::max, usize};

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};

#[derive(Clone, Debug, CanonicalDeserialize, CanonicalSerialize)]
pub struct NodeElem<F: ArkPrimeField> {
    pub id: usize,
    pub terminal: bool, //terminal = 1, not = 0
    pub symbol: F,
    pub sib: usize,
    pub child: usize,
    pub parent: usize,
}

impl<F: ArkPrimeField> NodeElem<F> {
    pub fn new(
        id: usize,
        terminal: bool,
        symbol: F,
        sib_id: usize,
        child: usize,
        parent: usize,
    ) -> Self {
        NodeElem {
            id,
            terminal,
            symbol,
            sib: sib_id,
            child,
            parent,
        }
    }

    pub fn mem_init(self, csc: &CoralStepCircuit<F>, mem_builder: &mut MemBuilder<F>) {
        let addr = self.id + csc.tree_ram_offset;
        let vals = vec![
            to_F(self.terminal as usize),
            self.symbol,
            to_F(self.sib),
            to_F(self.child),
            to_F(self.parent),
        ];
        mem_builder.init(addr, vals, csc.tree_ram_tag);
    }
}

pub fn coral_hash<F: ArkPrimeField>(obj: &str) -> F {
    let mut out: F;
    if obj.len() == 1 {
        let char = obj.chars().next().unwrap();
        out = F::from(char as u32);
    } else {
        let hash = Sha256::digest(obj.as_bytes());
        out = F::from_le_bytes_mod_order(&hash);
    }
    assert!(out > F::ZERO);
    if obj.is_empty() {
        out = F::zero();
    }
    out
}

pub fn make_node_elem<F: ArkPrimeField>(id: usize, g: &GrammarGraph) -> NodeElem<F> {
    let n = g.get_node(id).unwrap();
    let node_index = g
        .lcrs_tree
        .node_indices()
        .find(|&i| g.lcrs_tree[i].id == id)
        .unwrap();

    let has_child = g
        .lcrs_tree
        .edges_directed(node_index, Direction::Outgoing)
        .find(|edge| edge.weight() == &EdgeType::Child);

    let child_index = match has_child {
        Some(e) => e.target().index(),
        None => g.lcrs_tree.node_count(),
    };

    let has_sib = g
        .lcrs_tree
        .edges_directed(node_index, Direction::Outgoing)
        .find(|edge| edge.weight() == &EdgeType::Sibling);
    let sib_index = match has_sib {
        Some(e) => e.target().index(),
        None => g.lcrs_tree.node_count(),
    };

    let parent = match n.parent_id {
        Some(x) => x,
        None => g.lcrs_tree.node_count(),
    };

    NodeElem::new(
        id,
        n.is_terminal,
        coral_hash(&n.rule_name),
        sib_index,
        child_index,
        parent,
    )
}

#[derive(Clone, Debug)]
pub struct CoralWires<F: ArkPrimeField> {
    pub cur_node_id: FpVar<F>,
    pub running_eval: FpVar<F>,
    pub parent_id: FpVar<F>,
    pub np_rule: FpVar<F>,
    pub np_parent_id: FpVar<F>,
    pub np_sp: FpVar<F>,
    pub atom_flag: Boolean<F>,
    pub atom_parent_id: FpVar<F>,
    pub atom_sp: FpVar<F>,
    pub count: FpVar<F>,
    pub doc_ctr: FpVar<F>,
    pub prev_t_sp: FpVar<F>,
    pub prev_step_t_ops: Boolean<F>,
}

pub fn print_wires<F: ArkPrimeField>(wires: &CoralWires<F>) {
    println!("cur_node_id: {:?}", wires.cur_node_id.value().unwrap());
    println!("running_eval: {:?}", wires.running_eval.value().unwrap());
    println!("parent_id: {:?}", wires.parent_id.value().unwrap());
    println!("np_rule: {:?}", wires.np_rule.value().unwrap());
    println!("np_parent_id: {:?}", wires.np_parent_id.value().unwrap());
    println!("atom_flag: {:?}", wires.atom_flag.value().unwrap());
    println!(
        "atom_parent_id: {:?}",
        wires.atom_parent_id.value().unwrap()
    );
    println!("count: {:?}", wires.count.value().unwrap());
    println!("doc_ctr: {:?}", wires.doc_ctr.value().unwrap());
    println!("prev_t_sp: {:?}", wires.prev_t_sp.value().unwrap());
    println!(
        "prev_step_t_ops: {:?}",
        wires.prev_step_t_ops.value().unwrap()
    );
    println!("np_sp: {:?}", wires.np_sp.value().unwrap());
    println!("atom_sp: {:?}", wires.atom_sp.value().unwrap());
}

impl<F: ArkPrimeField> CoralWires<F> {
    pub fn clean_copy(old_wires: CoralWires<F>) -> CoralWires<F> {
        CoralWires {
            cur_node_id: old_wires.cur_node_id.clone(),
            running_eval: old_wires.running_eval.clone(),
            parent_id: old_wires.parent_id.clone(),
            np_rule: old_wires.np_rule.clone(),
            np_parent_id: old_wires.np_parent_id.clone(),
            atom_flag: old_wires.atom_flag.clone(),
            atom_parent_id: old_wires.atom_parent_id.clone(),
            count: old_wires.count.clone(),
            doc_ctr: old_wires.doc_ctr.clone(),
            prev_t_sp: old_wires.prev_t_sp.clone(),
            prev_step_t_ops: old_wires.prev_step_t_ops.clone(),
            np_sp: old_wires.np_sp.clone(),
            atom_sp: old_wires.atom_sp.clone(),
        }
    }

    pub fn wires_from_irw(
        irw: &InterRoundWires<F>,
        cs: ConstraintSystemRef<F>,
        csc: &mut CoralStepCircuit<F>,
        i: usize,
    ) -> CoralWires<F> {
        CoralWires {
            parent_id: FpVar::<F>::new_witness(cs.clone(), || {
                Ok(to_F::<F>(csc.parent_node_wits[i * csc.batch_size].id))
            })
            .unwrap(),
            cur_node_id: FpVar::<F>::new_witness(cs.clone(), || {
                Ok(to_F::<F>(csc.node_wits[i * csc.batch_size].id))
            })
            .unwrap(),
            count: FpVar::<F>::new_witness(cs.clone(), || Ok(irw.count)).unwrap(),
            running_eval: FpVar::<F>::new_witness(cs.clone(), || Ok(irw.running_eval)).unwrap(),
            atom_parent_id: FpVar::<F>::new_witness(cs.clone(), || Ok(irw.atom_parent_id)).unwrap(),
            atom_flag: Boolean::<F>::new_witness(cs.clone(), || Ok(irw.atom_flag)).unwrap(),
            np_parent_id: FpVar::<F>::new_witness(cs.clone(), || Ok(irw.np_parent_id)).unwrap(),
            np_rule: FpVar::<F>::new_witness(cs.clone(), || Ok(irw.np_rule)).unwrap(),
            doc_ctr: FpVar::<F>::new_witness(cs.clone(), || Ok(irw.doc_ctr)).unwrap(),
            prev_t_sp: FpVar::<F>::new_witness(cs.clone(), || Ok(irw.prev_t_sp)).unwrap(),
            prev_step_t_ops: Boolean::<F>::new_witness(cs.clone(), || Ok(irw.prev_step_t_ops))
                .unwrap(),
            np_sp: FpVar::<F>::new_witness(cs.clone(), || Ok(irw.np_sp)).unwrap(),
            atom_sp: FpVar::<F>::new_witness(cs.clone(), || Ok(irw.atom_sp)).unwrap(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InterRoundWires<F: ArkPrimeField> {
    pub running_eval: F,
    pub np_rule: F,
    pub np_parent_id: F,
    pub np_sp: F,
    pub atom_flag: bool,
    pub atom_parent_id: F,
    pub atom_sp: F,
    pub prev_t_sp: F,
    pub prev_step_t_ops: bool,
    pub count: F,
    pub doc_ctr: F,
}

impl<F: ArkPrimeField> Default for InterRoundWires<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: ArkPrimeField> InterRoundWires<F> {
    pub fn new() -> Self {
        InterRoundWires {
            running_eval: F::ONE,
            count: F::ZERO,
            atom_parent_id: F::ZERO,
            atom_flag: false,
            np_parent_id: F::ZERO,
            np_rule: F::ZERO,
            doc_ctr: F::ZERO,
            prev_t_sp: F::ONE,
            prev_step_t_ops: false,
            atom_sp: F::ZERO,
            np_sp: F::ZERO,
        }
    }

    pub fn update(&mut self, res: CoralWires<F>) {
        self.running_eval = res.running_eval.value().unwrap();
        self.doc_ctr = res.doc_ctr.value().unwrap();
        self.atom_parent_id = res.atom_parent_id.value().unwrap();
        self.atom_flag = res.atom_flag.value().unwrap();
        self.np_parent_id = res.np_parent_id.value().unwrap();
        self.np_rule = res.np_rule.value().unwrap();
        self.count = res.count.value().unwrap();
        self.prev_t_sp = res.prev_t_sp.value().unwrap();
        self.prev_step_t_ops = res.prev_step_t_ops.value().unwrap();
        self.atom_sp = res.atom_sp.value().unwrap();
        self.np_sp = res.np_sp.value().unwrap();
    }
}

pub fn make_rule_vector<F: ArkPrimeField>(g: &GrammarGraph) -> Vec<Vec<F>> {
    assert!((g.rule_count as u32) < u32::MAX);
    let mut out: Vec<Vec<F>> = Vec::new();
    for (rule_name, rules) in g.rules.iter() {
        let is_atomic = g.atom.contains(&rule_name.clone());
        let is_np = g.np_rule_names.contains(&rule_name.clone());
        for i in 0..rules.len() {
            let mut rule: Vec<F> = rules[i].iter().map(|x| coral_hash(x)).collect();
            let rule_len = rule.len();
            for _ in 0..g.max_rule_size - rule_len {
                rule.push(F::ZERO);
            }
            rule.push(to_F(is_atomic as usize));
            rule.push(to_F(is_np as usize));
            out.push(rule.clone());
        }
    }
    out.sort();

    let mut any = vec![F::ZERO, coral_hash("terminal_ANY")];
    for _ in 0..g.max_rule_size {
        any.push(F::ZERO);
    }
    out.push(any);
    out
}

pub fn make_np_vector<F: ArkPrimeField>(g: &GrammarGraph) -> Vec<Vec<F>> {
    let np_filler: F = to_F(std::u32::MAX as usize + 1);
    let np_rule_len = max(g.max_np_rule_size + 1, 1);
    let mut out: Vec<Vec<F>> = Vec::new();
    for (rule_name, rule) in g.np.iter() {
        let mut np_rule: Vec<F> = vec![coral_hash(rule_name)];
        for val in rule.0.iter() {
            np_rule.push(coral_hash(val));
        }
        for _ in 0..np_rule_len - np_rule.len() {
            np_rule.push(np_filler);
        }
        out.push(np_rule.clone());
    }
    out.sort();

    out
}

pub fn make_whitespace_vec<F: ArkPrimeField>(g: &GrammarGraph) -> Vec<F> {
    let ws_filler: F = to_F(std::u32::MAX as usize + 1);
    let mut out: Vec<F> = vec![ws_filler];
    let symbols = g.rules.get("WHITESPACE");
    match symbols {
        None => {
            return out;
        }
        Some(rules) => {
            for rule in rules.iter() {
                out.push(coral_hash(&rule[0]));
            }
        }
    }
    out
}

pub fn vec_search<F: ArkPrimeField>(find: &Vec<F>, among: &Vec<Vec<F>>) -> usize {
    let loc = among.iter().position(|x| x == find);
    assert!(loc.is_some());

    loc.unwrap()
}

#[allow(non_snake_case)]
pub fn to_F<F: ArkPrimeField>(num: usize) -> F {
    F::from(num as u64)
}

pub fn make_rule<F: ArkPrimeField>(g: &GrammarGraph, node: &NodeElem<F>, whitespace: F) -> Vec<F> {
    let mut rule: Vec<F> = g
        .get_all_children(node.id)
        .iter()
        .map(|x| coral_hash(&x.rule_name))
        .filter(|x| *x != whitespace)
        .collect();

    rule.reverse();
    rule.push(node.symbol);
    rule
}

pub fn converted_np_map<F: ArkPrimeField>(
    g: &GrammarGraph,
    vals_size: usize,
) -> HashMap<F, Vec<F>> {
    let np_filler: F = to_F(std::u32::MAX as usize + 1);
    let mut out: HashMap<F, Vec<F>> = new_hash_map();
    for (rule, poly) in g.np.iter() {
        let mut vec: Vec<F> = vec![coral_hash(rule)];
        for val in &poly.0 {
            vec.push(coral_hash(val));
        }
        for _ in 0..vals_size - vec.len() {
            vec.push(np_filler);
        }
        out.insert(coral_hash(rule), vec);
    }
    out
}

#[derive(Debug, Clone, CanonicalDeserialize, CanonicalSerialize)]
pub struct CoralStepCircuit<F: ArkPrimeField> {
    //empty bool
    pub empty: bool,
    //Blind for hashchain
    pub blind: F,
    //Public Values
    pub ws_pts: Vec<F>,
    pub rule_size: usize,
    pub n_rules: usize,
    pub whitespace_rule_val: F,
    pub any_rule_val: F,
    pub epsilon_val: F,
    pub tree_ram_offset: usize,
    pub tree_ram_tag: usize,
    pub rule_ram_tag: usize,
    pub rule_ram_offset: usize,
    pub np_ram_tag: usize,
    pub np_ram_offset: usize,
    pub rule_stack_tag: usize,
    pub trans_stack_tag: usize,
    pub mem_ops: usize,
    pub stack_ops: usize,
    pub batch_size: usize,
    pub atom: HashSet<F>,
    pub np: HashSet<F>,
    pub n_np: usize,
    pub np_size: usize,
    pub negative_one: F,
    pub shift_powers: [F; 7],
    //Private Tree Information
    pub tree_size: F,
    pub tree_size_usize: usize,
    pub tree_null_val: usize,
    //Memory Obj
    pub mem: Option<RunningMem<F>>,
    pub key_length: usize,
    //Rule lookup witnesses
    pub switch_wits: Vec<F>,
    pub rule_memory_vec_wits: Vec<Vec<F>>,
    pub rule_memory_addr_wits: Vec<usize>,
    //Negative preidcate witnesses
    pub np_memory_vec_wits: Vec<Vec<F>>,
    pub np_memory_addr_wits: Vec<usize>,
    //IVC stuff
    pub node_wits: Vec<NodeElem<F>>,
    pub parent_node_wits: Vec<NodeElem<F>>,
    pub round_num: usize,
}

impl<F: ArkPrimeField> CoralStepCircuit<F> {
    pub fn new(g: &GrammarGraph, batch_size: usize, doc_blind: F) -> Self {
        let epsilon_val_hash: F = coral_hash("");

        let tree_size = g.lcrs_tree.node_count();

        let tree_ram_offset = 1;
        let rule_ram_offset = tree_ram_offset + (tree_size + 1);
        let np_ram_offset = rule_ram_offset + g.rule_count + 1;
        let mut shift_powers = [F::ONE; 7];
        let mut power = F::from(1u64 << 32);
        for p in &mut shift_powers[1..] {
            *p = power;
            power.square_in_place();
        }

        let np_size = max(g.max_np_rule_size + 1, 1);

        
        Self {
            empty: false,
            //Blind for KZG
            blind: doc_blind,
            //Public Values
            negative_one: F::from(-1),
            ws_pts: make_whitespace_vec(g),
            rule_size: g.max_rule_size,
            n_rules: g.rule_count + 1,
            whitespace_rule_val: coral_hash("WHITESPACE"),
            any_rule_val: coral_hash("terminal_ANY"),
            epsilon_val: epsilon_val_hash,
            batch_size,
            atom: g.atom.iter().map(|x| coral_hash(x)).collect(),
            np: g.np_rule_names.iter().map(|x| coral_hash(x)).collect(),
            n_np: g.np.len(),
            np_size,
            shift_powers,
            //Private Tree Info
            tree_null_val: tree_size,
            tree_size_usize: tree_size,
            tree_size: to_F(tree_size),
            //Memory
            mem: None,
            rule_stack_tag: 0,
            trans_stack_tag: 1,
            tree_ram_offset,
            tree_ram_tag: 2,
            rule_ram_tag: 0,
            rule_ram_offset,
            np_ram_tag: 1,
            np_ram_offset,
            mem_ops: 3 * batch_size,
            stack_ops: (g.max_rule_size + 3) * batch_size,
            key_length: 0,
            //Rule lookup witnesses
            switch_wits: Vec::new(),
            rule_memory_vec_wits: Vec::new(),
            rule_memory_addr_wits: Vec::new(),
            //Negative Predicate Witnesses
            np_memory_addr_wits: Vec::new(),
            np_memory_vec_wits: Vec::new(),
            //IVC Stuff
            node_wits: Vec::new(),
            parent_node_wits: Vec::new(),
            round_num: 0,
        }
    }

    #[allow(non_snake_case)]
    pub fn to_F(num: usize) -> F {
        F::from(num as u64)
    }

    pub fn make_emtpy(self) -> Self {
        let mut empty = self.clone();

        empty.empty = true;
        empty.blind = F::ZERO;

        let filler_vec_rule: Vec<F> = (0..self.rule_size).map(|_| F::ZERO).collect();
        let filler_vec_np: Vec<F> = (0..self.np_size).map(|_| F::ZERO).collect();

        empty.mem = Some(self.mem.unwrap().get_dummy());

        empty.rule_memory_addr_wits = Vec::new();
        empty.rule_memory_vec_wits = Vec::new();
        empty.np_memory_addr_wits = Vec::new();
        empty.np_memory_vec_wits = Vec::new();
        empty.node_wits = Vec::new();
        empty.parent_node_wits = Vec::new();
        empty.switch_wits = Vec::new();

        let dead_node = NodeElem {
            id: empty.tree_null_val,
            terminal: true,
            parent: empty.tree_null_val,
            symbol: F::ZERO,
            child: empty.tree_null_val,
            sib: empty.tree_null_val,
        };

        for _ in 0..self.batch_size {
            empty.node_wits.push(dead_node.clone());
            empty.parent_node_wits.push(dead_node.clone());

            empty.np_memory_vec_wits.push(filler_vec_np.clone());
            empty.np_memory_addr_wits.push(self.np_ram_offset);

            empty.rule_memory_vec_wits.push(filler_vec_rule.clone());
            empty.rule_memory_addr_wits.push(self.rule_ram_offset);
            empty.switch_wits.push(F::zero());
        }

        empty
    }

    pub fn init_set(&mut self, g: &GrammarGraph) -> (MemBuilder<F>, Vec<Vec<F>>, Vec<Vec<F>>) {
        let mut mem_builder = MemBuilder::new(
            vec![
                MemType::PrivROM(self.tree_ram_tag, 5),
                MemType::PubROM(self.rule_ram_tag, self.rule_size + 2),
                MemType::PubROM(self.np_ram_tag, self.np_size),
            ],
            vec![2, 2],
        );

        let np_vector = make_np_vector(g);

        let rule_vector = make_rule_vector(g);

        if np_vector.is_empty() {
            mem_builder.init(
                self.np_ram_offset,
                (0..self.np_size).map(|_| F::ZERO).collect(),
                self.np_ram_tag,
            );
        } else {
            for i in 0..np_vector.len() {
                mem_builder.init(
                    i + self.np_ram_offset,
                    np_vector[i].clone(),
                    self.np_ram_tag,
                );
            }
        }

        for i in 0..rule_vector.len() {
            mem_builder.init(
                i + self.rule_ram_offset,
                rule_vector[i].clone(),
                self.rule_ram_tag,
            );
        }

        (mem_builder, np_vector, rule_vector)
    }

    pub fn solve(
        &mut self,
        g: &GrammarGraph,
    ) -> Result<(Vec<Vec<N1>>, Vec<Vec<N1>>, CoralStepCircuit<F>), SynthesisError> {
        #[cfg(feature = "metrics")]
        {
            log::tic(Component::Solver, "e2e_solving");
            log::tic(Component::Solver, "wit_solving");
        }

        let (mut mem_builder, np_vec, rule_vec) = self.init_set(g);

        let converted_np_map = converted_np_map(g, self.np_size);

        let ws_f = coral_hash("WHITESPACE");

        let mut any = vec![F::ZERO, self.any_rule_val];
        for _ in 0..self.rule_size {
            any.push(F::ZERO);
        }

        let any_addr = vec_search(&any, &rule_vec) + self.rule_ram_offset;

        let np_f_set: HashSet<F> = g.np_rule_names.iter().map(|x| coral_hash(x)).collect();

        let atom_f_set: HashSet<F> = g.atom.iter().map(|x| coral_hash(x)).collect();

        let mut node = make_node_elem(0, g);

        let mut parent = NodeElem {
            id: self.tree_null_val,
            terminal: false,
            parent: self.tree_null_val,
            symbol: F::ZERO,
            sib: self.tree_null_val,
            child: self.tree_null_val,
        };
        parent.clone().mem_init(self, &mut mem_builder);

        let mut trans_stack: Vec<(usize, usize)> = Vec::new();
        let mut rule_stack: Vec<F> = Vec::new();

        let mut rule_stack_max_depth = 0;
        let mut trans_stack_max_depth = 0;

        let filler_vec_rule: Vec<F> = (0..self.rule_size).map(|_| F::ZERO).collect();
        let filler_vec_np: Vec<F> = (0..self.np_size).map(|_| F::ZERO).collect();
        let filler_vec_stack: Vec<F> = (0..2).map(|_| F::ZERO).collect();

        let mut np_rule: F = F::ZERO;

        for w in 0..g.lcrs_tree.node_count() {
            self.node_wits.push(node.clone());
            node.clone().mem_init(self, &mut mem_builder);
            mem_builder.read(node.id + self.tree_ram_offset, self.tree_ram_tag);

            self.parent_node_wits.push(parent.clone());

            if np_rule == F::ZERO && np_f_set.contains(&node.symbol) {
                np_rule = node.symbol;
            };

            if node.symbol != ws_f {
                let pop_cond = node.id != 0;
                let top = mem_builder.cond_pop(pop_cond, self.rule_stack_tag);
                if pop_cond {
                    rule_stack.pop();
                    if rule_stack.len() > rule_stack_max_depth {
                        rule_stack_max_depth = rule_stack.len();
                    }
                    assert_eq!(top[0], node.symbol);
                    if !node.terminal {
                        assert_ne!(top[0], F::ZERO);
                    }
                }
            } else {
                mem_builder.cond_pop(false, self.rule_stack_tag);
            }

            if node.terminal {
                let is_np_rule = np_rule != F::ZERO;
                let mut np_rule_addr = 0;
                if np_rule == F::ZERO {
                    self.np_memory_vec_wits.push(filler_vec_np.clone());
                    self.np_memory_addr_wits.push(np_rule_addr);
                } else {
                    let np_rule_vec = converted_np_map.get(&np_rule).unwrap();
                    self.np_memory_vec_wits.push(np_rule_vec.clone());
                    let addr = vec_search(np_rule_vec, &np_vec) + self.np_ram_offset;
                    self.np_memory_addr_wits.push(addr);
                    np_rule_addr = addr;
                }
                mem_builder.cond_read(is_np_rule, np_rule_addr, self.np_ram_tag);

                let should_trans_pop =
                    (w < g.lcrs_tree.node_count() - 1) & (node.sib == self.tree_null_val);
                mem_builder.cond_pop(should_trans_pop, self.trans_stack_tag);

                if should_trans_pop {
                    let next = trans_stack.pop();
                    if trans_stack.len() > trans_stack_max_depth {
                        trans_stack_max_depth = trans_stack.len();
                    }

                    match next {
                        None => {}
                        Some(t) => {
                            node = make_node_elem(t.0, g);
                            parent = make_node_elem(t.1, g);
                        }
                    }
                } else if node.sib != self.tree_null_val {
                    node = make_node_elem(node.sib, g);
                }
                //not terminal trans push
                mem_builder.cond_push(false, self.trans_stack_tag, filler_vec_stack.clone());

                //non terminal rule stack push
                for _ in 0..self.rule_size {
                    mem_builder.cond_push(false, self.rule_stack_tag, filler_vec_stack.clone());
                }

                //Non terminal rule read
                self.rule_memory_vec_wits.push(filler_vec_rule.clone());
                self.rule_memory_addr_wits.push(0);
                mem_builder.cond_read(false, 0, self.rule_ram_tag);

                self.switch_wits.push(F::zero());
                np_rule = F::ZERO;
            } else {
                //terminal NP memory read
                self.np_memory_vec_wits.push(filler_vec_np.clone());
                self.np_memory_addr_wits.push(0);
                mem_builder.cond_read(false, 0, self.np_ram_tag);

                //terminal trans stack pop
                mem_builder.cond_pop(false, self.trans_stack_tag);

                let mut children_rule: Vec<F> = make_rule(g, &node, self.whitespace_rule_val);

                if node.symbol == self.any_rule_val {
                    assert!(children_rule.len() == 2)
                };

                let trans_stack_push_cond = node.sib != self.tree_null_val;
                let mut trans_stack_push_val = filler_vec_stack.clone();
                if trans_stack_push_cond {
                    trans_stack.push((node.sib, node.parent));
                    trans_stack_push_val[0] = to_F::<F>(node.sib);
                    trans_stack_push_val[1] = to_F::<F>(node.parent);
                    if trans_stack.len() > trans_stack_max_depth {
                        trans_stack_max_depth = trans_stack.len();
                    }
                }
                mem_builder.cond_push(
                    trans_stack_push_cond,
                    self.trans_stack_tag,
                    trans_stack_push_val.clone(),
                );

                let cur_rule_len = children_rule.len();
                self.switch_wits.push(to_F(cur_rule_len - 1));

                for _ in 0..g.max_rule_size - cur_rule_len {
                    children_rule.push(F::ZERO);
                }
                self.rule_memory_vec_wits.push(children_rule.clone());

                let atomic = if atom_f_set.contains(&node.symbol) {
                    F::ONE
                } else {
                    F::ZERO
                };
                children_rule.push(atomic);

                let np = if np_rule == node.symbol || np_f_set.contains(&node.symbol) {
                    F::ONE
                } else {
                    F::ZERO
                };
                children_rule.push(np);

                let addr = if node.symbol == self.any_rule_val {
                    any_addr
                } else {
                    vec_search(&children_rule, &rule_vec) + self.rule_ram_offset
                };

                self.rule_memory_addr_wits.push(addr);

                for child_idx in 0..cur_rule_len - 1 {
                    let mut to_push = vec![children_rule[child_idx], F::ZERO];
                    if child_idx == 0 {
                        to_push[1] = F::ONE;
                    }
                    mem_builder.push(self.rule_stack_tag, to_push.clone());

                    rule_stack.push(children_rule[child_idx]);
                    if rule_stack.len() > rule_stack_max_depth {
                        rule_stack_max_depth = rule_stack.len();
                    }
                }
                for _ in 0..(self.rule_size - cur_rule_len + 1) {
                    mem_builder.cond_push(false, self.rule_stack_tag, filler_vec_stack.clone());
                }
                mem_builder.read(addr, self.rule_ram_tag);

                parent = node.clone();
                node = make_node_elem(node.child, g);
            }
        }

        let dead_node = NodeElem {
            id: self.tree_null_val,
            terminal: true,
            parent: self.tree_null_val,
            symbol: F::ZERO,
            child: self.tree_null_val,
            sib: self.tree_null_val,
        };

        let padding_needed = if g.lcrs_tree.node_count().is_multiple_of(self.batch_size) {
            0
        } else {
            self.batch_size - (g.lcrs_tree.node_count() % self.batch_size)
        };

        for _ in 0..padding_needed {
            self.node_wits.push(dead_node.clone());
            self.parent_node_wits.push(dead_node.clone());

            self.np_memory_vec_wits.push(filler_vec_np.clone());
            self.np_memory_addr_wits.push(0);

            self.rule_memory_addr_wits.push(0);
            self.rule_memory_vec_wits.push(filler_vec_rule.clone());
            self.switch_wits.push(F::zero());

            //Is node
            mem_builder.cond_read(
                false,
                self.tree_null_val + self.tree_ram_offset,
                self.tree_ram_tag,
            );

            mem_builder.cond_pop(false, self.rule_stack_tag);

            //Is terminal
            mem_builder.cond_read(false, 0, self.np_ram_tag);
            mem_builder.cond_pop(false, self.trans_stack_tag);

            //Is not terminal
            mem_builder.cond_push(false, self.trans_stack_tag, filler_vec_stack.clone());

            for _ in 0..self.rule_size {
                mem_builder.cond_push(false, self.rule_stack_tag, filler_vec_stack.clone());
            }

            mem_builder.cond_read(false, 0, self.rule_ram_tag);
        }

        #[cfg(feature = "metrics")]
        log::stop(Component::Solver, "wit_solving");

        #[cfg(feature = "metrics")]
        log::tic(Component::Solver, "ic");
        let (blinds, ram_hints, ram_batch_size, rm) = mem_builder.new_running_mem(
            vec![
                (self.tree_ram_tag, self.batch_size),
                (self.np_ram_tag, self.batch_size),
                (self.rule_ram_tag, self.batch_size),
            ],
            vec![2 * self.batch_size, (self.rule_size + 1) * self.batch_size],
            false,
            "./ppot_0080_23.ptau",
        );
        self.mem = Some(rm);
        self.key_length = ram_batch_size;

        #[cfg(feature = "metrics")]
        {
            log::stop(Component::Solver, "ic");
            log::tic(Component::Solver, "e2e_solving");
        }

        let empty = self.clone().make_emtpy();

        Ok((blinds, ram_hints, empty))
    }
}
