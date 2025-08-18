use crate::solver::{CoralStepCircuit, CoralWires, to_F};
use crate::util::ArkPrimeField;
use ark_r1cs_std::{
    GR1CSVar,
    alloc::AllocVar,
    boolean::Boolean,
    eq::EqGadget,
    fields::{FieldVar, fp::FpVar},
};
use ark_relations::gr1cs::{ConstraintSystemRef, SynthesisError};
use core::ops::Not;
use segmented_circuit_memory::bellpepper::AllocIoVar;
use segmented_circuit_memory::memory::nebula::RunningMemWires;

#[tracing::instrument(target = "gr1cs")]
pub fn tree_read<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    addr: &FpVar<F>,
    cond: &Boolean<F>,
    vals_fpvars: &[FpVar<F>],
    wires: &mut CoralWires<F>,
    memory: &mut RunningMemWires<F>,
    cs: ConstraintSystemRef<F>,
) -> Result<(), SynthesisError> {
    addr.conditional_enforce_not_equal(&FpVar::Constant(F::ZERO), cond)?;

    let res = csc
        .mem
        .as_mut()
        .unwrap()
        .conditional_read(cond, addr, csc.tree_ram_tag, memory)?;

    chunk_cee(cond, &res.vals, vals_fpvars, csc, cs)?;

    Ok(())
}

#[tracing::instrument(target = "gr1cs")]
pub fn np_read<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    addr: &FpVar<F>,
    cond: &Boolean<F>,
    vals_fpvars: &Vec<FpVar<F>>,
    wires: &mut CoralWires<F>,
    memory: &mut RunningMemWires<F>,
    cs: ConstraintSystemRef<F>,
) -> Result<(), SynthesisError> {
    addr.conditional_enforce_not_equal(&FpVar::zero(), cond)?;

    let res = csc
        .mem
        .as_mut()
        .unwrap()
        .conditional_read(cond, addr, csc.np_ram_tag, memory)?;

    chunk_cee(cond, &res.vals, vals_fpvars, csc, cs)?;

    Ok(())
}

#[tracing::instrument(target = "gr1cs")]
pub fn rule_read<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    addr: &FpVar<F>,
    cond: &Boolean<F>,
    vals_fpvars: &Vec<FpVar<F>>,
    wires: &mut CoralWires<F>,
    memory: &mut RunningMemWires<F>,
    cs: ConstraintSystemRef<F>,
) -> Result<(), SynthesisError> {
    addr.conditional_enforce_not_equal(&FpVar::Constant(F::ZERO), cond)?;

    let res = csc
        .mem
        .as_mut()
        .unwrap()
        .conditional_read(cond, addr, csc.rule_ram_tag, memory)?;

    chunk_cee(cond, &res.vals, vals_fpvars, csc, cs)?;

    Ok(())
}

#[tracing::instrument(target = "gr1cs")]
pub fn chunk_cee<F: ArkPrimeField>(
    cond: &Boolean<F>,
    l_vals: &[FpVar<F>],
    r_vals: &[FpVar<F>],
    csc: &mut CoralStepCircuit<F>,
    cs: ConstraintSystemRef<F>,
) -> Result<(), SynthesisError> {
    debug_assert_eq!(l_vals.len(), r_vals.len());
    let shift_powers = csc.shift_powers.map(FpVar::constant);

    for (l_chunk, r_chunk) in l_vals.chunks(7).zip(r_vals.chunks(7)) {
        let shift_powers = &shift_powers[..l_chunk.len()];
        let l_pack = FpVar::inner_product(l_chunk, shift_powers)?;
        let r_pack = FpVar::inner_product(r_chunk, shift_powers)?;
        l_pack.conditional_enforce_equal(&r_pack, cond)?;
    }

    Ok(())
}

#[tracing::instrument(target = "gr1cs")]
pub fn chunk_cee_zero<F: ArkPrimeField>(
    cond: &Boolean<F>,
    l_vals: &[FpVar<F>],
    csc: &mut CoralStepCircuit<F>,
    cs: ConstraintSystemRef<F>,
) -> Result<(), SynthesisError> {
    let shift_powers = csc.shift_powers.map(FpVar::constant);
    for l_chunk in l_vals.chunks(7) {
        let shift_powers = &shift_powers[..l_chunk.len()];
        let l_pack = FpVar::inner_product(l_chunk, shift_powers)?;
        l_pack.conditional_enforce_equal(&FpVar::zero(), cond)?;
    }
    Ok(())
}

#[tracing::instrument(target = "gr1cs")]
pub fn assert_filler<F: ArkPrimeField>(
    addr: &FpVar<F>,
    vals: &[FpVar<F>],
    cond: &Boolean<F>,
    csc: &mut CoralStepCircuit<F>,
    cs: ConstraintSystemRef<F>,
) -> Result<(), SynthesisError> {
    chunk_cee_zero(cond, vals, csc, cs)
}

#[tracing::instrument(target = "gr1cs")]
pub fn vanishing_poly<F: ArkPrimeField>(
    vanish_pts: &[FpVar<F>],
    x: &FpVar<F>,
) -> Result<FpVar<F>, SynthesisError> {
    let mut vanish = x - &vanish_pts[0];
    for pt in &vanish_pts[1..] {
        vanish *= x - pt;
    }
    Ok(vanish)
}

#[tracing::instrument(target = "gr1cs")]
pub fn rule_pop_wrapper<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    condition: &Boolean<F>,
    wires: &mut CoralWires<F>,
    memory: &mut RunningMemWires<F>,
) -> Result<(FpVar<F>, FpVar<F>), SynthesisError> {
    let popped =
        csc.mem
            .as_mut()
            .unwrap()
            .conditional_pop(condition, csc.rule_stack_tag, memory)?;

    Ok((popped.vals[0].clone(), popped.vals[1].clone()))
}

#[tracing::instrument(target = "gr1cs")]
pub fn rule_push_wrapper<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    condition: &Boolean<F>,
    vals: Vec<FpVar<F>>,
    wires: &mut CoralWires<F>,
    memory: &mut RunningMemWires<F>,
) -> Result<(), SynthesisError> {
    csc.mem
        .as_mut()
        .unwrap()
        .conditional_push(condition, csc.rule_stack_tag, vals, memory)?;

    Ok(())
}

#[tracing::instrument(target = "gr1cs")]
pub fn trans_pop_wrapper<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    condition: &Boolean<F>,
    wires: &mut CoralWires<F>,
    memory: &mut RunningMemWires<F>,
) -> Result<Vec<FpVar<F>>, SynthesisError> {
    wires.prev_step_t_ops = condition.select(&Boolean::TRUE, &wires.prev_step_t_ops)?;
    let popped =
        csc.mem
            .as_mut()
            .unwrap()
            .conditional_pop(condition, csc.trans_stack_tag, memory)?;

    Ok(popped.vals[..2].to_vec())
}

#[tracing::instrument(target = "gr1cs")]
pub fn trans_push_wrapper<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    condition: &Boolean<F>,
    vals: Vec<FpVar<F>>,
    wires: &mut CoralWires<F>,
    memory: &mut RunningMemWires<F>,
) -> Result<(), SynthesisError> {
    wires.prev_step_t_ops = condition.select(&Boolean::TRUE, &wires.prev_step_t_ops)?;

    csc.mem
        .as_mut()
        .unwrap()
        .conditional_push(condition, csc.trans_stack_tag, vals, memory)?;

    Ok(())
}

#[tracing::instrument(target = "gr1cs")]
pub fn extend_commit<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    shift: &FpVar<F>,
    terminal: &Boolean<F>,
    wires: &CoralWires<F>,
    memory: &mut RunningMemWires<F>,
    val: &FpVar<F>,
    cs: ConstraintSystemRef<F>,
) -> Result<FpVar<F>, SynthesisError> {
    let blind = FpVar::new_witness(cs.clone(), || Ok(csc.blind))?;
    let running_eval = &wires.running_eval;
    let is_one = running_eval.is_eq(&FpVar::one())?;
    let chal = memory.perm_chal.clone();

    let cond_running_eval = is_one.select(&(&chal[0] - &blind), running_eval)?;

    let epsilon_val = FpVar::constant(csc.epsilon_val);

    let is_epsilon = val.is_eq(&epsilon_val)?;

    let same_eval = !terminal | &is_epsilon;

    let root = (val * shift) + &wires.doc_ctr; //To account for SOI 
    let next_root = &chal[0] - root;
    let eval = &wires.running_eval * next_root;

    same_eval.select(&cond_running_eval, &eval)
}

#[tracing::instrument(target = "gr1cs")]
pub fn wires_update<F: ArkPrimeField>(
    new_wires: &mut CoralWires<F>,
    terminal_wires: &mut CoralWires<F>,
    non_terminal_wires: &mut CoralWires<F>,
    pre_op_sp: &FpVar<F>,
    is_terminal: &Boolean<F>,
    memory: &mut RunningMemWires<F>,
    csc: &mut CoralStepCircuit<F>,
) -> Result<(), SynthesisError> {
    new_wires.cur_node_id =
        is_terminal.select(&terminal_wires.cur_node_id, &non_terminal_wires.cur_node_id)?;

    new_wires.doc_ctr = is_terminal.select(&terminal_wires.doc_ctr, &non_terminal_wires.doc_ctr)?;

    new_wires.running_eval = is_terminal.select(
        &terminal_wires.running_eval,
        &non_terminal_wires.running_eval,
    )?;

    new_wires.parent_id =
        is_terminal.select(&terminal_wires.parent_id, &non_terminal_wires.parent_id)?;

    new_wires.np_rule = is_terminal.select(&terminal_wires.np_rule, &non_terminal_wires.np_rule)?;

    new_wires.np_parent_id = is_terminal.select(
        &terminal_wires.np_parent_id,
        &non_terminal_wires.np_parent_id,
    )?;

    new_wires.np_sp = is_terminal.select(&terminal_wires.np_sp, &non_terminal_wires.np_sp)?;

    new_wires.atom_flag =
        is_terminal.select(&terminal_wires.atom_flag, &non_terminal_wires.atom_flag)?;

    new_wires.atom_parent_id = is_terminal.select(
        &terminal_wires.atom_parent_id,
        &non_terminal_wires.atom_parent_id,
    )?;

    new_wires.atom_sp = is_terminal.select(&terminal_wires.atom_sp, &non_terminal_wires.atom_sp)?;

    new_wires.prev_step_t_ops =
        &terminal_wires.prev_step_t_ops | &non_terminal_wires.prev_step_t_ops;

    new_wires.count = is_terminal.select(&terminal_wires.count, &non_terminal_wires.count)?;

    Ok(())
}

#[tracing::instrument(target = "gr1cs")]
pub fn is_terminal<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    tree_null_val: &FpVar<F>,
    shift: &FpVar<F>,
    terminal: &Boolean<F>,
    cur_symbol: &FpVar<F>,
    child: &FpVar<F>,
    sib: &FpVar<F>,
    wires: &CoralWires<F>,
    memory: &mut RunningMemWires<F>,
    cs: ConstraintSystemRef<F>,
) -> Result<CoralWires<F>, SynthesisError> {
    let is_np: Boolean<F> = wires.np_rule.is_neq(&FpVar::zero())?;

    let mut new_wires = wires.clone();

    // Check is leaf
    let child_check = child.is_eq(tree_null_val)?;
    let sib_not_null = sib.is_neq(tree_null_val)?;

    child_check.conditional_enforce_equal(&Boolean::TRUE, terminal)?;

    //If np is on check polynomial
    let np_lookup_wits = csc.np_memory_vec_wits[csc.round_num].clone();
    let np_lookup_addr = FpVar::new_witness(cs.clone(), || {
        Ok(to_F::<F>(csc.np_memory_addr_wits[csc.round_num]))
    })?;
    let polys: Vec<FpVar<F>> = np_lookup_wits
        .iter()
        .map(|x| FpVar::new_witness(cs.clone(), || Ok(x)).unwrap())
        .collect();

    assert_filler(&np_lookup_addr, &polys, &!&is_np, csc, cs.clone())?;

    np_read(
        csc,
        &np_lookup_addr,
        &(terminal & &is_np),
        &polys,
        &mut new_wires,
        memory,
        cs.clone(),
    )?;

    let poly_eval = vanishing_poly(&polys, cur_symbol)?;

    poly_eval.conditional_enforce_not_equal(&FpVar::zero(), &(terminal & &is_np))?;

    let last = FpVar::<F>::new_witness(cs.clone(), || Ok(csc.tree_size))?;

    let is_last = wires.count.is_eq(&(last - F::ONE))?;

    let trans_stack_pop_values = trans_pop_wrapper(
        csc,
        &(terminal & is_last.not() & &sib_not_null.clone().not()),
        &mut new_wires,
        memory,
    )?;

    let running_eval = extend_commit(
        csc,
        shift,
        terminal,
        &new_wires,
        memory,
        cur_symbol,
        cs.clone(),
    )?;

    new_wires.running_eval = running_eval;
    new_wires.parent_id = sib_not_null.select(&new_wires.parent_id, &trans_stack_pop_values[1])?;
    new_wires.cur_node_id = sib_not_null.select(sib, &trans_stack_pop_values[0])?;
    let is_epsilon = cur_symbol.is_eq(&FpVar::constant(csc.epsilon_val))?;
    new_wires.doc_ctr = is_epsilon.select(&new_wires.doc_ctr, &(&new_wires.doc_ctr + F::ONE))?;

    new_wires.np_rule = FpVar::zero();

    Ok(new_wires)
}

#[tracing::instrument(target = "gr1cs")]
pub fn not_terminal<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    tree_null_val: &FpVar<F>,
    should_run: &Boolean<F>,
    cur_id: &FpVar<F>,
    cur_symbol: &FpVar<F>,
    child: &FpVar<F>,
    sib: &FpVar<F>,
    parent: &FpVar<F>,
    is_root: &Boolean<F>,
    ws_val: &FpVar<F>,
    round_num: usize,
    cur_is_atomic: &Boolean<F>,
    cur_is_np: &Boolean<F>,
    wires: &CoralWires<F>,
    memory: &mut RunningMemWires<F>,
    cs: ConstraintSystemRef<F>,
) -> Result<CoralWires<F>, SynthesisError> {
    let mut new_wires = wires.clone();

    //Is sib Null?
    let sib_check = sib.is_neq(tree_null_val)?;

    //Should have a child
    let child_check = child.is_neq(tree_null_val)?;
    child_check.conditional_enforce_equal(&Boolean::TRUE, should_run)?;

    let ts_push = &sib_check & should_run;

    let mut trans_push_vec = vec![];

    let trans_push_sib = ts_push.select(sib, &FpVar::zero())?;
    trans_push_vec.push(trans_push_sib);

    let trans_push_parent = ts_push.select(parent, &FpVar::zero())?;
    trans_push_vec.push(trans_push_parent);

    //push to trans stack if sib is not null
    trans_push_wrapper(csc, &ts_push, trans_push_vec, &mut new_wires, memory)?;

    //assert rule in table
    let rule_lookup_addr_usize: usize = csc.rule_memory_addr_wits[round_num];
    let rule_lookup_addr =
        FpVar::new_witness(cs.clone(), || Ok(to_F::<F>(rule_lookup_addr_usize)))?;
    let mut rule_lookup_vec = csc.rule_memory_vec_wits[round_num]
        .iter()
        .map(|val| FpVar::new_witness(cs.clone(), || Ok(val)))
        .collect::<Result<Vec<_>, _>>()?;

    let is_any = cur_symbol.is_eq(&FpVar::constant(csc.any_rule_val))?;
    let is_not_root_check = !is_root & is_any.clone().not() & should_run;

    //atomic
    rule_lookup_vec.push(FpVar::from(cur_is_atomic & &is_not_root_check));

    //np
    rule_lookup_vec.push(FpVar::from(cur_is_np & &is_not_root_check));

    //Switch point for rule push
    let switch_var = FpVar::new_witness(cs.clone(), || Ok(csc.switch_wits[round_num]))?;

    let mut prev_round_flag: Boolean<F> = Boolean::FALSE;
    let mut switch_flag;

    for i in 0..csc.rule_size {
        //If less than switch - so not padding
        let switch_var_eq = switch_var.is_eq(&FpVar::constant(F::from(i as u64)))?;
        switch_flag = &switch_var_eq | &prev_round_flag;

        //This should always be false - you shouldnt be passing ws here except for the ws rule itself
        let is_ws = rule_lookup_vec[i].is_eq(ws_val)?;

        let is_not_ws_rule_itself = should_run & !switch_var_eq.clone();

        is_ws.conditional_enforce_equal(&Boolean::FALSE, &is_not_ws_rule_itself)?;

        let push_rule = should_run & !switch_flag.clone();

        let mut push_rule_vec = vec![rule_lookup_vec[i].clone()];

        push_rule_vec[0] = push_rule.select(&push_rule_vec[0], &FpVar::zero())?;

        let is_first_step = FpVar::constant(F::from(i as u64)).is_eq(&FpVar::zero())?;

        push_rule_vec.push((is_first_step & should_run).select(&FpVar::one(), &FpVar::zero())?);

        rule_push_wrapper(csc, &push_rule, push_rule_vec, &mut new_wires, memory)?;

        // If greater than switch, just padding out at this point
        let pad_condition = &switch_flag & is_not_ws_rule_itself;

        rule_lookup_vec[i].conditional_enforce_equal(&FpVar::zero(), &pad_condition)?;

        rule_lookup_vec[i].conditional_enforce_equal(cur_symbol, &(&switch_var_eq & should_run))?;

        prev_round_flag = switch_flag;
    }

    new_wires.parent_id = cur_id.clone();
    new_wires.cur_node_id = child.clone();

    assert_filler(
        &rule_lookup_addr,
        &rule_lookup_vec,
        &!should_run,
        csc,
        cs.clone(),
    )?;

    rule_lookup_vec[0] = is_any.select(&FpVar::zero(), &rule_lookup_vec[0])?;

    rule_read(
        csc,
        &rule_lookup_addr,
        should_run,
        &rule_lookup_vec,
        &mut new_wires,
        memory,
        cs.clone(),
    )?;

    Ok(new_wires)
}

#[tracing::instrument(target = "gr1cs")]
pub fn node_circuit<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    mut wires: CoralWires<F>,
    memory: &mut RunningMemWires<F>,
    should_run: &Boolean<F>,
    tree_null_val: &FpVar<F>,
    shift: &FpVar<F>,
    offsets: &[FpVar<F>],
    cs: ConstraintSystemRef<F>,
) -> Result<CoralWires<F>, SynthesisError> {
    let round_num = csc.round_num;

    // //Current tree location information
    let node_elem = csc.node_wits[round_num].clone();

    let cur_node = wires.cur_node_id.clone();

    let should_run_inv = should_run.not();

    let cur_is_null = cur_node.is_eq(tree_null_val)?;

    cur_is_null.enforce_equal(&should_run_inv)?;

    cur_node.conditional_enforce_equal(tree_null_val, &should_run_inv)?;

    cur_node.conditional_enforce_equal(&wires.count, should_run)?;

    let parent = wires.parent_id.clone();
    let child = FpVar::new_witness(cs.clone(), || Ok(F::from(node_elem.child as u64)))?;
    let sib = FpVar::new_witness(cs.clone(), || Ok(F::from(node_elem.sib as u64)))?;
    let terminal = Boolean::new_witness(cs.clone(), || Ok(node_elem.terminal))?;

    let symbol = FpVar::new_witness(cs.clone(), || Ok(node_elem.symbol))?;

    let node_elem_vec = [
        FpVar::from(terminal.clone()),
        symbol.clone(),
        sib.clone(),
        child.clone(),
        parent.clone(),
    ];
    let node_elem_addr = &(&cur_node + FpVar::constant(to_F(csc.tree_ram_offset)));

    //Update atomic flag and sp
    let atom_sp_eq_cur_sp = wires
        .atom_sp
        .is_eq(&memory.stack_ptrs[csc.trans_stack_tag])?
        & &wires.prev_step_t_ops;

    //If we're out of the atomic subtree turn off
    wires.atom_flag = atom_sp_eq_cur_sp.select(&Boolean::FALSE, &wires.atom_flag)?;
    wires.atom_sp = atom_sp_eq_cur_sp.select(&FpVar::zero(), &wires.atom_sp)?;

    //Update atomic flag and sp
    let cur_is_atom = Boolean::new_witness(cs.clone(), || Ok(csc.atom.contains(&symbol.value()?)))?;

    let af_on = wires.atom_flag.clone();

    let rule_is_atom_af_off =
        cur_is_atom.select(&memory.stack_ptrs[csc.trans_stack_tag], &wires.atom_sp)?;

    wires.atom_sp = wires
        .atom_flag
        .select(&wires.atom_sp, &rule_is_atom_af_off)?;
    wires.atom_flag = (af_on.clone()) | (!af_on & cur_is_atom.clone());

    // //Update np rule
    //If we're out of the np subtree turn off
    let np_sp_eq_cur_sp =
        wires.np_sp.is_eq(&memory.stack_ptrs[csc.trans_stack_tag])? & &wires.prev_step_t_ops;

    wires.np_rule = np_sp_eq_cur_sp.select(&FpVar::zero(), &wires.np_rule)?;

    wires.np_sp = np_sp_eq_cur_sp.select(&FpVar::zero(), &wires.np_sp)?;

    let cur_is_np = Boolean::new_witness(cs.clone(), || Ok(csc.np.contains(&symbol.value()?)))?;

    let np_on = wires.np_rule.is_neq(&FpVar::zero())?;

    let rule_is_np_npf_off = cur_is_np.select(&symbol, &FpVar::zero())?;

    let rule_is_np_npf_off_sp =
        cur_is_np.select(&memory.stack_ptrs[csc.trans_stack_tag], &FpVar::zero())?;

    let new_np_rule_sp = np_on.select(&wires.np_sp, &rule_is_np_npf_off_sp)?;

    let new_np_rule = np_on.select(&wires.np_rule, &rule_is_np_npf_off)?;

    wires.np_rule = new_np_rule;
    wires.np_sp = new_np_rule_sp;

    //Make sure has parent unless we're the root
    let parent_null = parent.is_eq(tree_null_val)?;
    let is_root = cur_node.is_zero()?;
    let is_root_check = wires.count.is_zero()? & should_run;

    let check_pn = is_root.clone() & should_run;

    parent_null.conditional_enforce_equal(&Boolean::TRUE, &check_pn)?;

    is_root.enforce_equal(&is_root_check)?;

    // Assert stack is empty at root
    let rule_stack_is_empty = memory.stack_ptrs[csc.rule_stack_tag].is_eq(&offsets[0])?;

    rule_stack_is_empty.conditional_enforce_equal(&Boolean::TRUE, &(&is_root & should_run))?;

    //Is in the tree
    tree_read(
        csc,
        node_elem_addr,
        should_run,
        &node_elem_vec,
        &mut wires,
        memory,
        cs.clone(),
    )?;

    // //Check if rule is WS
    let ws_val = FpVar::constant(csc.whitespace_rule_val);
    let is_ws = symbol.is_eq(&ws_val)?;

    // pop from rule stack
    let (top_rule_pop_values, top_rule_pop_bool) = rule_pop_wrapper(
        csc,
        &(is_ws.clone().not() & should_run & is_root.clone().not()),
        &mut wires,
        memory,
    )?;

    //rule popped == node
    let pop_equal_node = symbol.is_eq(&top_rule_pop_values)?;

    let sib_is_null_eq = sib.is_eq(tree_null_val)?.value()?;

    let sib_is_null = FpVar::new_witness(cs.clone(), || Ok(F::from(sib_is_null_eq)))?;

    let rule_end = sib_is_null.is_eq(&top_rule_pop_bool)?;

    //Either rule popped == node and not zero (DOBULE CHECK SOME STUFF WITH WS)
    let not_root_stack_cond = pop_equal_node & (is_root.clone().not()) & &rule_end;
    let ws_stack_cond = is_ws & &wires.atom_flag.clone().not();

    let stack_condition = not_root_stack_cond | rule_stack_is_empty | ws_stack_cond;

    stack_condition.conditional_enforce_equal(&Boolean::TRUE, should_run)?;

    let pre_op_sp = memory.stack_ptrs[csc.trans_stack_tag].clone();

    wires.prev_step_t_ops = Boolean::FALSE;

    let terminal_res = is_terminal(
        csc,
        tree_null_val,
        shift,
        &(terminal.clone() & should_run),
        &symbol,
        &child,
        &sib,
        &wires,
        memory,
        cs.clone(),
    );

    debug_assert!(terminal_res.is_ok(), "{:?}", terminal_res);

    let mut terminal_wires = terminal_res.unwrap();

    let non_terminal_res = not_terminal(
        csc,
        tree_null_val,
        &(terminal.clone().not() & should_run),
        &cur_node,
        &symbol,
        &child,
        &sib,
        &parent,
        &is_root,
        &ws_val,
        round_num,
        &cur_is_atom,
        &cur_is_np,
        &wires,
        memory,
        cs.clone(),
    );

    debug_assert!(non_terminal_res.is_ok(), "{:?}", non_terminal_res);

    let mut non_terminal_wires = non_terminal_res.unwrap();

    csc.round_num = if should_run.value()? {
        csc.round_num + 1
    } else {
        csc.round_num
    };

    wires_update(
        &mut wires,
        &mut terminal_wires,
        &mut non_terminal_wires,
        &pre_op_sp,
        &terminal,
        memory,
        csc,
    )?;

    wires.count = &wires.count + FpVar::one();

    Ok(wires)
}

#[tracing::instrument(target = "gr1cs")]
pub fn multi_node_step<F: ArkPrimeField>(
    csc: &mut CoralStepCircuit<F>,
    wires: &mut CoralWires<F>,
    memory: &mut RunningMemWires<F>,
    cs: ConstraintSystemRef<F>,
) -> Result<CoralWires<F>, SynthesisError> {
    let is_empty = Boolean::new_witness(cs.clone(), || Ok(csc.empty))?;
    let mut next_wires = wires.clone();

    let tree_size = FpVar::new_witness(cs.clone(), || Ok(csc.tree_size))?;

    let tree_null_val = FpVar::constant(to_F::<F>(csc.tree_null_val));

    let shift = FpVar::constant(csc.shift_powers[1]); // We want 2^{32}.

    let offsets = [
        FpVar::constant(F::ZERO),
        FpVar::constant(F::ZERO),
        FpVar::constant(F::from(csc.tree_ram_offset as u64)),
        FpVar::constant(F::from(csc.rule_ram_offset as u64)),
        FpVar::constant(F::from(csc.np_ram_offset as u64)),
    ];

    let mut prev_round_flag = Boolean::new_witness(cs.clone(), || Ok(false))?;

    let mut switch_flag: Boolean<F>;

    for _ in 0..csc.batch_size {
        let past_last_node = next_wires.count.is_eq(&tree_size)?;
        switch_flag = &past_last_node | &prev_round_flag;
        let switch_or_empty = &switch_flag | &is_empty;

        next_wires.cur_node_id = switch_or_empty.select(&tree_null_val, &next_wires.cur_node_id)?;

        next_wires.parent_id = switch_or_empty.select(&tree_null_val, &next_wires.parent_id)?;

        next_wires = node_circuit(
            csc,
            next_wires,
            memory,
            &(!switch_or_empty),
            &tree_null_val,
            &shift,
            &offsets,
            cs.clone(),
        )?;

        prev_round_flag = switch_flag;
    }

    next_wires.count.conditional_enforce_equal(
        &(wires.count.clone() + F::from(csc.batch_size as u32)),
        &is_empty.not(),
    )?;

    let is_last_round = next_wires.count.value()? >= csc.tree_size;

    ivcify(
        wires,
        &mut next_wires,
        is_last_round,
        memory,
        cs.clone(),
        csc,
    )?;

    Ok(next_wires)
}

#[tracing::instrument(target = "gr1cs")]
pub fn ivcify<F: ArkPrimeField>(
    old_wires: &mut CoralWires<F>,
    new_wires: &mut CoralWires<F>,
    last_round: bool,
    memory: &mut RunningMemWires<F>,
    cs: ConstraintSystemRef<F>,
    csc: &mut CoralStepCircuit<F>,
) -> Result<(), SynthesisError> {
    csc.mem.as_mut().unwrap().scan(memory, last_round)?;

    let mut l_vals: Vec<FpVar<F>> = Vec::new();
    let mut r_vals: Vec<FpVar<F>> = Vec::new();

    csc.mem.as_mut().unwrap().ivcify(memory)?;

    let (running_eval_in, running_eval_out) = FpVar::new_input_output_pair(
        cs.clone(),
        || old_wires.running_eval.value(),
        || new_wires.running_eval.value(),
    )?;
    old_wires.running_eval.enforce_equal(&running_eval_in)?;
    new_wires.running_eval.enforce_equal(&running_eval_out)?;

    let (doc_ctr_in, doc_ctr_out) = FpVar::new_input_output_pair(
        cs.clone(),
        || old_wires.doc_ctr.value(),
        || new_wires.doc_ctr.value(),
    )?;
    l_vals.push(old_wires.doc_ctr.clone());
    r_vals.push(doc_ctr_in.clone());
    l_vals.push(new_wires.doc_ctr.clone());
    r_vals.push(doc_ctr_out.clone());

    let (cur_node_id_in, cur_node_id_out) = FpVar::new_input_output_pair(
        cs.clone(),
        || old_wires.cur_node_id.value(),
        || new_wires.cur_node_id.value(),
    )?;
    l_vals.push(old_wires.cur_node_id.clone());
    r_vals.push(cur_node_id_in.clone());
    l_vals.push(new_wires.cur_node_id.clone());
    r_vals.push(cur_node_id_out.clone());

    let (parent_id_in, parent_id_out) = FpVar::new_input_output_pair(
        cs.clone(),
        || old_wires.parent_id.value(),
        || new_wires.parent_id.value(),
    )?;
    l_vals.push(old_wires.parent_id.clone());
    r_vals.push(parent_id_in.clone());
    l_vals.push(new_wires.parent_id.clone());
    r_vals.push(parent_id_out.clone());

    let (np_rule_in, np_rule_out) = FpVar::new_input_output_pair(
        cs.clone(),
        || old_wires.np_rule.value(),
        || new_wires.np_rule.value(),
    )?;
    l_vals.push(old_wires.np_rule.clone());
    r_vals.push(np_rule_in.clone());
    l_vals.push(new_wires.np_rule.clone());
    r_vals.push(np_rule_out.clone());

    let (atom_sp_in, atom_sp_out) = FpVar::new_input_output_pair(
        cs.clone(),
        || old_wires.atom_parent_id.value(),
        || new_wires.atom_parent_id.value(),
    )?;
    l_vals.push(old_wires.atom_parent_id.clone());
    r_vals.push(atom_sp_in.clone());
    l_vals.push(new_wires.atom_parent_id.clone());
    r_vals.push(atom_sp_out.clone());

    let (atom_flag_in, atom_flag_out) = Boolean::new_input_output_pair(
        cs.clone(),
        || old_wires.atom_flag.value(),
        || new_wires.atom_flag.value(),
    )?;
    l_vals.push(old_wires.atom_flag.clone().into());
    r_vals.push(atom_flag_in.into());
    l_vals.push(new_wires.atom_flag.clone().into());
    r_vals.push(atom_flag_out.into());

    let (np_sp_in, np_sp_out) = FpVar::new_input_output_pair(
        cs.clone(),
        || old_wires.np_parent_id.value(),
        || new_wires.np_parent_id.value(),
    )?;
    l_vals.push(old_wires.np_parent_id.clone());
    r_vals.push(np_sp_in.clone());
    l_vals.push(new_wires.np_parent_id.clone());
    r_vals.push(np_sp_out.clone());

    let (count_in, count_out) = FpVar::new_input_output_pair(
        cs.clone(),
        || old_wires.count.value(),
        || new_wires.count.value(),
    )?;
    l_vals.push(old_wires.count.clone());
    r_vals.push(count_in.clone());
    l_vals.push(new_wires.count.clone());
    r_vals.push(count_out.clone());

    chunk_cee(&Boolean::TRUE, &l_vals, &r_vals, csc, cs)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::parser::*;
    use crate::prover::{run_doc_committer, setup};
    use crate::{circuit::*, solver::InterRoundWires, util::*};
    use ark_bn254::Fr as F;
    use ark_relations::gr1cs::{
        ConstraintSystem,
        trace::{ConstraintLayer, TracingMode},
    };
    use std::fs;
    use tracing_subscriber::{Registry, layer::SubscriberExt};

    pub fn full_test_function_multi(pest_file: String, input: String) {
        let grammar = fs::read_to_string(pest_file).expect("Failed to read grammar file");
        let input_text = fs::read_to_string(input).expect("Failed to read input file");

        let mut grammar_graph = GrammarGraph::new();
        grammar_graph
            .parse_text_and_build_graph(&grammar, &input_text)
            .expect("Failed to parse input");

        // Convert the petgraph tree to a left-child right-sibling tree
        grammar_graph.parse_and_convert_lcrs();

        println!("Tree Size {:?}", grammar_graph.lcrs_tree.node_count());

        let nodes_per_step = 1;

        let (ark_ck, _) = gen_ark_pp(input_text.len());

        let doc_commit = run_doc_committer(&input_text.chars().collect(), &ark_ck);

        let (_, mut base, _, _) =
            setup::<AF>(&grammar_graph, nodes_per_step, doc_commit.blind).unwrap();

        let mut irw = InterRoundWires::new();

        let n_rounds = u32::div_ceil(
            grammar_graph.lcrs_tree.node_count() as u32,
            nodes_per_step as u32,
        ) as usize;

        let constraint_layer = ConstraintLayer::new(TracingMode::OnlyConstraints);
        let subscriber = Registry::default()
            // .with(fmt::layer()) // Optional: Log formatted output to stdout
            .with(constraint_layer);

        let _ = tracing::subscriber::set_global_default(subscriber);

        for i in 0..n_rounds {
            println!("===========================");
            println!("batch {:?}", i);

            let constraint_layer = ConstraintLayer::new(TracingMode::OnlyConstraints);
            let _subscriber = Registry::default()
                // .with(fmt::layer()) // Optional: Log formatted output to stdout
                .with(constraint_layer);

            let cs = ConstraintSystem::<F>::new_ref();

            let mut wires = CoralWires::wires_from_irw(&irw, cs.clone(), &mut base, i);

            let mut memory = base
                .mem
                .as_mut()
                .unwrap()
                .begin_new_circuit(cs.clone())
                .unwrap();

            let wires_res = multi_node_step(&mut base, &mut wires, &mut memory, cs.clone());

            if !wires_res.is_ok() {
                // If it isn't, find out the offending constraint.
                println!("{:?}", wires_res);
            }

            assert!(wires_res.is_ok(), "Failed at iter {}", i);

            let res = wires_res.unwrap();

            irw.update(res);

            cs.finalize();

            let is_sat = cs.is_satisfied().unwrap();

            if !is_sat {
                let trace = cs.which_is_unsatisfied().unwrap().unwrap();
                println!(
                    "The constraint system was not satisfied; here is a trace indicating which constraint was unsatisfied: \n{trace}",
                )
            }

            assert!(is_sat, "Not sat at iter {}", i);

            println!("end of {:?}", i);
            println!("n constraints: {:}", cs.num_constraints());
            println!("n witnesses: {:}", cs.num_witness_variables());
        }
    }

    #[test]
    fn full_test_multi_atomic() {
        full_test_function_multi(
            "grammars/test_atomic.pest".to_string(),
            "tests/test_docs/test_atomic.txt".to_string(),
        );
    }

    #[test]
    fn full_test_multi_np() {
        full_test_function_multi(
            "grammars/test_np.pest".to_string(),
            "tests/test_docs/test_np.txt".to_string(),
        );
    }

    #[test]
    fn full_test_multi_any() {
        full_test_function_multi(
            "grammars/test_any.pest".to_string(),
            "tests/test_docs/test_any.txt".to_string(),
        );
    }

    #[test]
    fn full_test_multi_ws() {
        full_test_function_multi(
            "grammars/test_ws.pest".to_string(),
            "tests/test_docs/test_ws.txt".to_string(),
        );
    }

    #[test]
    fn full_test_multi_simple() {
        full_test_function_multi(
            "grammars/test_simple.pest".to_string(),
            "tests/test_docs/test_simple.txt".to_string(),
        );
    }

    #[test]
    fn full_test_multi_json() {
        full_test_function_multi(
            "grammars/json.pest".to_string(),
            "./tests/test_docs/json/test_json_128.txt".to_string(),
        );
    }

    #[test]
    fn full_test_multi_c() {
        full_test_function_multi(
            "grammars/c_simple.pest".to_string(),
            "./tests/test_docs/c/c1.txt".to_string(),
        );
    }

    #[test]
    fn full_test_multi_toml() {
        full_test_function_multi(
            "grammars/toml.pest".to_string(),
            "./tests/test_docs/toml/t1.txt".to_string(),
        );
    }
}
